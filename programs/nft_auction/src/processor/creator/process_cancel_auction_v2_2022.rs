use {
    anchor_lang::prelude::*,
    anchor_spl::token::Token,
    crate::{error::ContractError, state::*},
    mpl_core::instructions::{TransferV1Builder, TransferV1Cpi, TransferV1InstructionArgs},
    anchor_spl::token_interface::{
        self, Token2022, Mint as Token2022Mint, TokenAccount as Token2022TokenAccount
    },
};

#[derive(Accounts)]
pub struct CancelAuctionV22022<'info> {
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

    pub accepted_mint: Box<InterfaceAccount<'info, Token2022Mint>>,

    /// The SPL Noop program.
    /// CHECK: Checked in mpl-core.
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// The MPL Core program.
    /// CHECK: Checked in mpl-core.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CancelAuctionV22022>) -> Result<()> {
    // Verify there are no bids
    require!(ctx.accounts.auction.num_bids == 0, ContractError::AuctionHasBids);

    let seeds = &[
        b"auction",
        ctx.accounts.auction.nft_mint.as_ref(),
        &[ctx.accounts.auction.bump],
    ];
    let signer = &[&seeds[..]];

    // Return NFT to creator
    let mut transfer_builder = TransferV1Builder::new();
    transfer_builder
        .asset(ctx.accounts.nft_mint.key())
        .payer(ctx.accounts.creator.key())
        .new_owner(ctx.accounts.creator.key());

    TransferV1Cpi {
        asset: &ctx.accounts.nft_mint.to_account_info(),
        collection: ctx.accounts.collection.as_ref(),
        payer: &ctx.accounts.creator.to_account_info(),
        authority: Some(&ctx.accounts.auction.to_account_info()),
        new_owner: &ctx.accounts.creator.as_ref(),
        system_program: Some(ctx.accounts.system_program.as_ref()),
        log_wrapper: ctx.accounts.log_wrapper.as_ref(),
        __program: &ctx.accounts.mpl_core,
        __args: TransferV1InstructionArgs {
            compression_proof: None,
        },
    }
    .invoke_signed(signer)?;

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