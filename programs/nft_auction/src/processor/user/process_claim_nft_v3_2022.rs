use {
    anchor_lang::prelude::*,
    anchor_spl::token::Token,
    anchor_spl::token_interface::{
        self, Token2022, Mint as Token2022Mint, TokenAccount as Token2022TokenAccount, TransferChecked, Burn as Burn2022,
    },
    crate::{error::ContractError, state::*, utils::*},
};

#[derive(Clone)]
pub struct SPLCompression;

impl anchor_lang::Id for SPLCompression {
  fn id() -> Pubkey {
    spl_account_compression::id()
  }
}

#[derive(Accounts)]
pub struct ClaimNftV32022<'info> {
    #[account(
        mut,
        constraint = is_super_admin(authority.key) || authority.key() == claimer.key() @ ContractError::InvalidAuthority
    )]
    pub authority: Signer<'info>,

    /// CHECK: we read this key only
    #[account(
        mut,
    )]
    pub claimer: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"auction", auction.nft_mint.as_ref()],
        bump,
        constraint = !auction.ended @ ContractError::AlreadyClaimed,
        constraint = Clock::get()?.unix_timestamp >= auction.end_time @ ContractError::AuctionNotEnded,
        constraint = auction.creator == creator.key() @ ContractError::InvalidCreator,
        close = creator
    )]
    pub auction: Account<'info, Auction>,

    /// CHECK: we read this key only
    #[account(
        mut,
    )] 
    pub creator: UncheckedAccount<'info>,

    /// The collection to which the asset belongs.
    /// CHECK: Checked in mpl-core.
    #[account(mut)]
    pub collection: Option<AccountInfo<'info>>,

    /// CHECK: we read this key only
    #[account(mut)]
    pub nft_mint: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = accepted_mint,
        token::authority = auction,
    )]
    pub vault_token_account: Box<InterfaceAccount<'info, Token2022TokenAccount>>,

    #[account(
        mut        
    )]
    pub destination_token_account: Box<InterfaceAccount<'info, Token2022TokenAccount>>,

    #[account(
        mut        
    )]
    pub fee_token_account: Option<Box<InterfaceAccount<'info, Token2022TokenAccount>>>,

    #[account(
      seeds = [b"creator", creator.key().as_ref()],
      bump,
      constraint = creator_account.fee_wallet == fee_wallet.key() @ ContractError::InvalidFeeWallet,
    )]
    pub creator_account: Box<Account<'info, Creator>>,

    /// CHECK: we read this key only
    #[account(
        mut,
    )] 
    pub fee_wallet: UncheckedAccount<'info>,   

    #[account(
        mut        
    )]
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

    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ClaimNftV32022<'info>>,
    root: [u8; 32],
    data_hash: [u8; 32],
    creator_hash: [u8; 32],
    nonce: u64,
    index: u32,
) -> Result<()> {
    let auction = &ctx.accounts.auction;
    let creator_account = &ctx.accounts.creator_account;

    // Verify destination token account
    require!(
        ctx.accounts.destination_token_account.owner == auction.destination,
        ContractError::InvalidDestinationAccount
    );
    require!(
        ctx.accounts.destination_token_account.mint == ctx.accounts.accepted_mint.key(),
        ContractError::InvalidDestinationMint
    );
    
    // Check if auction has a winner
    match auction.current_winner {
        // If there's a winner, only they can claim
        Some(winner) => {
            require!(
                ctx.accounts.claimer.key() == winner,
                ContractError::UnauthorizedClaimer
            );
        },
        // If no winner (no bids), only creator can claim
        None => {
            require!(
                ctx.accounts.claimer.key() == auction.creator,
                ContractError::UnauthorizedClaimer
            );
        }
    }

    let seeds = &[
        b"auction",
        ctx.accounts.auction.nft_mint.as_ref(),
        &[ctx.accounts.auction.bump],
    ];
    let signer = &[&seeds[..]];

    // Return NFT to creator
    let tree_config = ctx.accounts.tree_authority.to_account_info();
    let leaf_owner = ctx.accounts.auction.to_account_info();
    let new_leaf_owner = ctx.accounts.claimer.to_account_info();
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

    // If there were bids and we're not burning proceeds, transfer them to destination
    if auction.current_bid > 0 {
        let current_bid_balance = ctx.accounts.vault_token_account.amount;
        let fee = calculate_fee(creator_account.fee_type, creator_account.fee_amount, current_bid_balance, ctx.accounts.accepted_mint.decimals);
        let proceeds = current_bid_balance - fee;

        if !auction.burn_proceeds {
            let cpi_accounts = TransferChecked {
                from: ctx.accounts.vault_token_account.to_account_info().clone(),
                mint: ctx.accounts.accepted_mint.to_account_info().clone(),
                to: ctx.accounts.destination_token_account.to_account_info().clone(),
                authority: ctx.accounts.auction.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_2022_program.to_account_info();
            let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token_interface::transfer_checked(cpi_context, proceeds, ctx.accounts.accepted_mint.decimals)?;

            if fee > 0 {
                // Verify fee token account
                require!(
                    ctx.accounts.fee_token_account.as_ref().unwrap().owner == ctx.accounts.fee_wallet.key(),
                    ContractError::InvalidFeeWallet
                );
                require!(
                    ctx.accounts.fee_token_account.as_ref().unwrap().mint == ctx.accounts.accepted_mint.key(),
                    ContractError::InvalidDestinationMint
                );
                
                let cpi_accounts = TransferChecked {
                    from: ctx.accounts.vault_token_account.to_account_info().clone(),
                    mint: ctx.accounts.accepted_mint.to_account_info().clone(),
                    to: ctx.accounts.fee_token_account.as_ref().unwrap().to_account_info().clone(),
                    authority: ctx.accounts.auction.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_2022_program.to_account_info();
                let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
                token_interface::transfer_checked(cpi_context, fee, ctx.accounts.accepted_mint.decimals)?;
            }
            
        } else {
            let cpi_accounts = Burn2022 {
                mint: ctx.accounts.accepted_mint.to_account_info().clone(),
                from: ctx.accounts.vault_token_account.to_account_info().clone(),
                authority: ctx.accounts.auction.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_2022_program.to_account_info();
            let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token_interface::burn(cpi_context, ctx.accounts.vault_token_account.amount)?;
        }
    }

    // We can't close the token-2022 account unless token authority withdraws fee if it comes with transfer fee extension
    
    // token_interface::close_account(
    //     CpiContext::new_with_signer(
    //         ctx.accounts.token_2022_program.to_account_info(),
    //         token_interface::CloseAccount {
    //           account: ctx.accounts.vault_token_account.to_account_info(),
    //           destination: ctx.accounts.creator.to_account_info(),
    //           authority: ctx.accounts.auction.to_account_info(),
    //         },
    //         signer
    //     )
    // )?;

    // Mark auction as ended
    let auction = &mut ctx.accounts.auction;
    auction.ended = true;

    Ok(())
}