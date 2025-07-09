use {
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, Mint, TokenAccount, Transfer},
    crate::{error::ContractError, state::*, constant::*},
};

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,

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
    pub bidder_token_account: Option<Account<'info, TokenAccount>>,
    
    /// CHECK: Optional previous bidder
    #[account(
        mut,
        constraint = previous_bidder.key() == auction.current_winner.unwrap_or(bidder.key()) @ ContractError::InvalidPreviousBidder
    )]
    pub previous_bidder: Option<UncheckedAccount<'info>>,

    /// CHECK: Optional previous bidder's token account
    #[account(
        mut,
        token::mint = accepted_mint,
        token::authority = auction.current_winner.unwrap_or(bidder.key()),
    )]
    pub previous_bidder_token_account: Option<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = accepted_mint,
        token::authority = auction,
    )]
    pub vault_token_account: Option<Account<'info, TokenAccount>>,

    pub accepted_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<PlaceBid>, bid_amount: u64) -> Result<()> {
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

    if ctx.accounts.auction.is_native_accepted_mint() {
        // Transfer new bid amount to vault
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.bidder.to_account_info(),
                    to: ctx.accounts.auction.to_account_info(),
                }
            ),
            bid_amount
        )?;

        // Return funds to previous bidder if exists
        if ctx.accounts.auction.current_winner.is_some() {
            **ctx.accounts.auction.to_account_info().try_borrow_mut_lamports()? -= ctx.accounts.auction.current_bid;
            **ctx.accounts.previous_bidder.as_ref().unwrap().to_account_info().try_borrow_mut_lamports()? += ctx.accounts.auction.current_bid;
        }
    } else  {
        // Return funds to previous bidder if exists
        if ctx.accounts.auction.current_winner.is_some() {            
            let seeds = &[
                b"auction",
                ctx.accounts.auction.nft_mint.as_ref(),
                &[ctx.accounts.auction.bump],
            ];
            let signer = &[&seeds[..]];

            let transfer_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_token_account.as_ref().unwrap().to_account_info(),
                    to: ctx.accounts.previous_bidder_token_account.as_ref().unwrap().to_account_info(),
                    authority: ctx.accounts.auction.to_account_info(),
                },
                signer,
            );
            
            anchor_spl::token::transfer(transfer_ctx, ctx.accounts.auction.current_bid)?;
        }

        // Transfer new bid amount to vault
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.bidder_token_account.as_ref().unwrap().to_account_info(),
                to: ctx.accounts.vault_token_account.as_ref().unwrap().to_account_info(),
                authority: ctx.accounts.bidder.to_account_info(),
            },
        );
        
        anchor_spl::token::transfer(transfer_ctx, bid_amount)?;
    }

    // Update auction state
    let auction = &mut ctx.accounts.auction;
    auction.current_bid = bid_amount;
    auction.current_winner = Some(ctx.accounts.bidder.key());
    auction.num_bids += 1;

    Ok(())
}