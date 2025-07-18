use {
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, Mint, TokenAccount, Transfer},
    crate::{error::ContractError, state::*},
};

#[derive(Accounts)]
pub struct CancelAuction<'info> {
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

    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = auction,
    )]
    pub vault_nft_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = creator,
    )]
    pub creator_nft_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = accepted_mint,
        token::authority = auction,
    )]
    pub vault_token_account: Option<Account<'info, TokenAccount>>,

    pub accepted_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CancelAuction>) -> Result<()> {
    // Verify there are no bids
    require!(ctx.accounts.auction.num_bids == 0, ContractError::AuctionHasBids);

    let seeds = &[
        b"auction",
        ctx.accounts.auction.nft_mint.as_ref(),
        &[ctx.accounts.auction.bump],
    ];
    let signer = &[&seeds[..]];

    // Return NFT to creator
    let nft_transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_nft_account.to_account_info(),
            to: ctx.accounts.creator_nft_account.to_account_info(),
            authority: ctx.accounts.auction.to_account_info(),
        },
        signer,
    );
    anchor_spl::token::transfer(nft_transfer_ctx, 1)?;

    // Close vault accounts
    anchor_spl::token::close_account(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: ctx.accounts.vault_nft_account.to_account_info(),
                destination: ctx.accounts.creator.to_account_info(),
                authority: ctx.accounts.auction.to_account_info(),
            },
            signer
        )
    )?;

    if !ctx.accounts.auction.is_native_accepted_mint() {
        anchor_spl::token::close_account(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::CloseAccount {
                    account: ctx.accounts.vault_token_account.as_ref().unwrap().to_account_info(),
                    destination: ctx.accounts.creator.to_account_info(),
                    authority: ctx.accounts.auction.to_account_info(),
                },
                signer
            )
        )?;
    }

    Ok(())
}