use {
    crate::{error::ContractError, state::*, constant::*},
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct CreateAuction<'info> {
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
    pub nft_mint: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = creator,
    )]
    pub creator_nft_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = auction,
    )]
    pub vault_nft_account: Account<'info, TokenAccount>,

    /// CHECK: we read this key only
    pub accepted_mint: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
  ctx: Context<CreateAuction>,
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
  auction.prize_type = 1; // 1: Standard NFT
  auction.num_bids = 0;
  auction.collection = ctx.accounts.collection.as_ref().map(|account_info| account_info.key()).unwrap_or_default();
  auction.bump = ctx.bumps.auction;
  auction.tick_option = tick_option;
  auction.tick_amount = tick_amount;

  // Transfer NFT to vault
  let transfer_ctx = CpiContext::new(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
          from: ctx.accounts.creator_nft_account.to_account_info(),
          to: ctx.accounts.vault_nft_account.to_account_info(),
          authority: ctx.accounts.creator.to_account_info(),
      },
  );
  
  anchor_spl::token::transfer(transfer_ctx, 1)?;

  if auction.is_native_accepted_mint() {
      require!(!auction.burn_proceeds, ContractError::InvalidMint);
  }

  Ok(())
}