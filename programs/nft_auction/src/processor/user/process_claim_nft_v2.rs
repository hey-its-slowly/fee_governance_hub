use {
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, Mint, TokenAccount, Transfer, Burn},
    crate::{error::ContractError, state::*, utils::*},
    mpl_core::instructions::{TransferV1Builder, TransferV1Cpi, TransferV1InstructionArgs}
};

#[derive(Accounts)]
pub struct ClaimNftV2<'info> {
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

    /// CHECK: we read this key only
    #[account(
        mut,
    )]
    pub destination: UncheckedAccount<'info>,

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
    )]
    pub vault_token_account: Option<Account<'info, TokenAccount>>,

    #[account(
        mut        
    )]
    pub destination_token_account: Option<Account<'info, TokenAccount>>,

    #[account(
        mut        
    )]
    pub fee_token_account: Option<Account<'info, TokenAccount>>,

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
    pub accepted_mint: Account<'info, Mint>,

    /// The SPL Noop program.
    /// CHECK: Checked in mpl-core.
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// The MPL Core program.
    /// CHECK: Checked in mpl-core.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimNftV2>) -> Result<()> {
    let auction = &ctx.accounts.auction;
    let creator_account = &ctx.accounts.creator_account;

    if !auction.is_native_accepted_mint() {
        // Verify destination token account
        require!(
            ctx.accounts.destination_token_account.as_ref().unwrap().mint == ctx.accounts.accepted_mint.key(),
            ContractError::InvalidDestinationMint
        );

        require!(
            ctx.accounts.destination_token_account.as_ref().unwrap().owner == auction.destination,
            ContractError::InvalidDestinationAccount
        );
    }
    
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

    // Transfer NFT to claimer
    let mut transfer_builder = TransferV1Builder::new();
    transfer_builder
        .asset(ctx.accounts.nft_mint.key())
        .payer(ctx.accounts.authority.key())
        .new_owner(ctx.accounts.claimer.key());

    TransferV1Cpi {
        asset: &ctx.accounts.nft_mint.to_account_info(),
        collection: ctx.accounts.collection.as_ref(),
        payer: &ctx.accounts.authority.to_account_info(),
        authority: Some(&ctx.accounts.auction.to_account_info()),
        new_owner: &ctx.accounts.claimer.as_ref(),
        system_program: Some(ctx.accounts.system_program.as_ref()),
        log_wrapper: ctx.accounts.log_wrapper.as_ref(),
        __program: &ctx.accounts.mpl_core,
        __args: TransferV1InstructionArgs {
            compression_proof: None,
        },
    }
    .invoke_signed(signer)?;

    // If there were bids and we're not burning proceeds, transfer them to destination
    if auction.current_bid > 0 {
        let fee = calculate_fee(creator_account.fee_type, creator_account.fee_amount, auction.current_bid, ctx.accounts.accepted_mint.decimals);
        let proceeds = auction.current_bid - fee;

        if auction.is_native_accepted_mint() {
            **ctx.accounts.auction.to_account_info().try_borrow_mut_lamports()? -= ctx.accounts.auction.current_bid;
            **ctx.accounts.destination.to_account_info().try_borrow_mut_lamports()? += proceeds;
            if fee > 0 {
                **ctx.accounts.fee_wallet.to_account_info().try_borrow_mut_lamports()? += fee;
            }
        } else {
            if !auction.burn_proceeds {
                let proceeds_transfer_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.vault_token_account.as_ref().unwrap().to_account_info(),
                        to: ctx.accounts.destination_token_account.as_ref().unwrap().to_account_info(),
                        authority: ctx.accounts.auction.to_account_info(),
                    },
                    signer,
                );
                anchor_spl::token::transfer(proceeds_transfer_ctx, proceeds)?;

                if fee > 0 {
                    // Verify destination token account
                    require!(
                        ctx.accounts.fee_token_account.as_ref().unwrap().mint == ctx.accounts.accepted_mint.key(),
                        ContractError::InvalidDestinationMint
                    );

                    require!(
                        ctx.accounts.fee_token_account.as_ref().unwrap().owner == ctx.accounts.fee_wallet.key(),
                        ContractError::InvalidFeeWallet
                    );
                    
                    let fee_transfer_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.vault_token_account.as_ref().unwrap().to_account_info(),
                            to: ctx.accounts.fee_token_account.as_ref().unwrap().to_account_info(),
                            authority: ctx.accounts.auction.to_account_info(),
                        },
                        signer,
                    );
                    anchor_spl::token::transfer(fee_transfer_ctx, fee)?;
                }
            } else {
                let burn_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Burn {
                        mint: ctx.accounts.accepted_mint.to_account_info(),
                        from: ctx.accounts.vault_token_account.as_ref().unwrap().to_account_info(),
                        authority: ctx.accounts.auction.to_account_info(),
                    },
                    signer,
                );
                anchor_spl::token::burn(burn_ctx, ctx.accounts.auction.current_bid)?;
            }

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
    }

    // Mark auction as ended
    let auction = &mut ctx.accounts.auction;
    auction.ended = true;

    Ok(())
}