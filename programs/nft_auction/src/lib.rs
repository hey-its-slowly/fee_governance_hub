use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer, Burn};
use anchor_spl::token_interface::{
    self, Token2022, Mint as Token2022Mint, TokenAccount as Token2022TokenAccount, TransferChecked, Burn as Burn2022,
};
use mpl_core::instructions::{TransferV1Builder, TransferV1Cpi, TransferV1InstructionArgs};
use anchor_spl::associated_token::{AssociatedToken, create_idempotent};

pub mod account;
pub mod error;

use account::*;
use error::*;
declare_id!("AGopdUYkfyWMehbcvjC1kA2e44hq5kDqTEkVPLckBPrV");

#[program]
pub mod nft_auction {
    use super::*;

    pub fn create_auction(
        ctx: Context<CreateAuction>,
        start_price: u64,
        start_time: i64,
        end_time: i64,
        destination: Option<Pubkey>,
        burn_proceeds: bool,
        tag: u64,
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        
        // Validate timestamps
        let clock = Clock::get()?;
        require!(start_time > clock.unix_timestamp, AuctionCode::InvalidStartTime);
        require!(end_time > start_time, AuctionCode::InvalidEndTime);
        
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
            require!(!auction.burn_proceeds, AuctionCode::InvalidMint);
        }

