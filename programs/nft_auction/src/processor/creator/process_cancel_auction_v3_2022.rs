use {
    anchor_lang::prelude::*,
    crate::{error::ContractError, state::*},
    anchor_spl::token_interface::{
        self, Token2022, Mint as Token2022Mint, TokenAccount as Token2022TokenAccount
    },
};

#[derive(Clone)]
pub struct SPLCompression;

impl anchor_lang::Id for SPLCompression {
  fn id() -> Pubkey {
    spl_account_compression::id()
  }
}

#[derive(Accounts)]
pub struct CancelAuctionV32022<'info> {
    #[account(
        mut,
        constraint = creator.key() == auction.creator @ ContractError::UnauthorizedCanceller
    )]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"auction", auction.nft_mint.as_ref()],
        bump,
        constraint = !auction.ended @ ContractError::AlreadyClaimed,
        close = creator
    )]
    pub auction: Account<'info, Auction>,

    /// CHECK: we read this key only
    pub nft_mint: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = accepted_mint,
        token::authority = auction,
    )]
    pub vault_token_account: Box<InterfaceAccount<'info, Token2022TokenAccount>>,

    pub accepted_mint: Box<InterfaceAccount<'info, Token2022Mint>>,

    #[account(mut)]
    #[account(
        seeds = [merkle_tree.key().as_ref()],
        bump,
        seeds::program = bubblegum_program.key()
    )]
    /// CHECK: This account is modified in the downstream program
    pub tree_authority: UncheckedAccount<'info>,
    
    /// CHECK: Leaf might be delegated to another
    pub leaf_delegate: UncheckedAccount<'info>,
    
    #[account(mut)]
    /// CHECK: This account is neither written to nor read from.
    pub merkle_tree: UncheckedAccount<'info>,
  
    /// CHECK: This account is neither written to nor read from.
    pub log_wrapper: UncheckedAccount<'info>,
  
    pub compression_program: Program<'info, SPLCompression>,
  
    /// CHECK: This account is neither written to nor read from.
    pub bubblegum_program: UncheckedAccount<'info>,
  
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CancelAuctionV32022<'info>>,
    // Cnft parameters
    root: [u8; 32],
    data_hash: [u8; 32],
    creator_hash: [u8; 32],
    nonce: u64,
    index: u32,
) -> Result<()> {
    // Verify there are no bids
    require!(ctx.accounts.auction.num_bids == 0, ContractError::AuctionHasBids);

    let seeds = &[
        b"auction",
        ctx.accounts.auction.nft_mint.as_ref(),
        &[ctx.accounts.auction.bump],
    ];
    let signer = &[&seeds[..]];

    // Return NFT to creator
    let tree_config = ctx.accounts.tree_authority.to_account_info();
    let leaf_owner = ctx.accounts.auction.to_account_info();
    let new_leaf_owner = ctx.accounts.creator.to_account_info();
    let leaf_delegate = ctx.accounts.leaf_delegate.to_account_info();
    let merkle_tree = ctx.accounts.merkle_tree.to_account_info();
    let log_wrapper = ctx.accounts.log_wrapper.to_account_info();
    let compression_program = ctx.accounts.compression_program.to_account_info();
    let system_program = ctx.accounts.system_program.to_account_info();

    let cnft_transfer_cpi = mpl_bubblegum::instructions::TransferCpi::new(
        &ctx.accounts.bubblegum_program,
        mpl_bubblegum::instructions::TransferCpiAccounts {
            tree_config: &tree_config,
            leaf_owner: (&leaf_owner, true),
            leaf_delegate: (&leaf_delegate, false),
            new_leaf_owner: &new_leaf_owner,
            merkle_tree: &merkle_tree,
            log_wrapper: &log_wrapper,
            compression_program: &compression_program,
            system_program: &system_program,
        },
        mpl_bubblegum::instructions::TransferInstructionArgs {
            root: root,
            data_hash: data_hash,
            creator_hash: creator_hash,
            nonce: nonce,
            index: index,
        },
    );

    cnft_transfer_cpi.invoke_signed_with_remaining_accounts(
        signer, 
        ctx.remaining_accounts
        .iter()
        .map(|account| (account, false, false))
        .collect::<Vec<_>>().as_slice()
    )?;

    // Close vault accounts
    token_interface::close_account(
        CpiContext::new_with_signer(
            ctx.accounts.token_2022_program.to_account_info(),
            token_interface::CloseAccount {
              account: ctx.accounts.vault_token_account.to_account_info(),
              destination: ctx.accounts.creator.to_account_info(),
              authority: ctx.accounts.auction.to_account_info(),
            },
            signer
        )
    )?;

    Ok(())
}