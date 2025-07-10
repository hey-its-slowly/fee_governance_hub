use {
    crate::{error::ContractError, state::*, constant::*},
    anchor_lang::prelude::*,
    mpl_core::instructions::{TransferV1Builder, TransferV1Cpi, TransferV1InstructionArgs}
};

#[derive(Accounts)]
pub struct CreateAuctionV2<'info> {
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
    /// CHECK: Checked in mpl-core.
    #[account(mut)]
    pub collection: Option<AccountInfo<'info>>,

    /// CHECK: we read this key only
    #[account(mut)]
    pub nft_mint: UncheckedAccount<'info>,

    /// The SPL Noop program.
    /// CHECK: Checked in mpl-core.
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// The MPL Core program.
    /// CHECK: Checked in mpl-core.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,

    /// CHECK: we read this key only
    pub accepted_mint: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateAuctionV2>,
    start_price: u64,
    start_time: i64,
    end_time: i64,
    destination: Option<Pubkey>,
    burn_proceeds: bool,
    tag: u64,
    tick_option: u8,
    tick_amount: u64,
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
    auction.prize_type = 2; // 2: CORE
    auction.num_bids = 0;
    auction.collection = ctx.accounts.collection.as_ref().map(|account_info| account_info.key()).unwrap_or_default();
    auction.bump = ctx.bumps.auction;
    auction.tick_option = tick_option;
    auction.tick_amount = tick_amount;

    // Transfer NFT to vault
    let mut transfer_builder = TransferV1Builder::new();
    transfer_builder
        .asset(ctx.accounts.nft_mint.key())
        .payer(ctx.accounts.creator.key())
        .new_owner(auction.key());

    TransferV1Cpi {
        asset: &ctx.accounts.nft_mint.to_account_info(),
        collection: ctx.accounts.collection.as_ref(),
        payer: &ctx.accounts.creator.to_account_info(),
        authority: Some(&ctx.accounts.creator.as_ref()),
        new_owner: &auction.to_account_info(),
        system_program: Some(ctx.accounts.system_program.as_ref()),
        log_wrapper: ctx.accounts.log_wrapper.as_ref(),
        __program: &ctx.accounts.mpl_core,
        __args: TransferV1InstructionArgs {
            compression_proof: None,
        },
    }
    .invoke()?;

    if auction.is_native_accepted_mint() {
        require!(!auction.burn_proceeds, ContractError::InvalidMint);
    }

    Ok(())
}