        Ok(())
    }

    /**
     * @dev Create an auction with a core prize
     */
    pub fn create_auction_v2(
        ctx: Context<CreateAuctionV2>,
        start_price: u64,
        start_time: i64,
        end_time: i64,
        destination: Option<Pubkey>,
        burn_proceeds: bool,
        tag: u64,
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        
        // Validate timestamps
        let clock = Clock::get()?;
        require!(start_time > clock.unix_timestamp, AuctionCode::InvalidStartTime);
        require!(end_time > start_time, AuctionCode::InvalidEndTime);
        
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
            require!(!auction.burn_proceeds, AuctionCode::InvalidMint);
        }

        Ok(())
    }

    pub fn init_auction_vault(ctx: Context<InitAuctionVault>) -> Result<()> {
        let create_idempotent_ctx = CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.authority.to_account_info(),
                associated_token: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.auction.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        );
        create_idempotent(create_idempotent_ctx)?;
        msg!("Initialized vault for mint: {}", ctx.accounts.mint.key());

        Ok(())
    }

    pub fn init_auction_vault_2022(ctx: Context<InitAuctionVault2022>) -> Result<()> {
        let create_idempotent_ctx = CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.authority.to_account_info(),
                associated_token: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.auction.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        );
        create_idempotent(create_idempotent_ctx)?;
        msg!("Initialized vault for token 2022 mint: {}", ctx.accounts.mint.key());

        Ok(())
    }

    pub fn place_bid(ctx: Context<PlaceBid>, bid_amount: u64) -> Result<()> {
        // Validate auction timing
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp >= ctx.accounts.auction.start_time,
            AuctionCode::AuctionNotStarted
        );
        require!(
            clock.unix_timestamp < ctx.accounts.auction.end_time,
            AuctionCode::AuctionEnded
        );
        
        // Validate bid amount
        let minimum_bid = if ctx.accounts.auction.current_bid == 0 {
            ctx.accounts.auction.start_price
        } else {
            ctx.accounts.auction.current_bid + (ctx.accounts.auction.current_bid / 100) // 1% increment
        };
        require!(bid_amount >= minimum_bid, AuctionCode::BidTooLow);

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

    pub fn place_bid_2022(ctx: Context<PlaceBid2022>, bid_amount: u64) -> Result<()> {
        // Validate auction timing
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp >= ctx.accounts.auction.start_time,
            AuctionCode::AuctionNotStarted
        );
        require!(
            clock.unix_timestamp < ctx.accounts.auction.end_time,
            AuctionCode::AuctionEnded
        );
        
        // Validate bid amount
        let minimum_bid = if ctx.accounts.auction.current_bid == 0 {
            ctx.accounts.auction.start_price
        } else {
            ctx.accounts.auction.current_bid + (ctx.accounts.auction.current_bid / 100) // 1% increment
        };
        require!(bid_amount >= minimum_bid, AuctionCode::BidTooLow);

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

    pub fn claim_nft(ctx: Context<ClaimNft>) -> Result<()> {
        let auction = &ctx.accounts.auction;

        if !auction.is_native_accepted_mint() {
            // Verify destination token account
            require!(
                ctx.accounts.destination_token_account.as_ref().unwrap().mint == ctx.accounts.accepted_mint.key(),
                AuctionCode::InvalidDestinationMint
            );

            require!(
                ctx.accounts.destination_token_account.as_ref().unwrap().owner == auction.destination,
                AuctionCode::InvalidDestinationAccount
            );
        }
        
        // Check if auction has a winner
        match auction.current_winner {
            // If there's a winner, only they can claim
            Some(winner) => {
                require!(
                    ctx.accounts.claimer.key() == winner,
                    AuctionCode::UnauthorizedClaimer
                );
            },
            // If no winner (no bids), only creator can claim
            None => {
                require!(
                    ctx.accounts.claimer.key() == auction.creator,
                    AuctionCode::UnauthorizedClaimer
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
        let nft_transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_nft_account.to_account_info(),
                to: ctx.accounts.claimer_nft_account.to_account_info(),
                authority: ctx.accounts.auction.to_account_info(),
            },
            signer,
        );
        anchor_spl::token::transfer(nft_transfer_ctx, 1)?;
    
        // If there were bids and we're not burning proceeds, transfer them to destination
        if auction.current_bid > 0 {
            if auction.is_native_accepted_mint() {
                **ctx.accounts.auction.to_account_info().try_borrow_mut_lamports()? -= ctx.accounts.auction.current_bid;
                **ctx.accounts.claimer.to_account_info().try_borrow_mut_lamports()? += ctx.accounts.auction.current_bid;
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
                    anchor_spl::token::transfer(proceeds_transfer_ctx, ctx.accounts.auction.current_bid)?;
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
                          account: ctx.accounts.vault_nft_account.to_account_info(),
                          destination: ctx.accounts.authority.to_account_info(),
                          authority: ctx.accounts.auction.to_account_info(),
                        },
                        signer
                    )
                )?;
        
                anchor_spl::token::close_account(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        anchor_spl::token::CloseAccount {
                          account: ctx.accounts.vault_token_account.as_ref().unwrap().to_account_info(),
                          destination: ctx.accounts.authority.to_account_info(),
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

    pub fn claim_nft_2022(ctx: Context<ClaimNft2022>) -> Result<()> {
        let auction = &ctx.accounts.auction;

        // Verify destination token account
        require!(
            ctx.accounts.destination_token_account.owner == auction.destination,
            AuctionCode::InvalidDestinationAccount
        );
        require!(
            ctx.accounts.destination_token_account.mint == ctx.accounts.accepted_mint.key(),
            AuctionCode::InvalidDestinationMint
        );
        
        // Check if auction has a winner
        match auction.current_winner {
            // If there's a winner, only they can claim
            Some(winner) => {
                require!(
                    ctx.accounts.claimer.key() == winner,
                    AuctionCode::UnauthorizedClaimer
                );
            },
            // If no winner (no bids), only creator can claim
            None => {
                require!(
                    ctx.accounts.claimer.key() == auction.creator,
                    AuctionCode::UnauthorizedClaimer
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
        let nft_transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_nft_account.to_account_info(),
                to: ctx.accounts.claimer_nft_account.to_account_info(),
                authority: ctx.accounts.auction.to_account_info(),
            },
            signer,
        );
        anchor_spl::token::transfer(nft_transfer_ctx, 1)?;
    
        // If there were bids and we're not burning proceeds, transfer them to destination
        if auction.current_bid > 0 {
            if !auction.burn_proceeds {
                let cpi_accounts = TransferChecked {
                    from: ctx.accounts.vault_token_account.to_account_info().clone(),
                    mint: ctx.accounts.accepted_mint.to_account_info().clone(),
                    to: ctx.accounts.destination_token_account.to_account_info().clone(),
                    authority: ctx.accounts.auction.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_2022_program.to_account_info();
                let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
                token_interface::transfer_checked(cpi_context, ctx.accounts.vault_token_account.amount, ctx.accounts.accepted_mint.decimals)?;
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

    pub fn claim_nft_v2(ctx: Context<ClaimNftV2>) -> Result<()> {
        let auction = &ctx.accounts.auction;

        if !auction.is_native_accepted_mint() {
            // Verify destination token account
            require!(
                ctx.accounts.destination_token_account.as_ref().unwrap().mint == ctx.accounts.accepted_mint.key(),
                AuctionCode::InvalidDestinationMint
            );

            require!(
                ctx.accounts.destination_token_account.as_ref().unwrap().owner == auction.destination,
                AuctionCode::InvalidDestinationAccount
            );
        }
        
        // Check if auction has a winner
        match auction.current_winner {
            // If there's a winner, only they can claim
            Some(winner) => {
                require!(
                    ctx.accounts.claimer.key() == winner,
                    AuctionCode::UnauthorizedClaimer
                );
            },
            // If no winner (no bids), only creator can claim
            None => {
                require!(
                    ctx.accounts.claimer.key() == auction.creator,
                    AuctionCode::UnauthorizedClaimer
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
            if auction.is_native_accepted_mint() {
                **ctx.accounts.auction.to_account_info().try_borrow_mut_lamports()? -= ctx.accounts.auction.current_bid;
                **ctx.accounts.claimer.to_account_info().try_borrow_mut_lamports()? += ctx.accounts.auction.current_bid;
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
                    anchor_spl::token::transfer(proceeds_transfer_ctx, ctx.accounts.auction.current_bid)?;
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

    pub fn claim_nft_v2_2022(ctx: Context<ClaimNftV22022>) -> Result<()> {
        let auction = &ctx.accounts.auction;

        // Verify destination token account
        require!(
            ctx.accounts.destination_token_account.owner == auction.destination,
            AuctionCode::InvalidDestinationAccount
        );
        require!(
            ctx.accounts.destination_token_account.mint == ctx.accounts.accepted_mint.key(),
            AuctionCode::InvalidDestinationMint
        );
        
        // Check if auction has a winner
        match auction.current_winner {
            // If there's a winner, only they can claim
            Some(winner) => {
                require!(
                    ctx.accounts.claimer.key() == winner,
                    AuctionCode::UnauthorizedClaimer
                );
            },
            // If no winner (no bids), only creator can claim
            None => {
                require!(
                    ctx.accounts.claimer.key() == auction.creator,
                    AuctionCode::UnauthorizedClaimer
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
            if !auction.burn_proceeds {
                let cpi_accounts = TransferChecked {
                    from: ctx.accounts.vault_token_account.to_account_info().clone(),
                    mint: ctx.accounts.accepted_mint.to_account_info().clone(),
                    to: ctx.accounts.destination_token_account.to_account_info().clone(),
                    authority: ctx.accounts.auction.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_2022_program.to_account_info();
                let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
                token_interface::transfer_checked(cpi_context, ctx.accounts.vault_token_account.amount, ctx.accounts.accepted_mint.decimals)?;
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

    pub fn cancel_auction(ctx: Context<CancelAuction>) -> Result<()> {
        // Verify there are no bids
        require!(ctx.accounts.auction.num_bids == 0, AuctionCode::AuctionHasBids);

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

    pub fn cancel_auction_2022(ctx: Context<CancelAuction2022>) -> Result<()> {
        // Verify there are no bids
        require!(ctx.accounts.auction.num_bids == 0, AuctionCode::AuctionHasBids);

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

    pub fn cancel_auction_v2(ctx: Context<CancelAuctionV2>) -> Result<()> {
        // Verify there are no bids
        require!(ctx.accounts.auction.num_bids == 0, AuctionCode::AuctionHasBids);

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

    pub fn cancel_auction_v2_2022(ctx: Context<CancelAuctionV22022>) -> Result<()> {
        // Verify there are no bids
        require!(ctx.accounts.auction.num_bids == 0, AuctionCode::AuctionHasBids);

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

    pub fn add_creator(ctx: Context<AddCreator>) -> Result<()> {
        ctx.accounts.creator.bump = ctx.bumps.creator;
        ctx.accounts.creator.wallet = ctx.accounts.creator_wallet.key();
        ctx.accounts.creator.created_at = Clock::get()?.unix_timestamp as u64;

        Ok(())
    }

    pub fn remove_creator(ctx: Context<RemoveCreator>) -> Result<()> {
        msg!("Removed creator: {}", ctx.accounts.creator_wallet.key());

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ClaimNft<'info> {
    #[account(
        mut,
        constraint = is_super_admin(authority.key) || authority.key() == claimer.key() @ AuctionCode::InvalidAuthority
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
        constraint = !auction.ended @ AuctionCode::AlreadyClaimed,
        constraint = Clock::get()?.unix_timestamp >= auction.end_time @ AuctionCode::AuctionNotEnded,
        constraint = auction.creator == creator.key() @ AuctionCode::InvalidCreator,
        close = creator
    )]
    pub auction: Account<'info, Auction>,

    /// CHECK: we read this key only
    #[account(
        mut,
    )] 
    pub creator: UncheckedAccount<'info>,

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
        token::authority = claimer,
    )]
    pub claimer_nft_account: Account<'info, TokenAccount>,

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
    pub accepted_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimNft2022<'info> {
    #[account(
        mut,
        constraint = is_super_admin(authority.key) || authority.key() == claimer.key() @ AuctionCode::InvalidAuthority
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
        constraint = !auction.ended @ AuctionCode::AlreadyClaimed,
        constraint = Clock::get()?.unix_timestamp >= auction.end_time @ AuctionCode::AuctionNotEnded,
        constraint = auction.creator == creator.key() @ AuctionCode::InvalidCreator,
        close = creator
    )]
    pub auction: Account<'info, Auction>,

    /// CHECK: we read this key only
    #[account(
        mut,
    )] 
    pub creator: UncheckedAccount<'info>,

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
        token::authority = claimer,
    )]
    pub claimer_nft_account: Account<'info, TokenAccount>,

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
    pub accepted_mint: Box<InterfaceAccount<'info, Token2022Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimNftV2<'info> {
    #[account(
        mut,
        constraint = is_super_admin(authority.key) || authority.key() == claimer.key() @ AuctionCode::InvalidAuthority
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
        constraint = !auction.ended @ AuctionCode::AlreadyClaimed,
        constraint = Clock::get()?.unix_timestamp >= auction.end_time @ AuctionCode::AuctionNotEnded,
        constraint = auction.creator == creator.key() @ AuctionCode::InvalidCreator,
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

#[derive(Accounts)]
pub struct ClaimNftV22022<'info> {
    #[account(
        mut,
        constraint = is_super_admin(authority.key) || authority.key() == claimer.key() @ AuctionCode::InvalidAuthority
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
        constraint = !auction.ended @ AuctionCode::AlreadyClaimed,
        constraint = Clock::get()?.unix_timestamp >= auction.end_time @ AuctionCode::AuctionNotEnded,
        constraint = auction.creator == creator.key() @ AuctionCode::InvalidCreator,
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

#[derive(Accounts)]
pub struct CreateAuction<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
      seeds = [b"creator", creator.key().as_ref()],
      bump,
      constraint = creator_account.is_creator_available(creator.key())? @ AuctionCode::InvalidAuthority
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

#[derive(Accounts)]
pub struct CreateAuctionV2<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
      seeds = [b"creator", creator.key().as_ref()],
      bump,
      constraint = creator_account.is_creator_available(creator.key())? @ AuctionCode::InvalidAuthority
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

#[derive(Accounts)]
pub struct InitAuctionVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: we don't read this account
    pub auction: UncheckedAccount<'info>,

    pub mint: Account<'info, Mint>,

    #[account(mut)]
    /// CHECK: we don't read this account
    pub vault: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct InitAuctionVault2022<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: we don't read this account
    pub auction: UncheckedAccount<'info>,

    pub mint: Box<InterfaceAccount<'info, Token2022Mint>>,

    #[account(mut)]
    /// CHECK: we don't read this account
    pub vault: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

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
        constraint = previous_bidder.key() == auction.current_winner.unwrap_or(bidder.key()) @ AuctionCode::InvalidPreviousBidder
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

#[derive(Accounts)]
pub struct PlaceBid2022<'info> {
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

#[derive(Accounts)]
pub struct CancelAuction<'info> {
    #[account(
        mut,
        constraint = creator.key() == auction.creator @ AuctionCode::UnauthorizedCanceller
    )]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"auction", auction.nft_mint.as_ref()],
        bump,
        constraint = !auction.ended @ AuctionCode::AlreadyClaimed,
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

#[derive(Accounts)]
pub struct CancelAuction2022<'info> {
    #[account(
        mut,
        constraint = creator.key() == auction.creator @ AuctionCode::UnauthorizedCanceller
    )]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"auction", auction.nft_mint.as_ref()],
        bump,
        constraint = !auction.ended @ AuctionCode::AlreadyClaimed,
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
    pub vault_token_account: Box<InterfaceAccount<'info, Token2022TokenAccount>>,

    pub accepted_mint: Box<InterfaceAccount<'info, Token2022Mint>>,

    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct CancelAuctionV2<'info> {
    #[account(
        mut,
        constraint = creator.key() == auction.creator @ AuctionCode::UnauthorizedCanceller
    )]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"auction", auction.nft_mint.as_ref()],
        bump,
        constraint = !auction.ended @ AuctionCode::AlreadyClaimed,
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
    pub vault_token_account: Option<Account<'info, TokenAccount>>,

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

#[derive(Accounts)]
pub struct CancelAuctionV22022<'info> {
    #[account(
        mut,
        constraint = creator.key() == auction.creator @ AuctionCode::UnauthorizedCanceller
    )]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"auction", auction.nft_mint.as_ref()],
        bump,
        constraint = !auction.ended @ AuctionCode::AlreadyClaimed,
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

#[derive(Accounts)]
pub struct AddCreator<'info> {
  #[account(mut,
    constraint = is_super_admin(authority.key) @ AuctionCode::InvalidAuthority
  )]
  pub authority: Signer<'info>,

  #[account(
    init,
    seeds = [b"creator", creator_wallet.key().as_ref()],
    bump,
    payer = authority,
    space = std::mem::size_of::<Creator>() + 8,
  )]
  pub creator: Box<Account<'info, Creator>>,

  /// CHECK: Not dangerous because only admin can send tx
  pub creator_wallet: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveCreator<'info> {
  #[account(mut,
    constraint = is_super_admin(authority.key) @ AuctionCode::InvalidAuthority
  )]
  pub authority: Signer<'info>,

  #[account(
    mut,
    seeds = [b"creator", creator_wallet.key().as_ref()],
    bump,
    close = authority
  )]
  pub creator: Box<Account<'info, Creator>>,

  /// CHECK: Not dangerous because only admin can send tx
  pub creator_wallet: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>,
}