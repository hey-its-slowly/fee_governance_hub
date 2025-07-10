use {
    crate::{error::ContractError, state::*, constant::*},
    anchor_lang::prelude::*,
};

#[derive(Clone)]
pub struct SPLCompression;

impl anchor_lang::Id for SPLCompression {
  fn id() -> Pubkey {
    spl_account_compression::id()
  }
}

#[derive(Accounts)]
pub struct CreateAuctionV3<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
      seeds = [b"creator", creator.key().as_ref()],
      bump,
      constraint = creator_account.is_creator_available(creator.key())? @ ContractError::InvalidAuthority
    )]
    pub creator_account: Account<'info, Creator>,

    #[account(
        init,
        payer = creator,
        space = 8 + std::mem::size_of::<Auction>(),
        seeds = [b"auction", nft_mint.key().as_ref()],
        bump,
    )]
    pub auction: Account<'info, Auction>,

    /// The collection to which the asset belongs.
    #[account(mut)]
    pub collection: Option<AccountInfo<'info>>,

    /// CHECK: we read this key only
    pub nft_mint: UncheckedAccount<'info>,

    /// CHECK: we read this key only
    pub accepted_mint: UncheckedAccount<'info>,

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
  
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateAuctionV3<'info>>,
    start_price: u64,
    start_time: i64,
    end_time: i64,
    destination: Option<Pubkey>,
    burn_proceeds: bool,
    tag: u64,
    tick_option: u8,
    tick_amount: u64,
    // Cnft parameters
    root: [u8; 32],
    data_hash: [u8; 32],
    creator_hash: [u8; 32],
    nonce: u64,
    index: u32,
) -> Result<()> {
    let auction = &mut ctx.accounts.auction;
    
    // Validate timestamps
    let clock = Clock::get()?;
    require!(start_time > clock.unix_timestamp, ContractError::InvalidStartTime);
    require!(end_time > start_time, ContractError::InvalidEndTime);
    require!(tick_option == TICK_OPTION_PERCENTAGE || tick_option == TICK_OPTION_FLAT, ContractError::InvalidTickOption);
    
    // Initialize auction
    auction.creator = ctx.accounts.creator.key();
    auction.nft_mint = ctx.accounts.nft_mint.key();
    auction.accepted_mint = ctx.accounts.accepted_mint.key();
    auction.ended = false;
    auction.tag = tag;
    auction.start_price = start_price;
    auction.current_bid = 0;
    auction.current_winner = None;
    auction.start_time = start_time;
    auction.end_time = end_time;
    auction.destination = destination.unwrap_or(ctx.accounts.creator.key());
    auction.burn_proceeds = burn_proceeds;
    auction.prize_type = 3; // 3: CNFT
    auction.num_bids = 0;
    auction.collection = ctx.accounts.collection.as_ref().map(|account_info| account_info.key()).unwrap_or_default();
    auction.bump = ctx.bumps.auction;
    auction.tick_option = tick_option;
    auction.tick_amount = tick_amount;

    // Transfer NFT to vault
    let tree_config = ctx.accounts.tree_authority.to_account_info();
    let leaf_owner = ctx.accounts.creator.to_account_info();
    let leaf_delegate = ctx.accounts.leaf_delegate.to_account_info();
    let new_leaf_owner = auction.to_account_info();
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

    cnft_transfer_cpi.invoke_with_remaining_accounts(
        ctx.remaining_accounts
        .iter()
        .map(|account| (account, false, false))
        .collect::<Vec<_>>().as_slice()
    )?;

    if auction.is_native_accepted_mint() {
        require!(!auction.burn_proceeds, ContractError::InvalidMint);
    }

    Ok(())
}