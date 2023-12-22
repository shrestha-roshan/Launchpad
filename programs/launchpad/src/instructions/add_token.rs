use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

use crate::{error::LaunchpadError, state::auction::Auction};

#[derive(Accounts)]
pub struct AddToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"auction", auction.name.as_bytes()],
        bump
    )]
    pub auction: Box<Account<'info, Auction>>,
    #[account(
        mut,
        constraint = owner_auction_token_account.owner == owner.key(),
        constraint = owner_auction_token_account.mint == auction_token.key()
    )]
    pub owner_auction_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = auction_token,
        associated_token::authority = auction,
    )]
    pub auction_vault_token_account: Box<Account<'info, TokenAccount>>,
    pub auction_token: Account<'info, Mint>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<AddToken>) -> Result<()> {
    let owner = &ctx.accounts.owner;
    let from = &mut ctx.accounts.owner_auction_token_account;
    let to = &mut ctx.accounts.auction_vault_token_account;
    let auction = &mut ctx.accounts.auction;
    let token_program = ctx.accounts.token_program.to_account_info();

    //Ensure that the auction is not enabled
    if auction.enabled {
        return Err(LaunchpadError::InvalidAuction.into());
    }

    let transfer = Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: owner.to_account_info(),
    };

    let ctx: CpiContext<'_, '_, '_, '_, _> = CpiContext::new(token_program, transfer);
    anchor_spl::token::transfer(ctx, auction.token_cap)?;
    Ok(())
}
