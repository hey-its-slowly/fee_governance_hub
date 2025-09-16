use {
    anchor_lang::prelude::*,
    anchor_spl::token_interface::{
        self, Token2022, Mint as Token2022Mint, TokenAccount as Token2022TokenAccount, TransferChecked,
    },
    crate::{error::ContractError, state::*, constant::*, utils::*},
};

#[derive(Accounts)]
pub struct PlaceBid2022<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,

    #[account(mut,
      constraint = is_super_admin(backend_authority.key) @ ContractError::InvalidAuthority
    )]
    pub backend_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"auction", auction.nft_mint.as_ref()],
        bump,
    )]
    pub auction: Account<'info, Auction>,

    #[account(
        mut,
        token::mint = accepted_mint,
        token::authority = bidder,
    )]
    pub bidder_token_account: Box<InterfaceAccount<'info, Token2022TokenAccount>>,

    /// CHECK: Optional previous bidder's token account
    #[account(
        mut,
        token::mint = accepted_mint,
        token::authority = auction.current_winner.unwrap_or(bidder.key()),
    )]
    pub previous_bidder_token_account: Option<Box<InterfaceAccount<'info, Token2022TokenAccount>>>,

    #[account(
        mut,
        token::mint = accepted_mint,
        token::authority = auction,
    )]
    pub vault_token_account: Box<InterfaceAccount<'info, Token2022TokenAccount>>,

    pub accepted_mint: Box<InterfaceAccount<'info, Token2022Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<PlaceBid2022>, bid_amount: u64) -> Result<()> {
    // Validate auction timing
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp >= ctx.accounts.auction.start_time,
        ContractError::AuctionNotStarted
    );
    require!(
        clock.unix_timestamp < ctx.accounts.auction.end_time,
        ContractError::AuctionEnded
    );
    
    // Validate bid amount
    let minimum_bid = if ctx.accounts.auction.current_bid == 0 {
        ctx.accounts.auction.start_price
    } else {
        if ctx.accounts.auction.tick_option == TICK_OPTION_PERCENTAGE {
            ctx.accounts.auction.current_bid + (ctx.accounts.auction.current_bid / 100) * ctx.accounts.auction.tick_amount
        } else if ctx.accounts.auction.tick_option == TICK_OPTION_FLAT {
            ctx.accounts.auction.current_bid + ctx.accounts.auction.tick_amount
        } else {
            ctx.accounts.auction.current_bid + (ctx.accounts.auction.current_bid / 100)
        }
    };
    require!(bid_amount >= minimum_bid, ContractError::BidTooLow);

    // Extend auction if bid is placed in last 5 minutes
    if ctx.accounts.auction.end_time - clock.unix_timestamp < 300 {
        ctx.accounts.auction.end_time = clock.unix_timestamp + 300;
    }

    // Return funds to previous bidder if exists
    if ctx.accounts.auction.current_winner.is_some() {
        let previous_bid_amount = ctx.accounts.vault_token_account.amount;
        
        let seeds = &[
            b"auction",
            ctx.accounts.auction.nft_mint.as_ref(),
            &[ctx.accounts.auction.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.vault_token_account.to_account_info().clone(),
            mint: ctx.accounts.accepted_mint.to_account_info().clone(),
            to: ctx.accounts.previous_bidder_token_account.as_ref().unwrap().to_account_info().clone(),
            authority: ctx.accounts.auction.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token_interface::transfer_checked(cpi_context, previous_bid_amount, ctx.accounts.accepted_mint.decimals)?;
    }

    // Transfer new bid amount to vault
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.bidder_token_account.to_account_info().clone(),
        mint: ctx.accounts.accepted_mint.to_account_info().clone(),
        to: ctx.accounts.vault_token_account.to_account_info().clone(),
        authority: ctx.accounts.bidder.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
    token_interface::transfer_checked(cpi_context, bid_amount, ctx.accounts.accepted_mint.decimals)?;

    // Update auction state
    let auction = &mut ctx.accounts.auction;
    auction.current_bid = bid_amount;
    auction.current_winner = Some(ctx.accounts.bidder.key());
    auction.num_bids += 1;

    Ok(())
}