use anchor_lang::prelude::*;

use crate::{error::LaunchpadError, state::Auction};
use anchor_spl::token::{Mint, TokenAccount, Transfer};

pub fn handler(ctx: Context<WithdrawFunds>) -> Result<()> {
    let auction = &mut ctx.accounts.auction.clone();
    let creator = ctx.accounts.creator.clone();

    // Ensure that the withdrawal is done by the auction creator
    if *creator.key != auction.owner {
        return Err(LaunchpadError::Unauthorized.into());
    }

    // Ensure that the auction has ended
    if ctx.accounts.clock.unix_timestamp < auction.end_time {
        return Err(LaunchpadError::AuctionNotEnded.into());
    }

    // Ensure that the auction has not been disabled
    if !auction.enabled {
        return Err(LaunchpadError::AuctionDisabled.into());
    }

    // Transfer if there are any remaining tokens
    if (auction.token_cap - auction.remaining_tokens) > 0 {
        let transfer = Transfer {
            from: ctx.accounts.auction_vault_token_account.to_account_info(),
            to: ctx.accounts.creator_auction_token_account.to_account_info(),
            authority: ctx.accounts.auction.to_account_info(),
        };
        let ctx: CpiContext<'_, '_, '_, '_, _> =
            CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer);
        anchor_spl::token::transfer(ctx, auction.remaining_tokens)?;
    }

    // Transfer spl if tokens have been sold
    if auction.token_cap != auction.remaining_tokens && !auction.pay_with_native {
        let spl_amount = (auction.token_cap - auction.remaining_tokens) * auction.unit_price;
        let transfer = Transfer {
            from: ctx.accounts.auction_vault_bid_account.to_account_info(),
            to: ctx.accounts.creator_bid_token_account.to_account_info(),
            authority: ctx.accounts.auction.to_account_info(),
        };
        let ctx: CpiContext<'_, '_, '_, '_, _> =
            CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer);
        anchor_spl::token::transfer(ctx, spl_amount)?;
    }

    //Transfer sol if tokens have been sold
    if auction.token_cap != auction.remaining_tokens && auction.pay_with_native {
        let sol_amount = (auction.token_cap - auction.remaining_tokens) * auction.unit_price;
        let transfer_sol = Transfer {
            from: ctx.accounts.auction.to_account_info(),
            to: ctx.accounts.creator.to_account_info(),
            authority: ctx.accounts.auction.to_account_info(),
        };
        let ctx: CpiContext<'_, '_, '_, '_, _> =
            CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_sol);
        anchor_spl::token::transfer(ctx, sol_amount)?;
    }

    // Reset the auction state
    auction.owner = Pubkey::default();
    auction.enabled = false;
    auction.fixed_amount = false;
    auction.unit_price = 0;
    auction.start_time = 0;
    auction.end_time = 0;
    auction.token_cap = 0;
    auction.remaining_tokens = 0;
    auction.pay_with_native = false;
    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(signer)]
    /// CHECK:
    pub creator: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"auction", auction.name.as_bytes()],
        bump
    )]
    pub auction: Box<Account<'info, Auction>>,
    #[account(
        mut,
        constraint = auction_vault_token_account.owner == auction.key(),
        constraint = auction_vault_token_account.mint == auction_token.key()
    )]
    pub auction_vault_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = auction_vault_bid_account.owner == auction.key(),
        constraint = auction_vault_bid_account.mint == bid_token.key()
    )]
    pub auction_vault_bid_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = creator_auction_token_account.owner == creator.key(),
        constraint = creator_auction_token_account.mint == auction_token.key()
    )]
    pub creator_auction_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = creator_bid_token_account.owner == creator.key(),
        constraint = creator_bid_token_account.mint == bid_token.key()
    )]
    pub creator_bid_token_account: Box<Account<'info, TokenAccount>>,
    pub auction_token: Box<Account<'info, Mint>>,
    pub bid_token: Box<Account<'info, Mint>>,
    /// CHECK:
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}