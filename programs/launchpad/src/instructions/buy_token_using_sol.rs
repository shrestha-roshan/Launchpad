use anchor_lang::prelude::*;

use crate::{error::LaunchpadError, state::Auction};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct BuyTokensSol<'info> {
    #[account(signer)]
    /// CHECK:
    pub buyer: AccountInfo<'info>,
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
        constraint = buyer_auction_token_account.owner == buyer.key(),
        constraint = buyer_auction_token_account.mint == auction_token.key()
    )]
    pub buyer_auction_token_account: Box<Account<'info, TokenAccount>>,
    pub auction_token: Box<Account<'info, Mint>>,
    /// CHECK:
    pub token_program: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

pub fn _handler(ctx: Context<BuyTokensSol>, sol_amount: u64) -> Result<()> {
    let auction = &mut ctx.accounts.auction;
    let buyer = ctx.accounts.buyer.clone();
    let auction_vault_token_account = ctx.accounts.auction_vault_token_account.clone();
    let token_program = ctx.accounts.token_program.as_ref();
    let buyer_auction_token_account = ctx.accounts.buyer_auction_token_account.clone();

    // Ensure that the auction is enabled for spl payments
    if !auction.pay_with_native {
        return Err(LaunchpadError::NonNativeAuction.into());
    }

    // amount of token to send to buyer
    let token_amount_to_buy = sol_amount * auction.unit_price;

    // Ensure that the auction is initialized and live
    if !auction.enabled
        || ctx.accounts.clock.unix_timestamp > auction.start_time
        || ctx.accounts.clock.unix_timestamp < auction.end_time
    {
        return Err(LaunchpadError::InvalidAuction.into());
    }

    // Ensure there are enough tokens remaining for the buyer
    if auction.remaining_tokens < token_amount_to_buy {
        return Err(LaunchpadError::InsufficientTokens.into());
    }

    // Perform the token transfer to the buyer
    let transfer = Transfer {
        from: auction_vault_token_account.to_account_info(),
        to: buyer_auction_token_account.to_account_info(),
        authority: auction.to_account_info(),
    };

    let ctx: CpiContext<'_, '_, '_, '_, _> =
        CpiContext::new(token_program.to_account_info(), transfer);
    anchor_spl::token::transfer(ctx, token_amount_to_buy)?;

    // Transfer sol from buyer to auction
    let transfer_sol = Transfer {
        from: buyer.to_account_info(),
        to: auction.to_account_info(),
        authority: buyer.to_account_info(),
    };

    let ctx: CpiContext<'_, '_, '_, '_, _> =
        CpiContext::new(token_program.to_account_info(), transfer_sol);
    anchor_spl::token::transfer(ctx, sol_amount)?;

    // Update the remaining tokens in the auction
    auction.remaining_tokens -= token_amount_to_buy;

    Ok(())
}
