use anchor_lang::prelude::*;

use crate::{error::LaunchpadError, state::Auction};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct BuyTokensUsdt<'info> {
    #[account(signer)]
    /// CHECK:
    pub buyer: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"auction", auction.owner.key().as_ref()],
        bump
    )]
    pub auction: Box<Account<'info, Auction>>,
    #[account(
        mut,
        constraint = auction_vault_token_account.owner == auction.key(),
        constraint = auction_vault_token_account.mint == mint.key()
    )]
    pub auction_vault_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub auction_vault_usdc_account: Box<Account<'info, TokenAccount>>,
    pub mint: Box<Account<'info, Mint>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    /// CHECK:
    pub token_program: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<BuyTokensUsdt>, usdc_amount: u64) -> Result<()> {
    let auction = &mut ctx.accounts.auction;
    let buyer = ctx.accounts.buyer.clone();
    let auction_vault_token_account = ctx.accounts.auction_vault_token_account.clone();
    let auction_vault_usdc_account = ctx.accounts.auction_vault_usdc_account.clone();
    let token_program = ctx.accounts.token_program.as_ref();

    // amount of token to send to buyer
    let token_amount_to_buy = usdc_amount * auction.unit_price;

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
        to: buyer.to_account_info(),
        authority: auction.to_account_info(),
    };

    let ctx: CpiContext<'_, '_, '_, '_, _> =
        CpiContext::new(token_program.to_account_info(), transfer);
    anchor_spl::token::transfer(ctx, token_amount_to_buy)?;

    // record the amount of tokens the buyer has purchased in auction

    // Transfer USDT from buyer to auction
    let transfer_usdt = Transfer {
        from: buyer.to_account_info(),
        to: auction_vault_usdc_account.to_account_info(),
        authority: buyer,
    };

    let ctx: CpiContext<'_, '_, '_, '_, _> =
        CpiContext::new(token_program.to_account_info(), transfer_usdt);
    anchor_spl::token::transfer(ctx, usdc_amount)?;

    // Update the remaining tokens in the auction
    auction.remaining_tokens -= token_amount_to_buy;

    Ok(())
}
