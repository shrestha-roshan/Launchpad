use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

use crate::state::auction::Auction;
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddTokenParams {
    pub token_amount: u64,
}

pub fn handler(ctx: Context<AddToken>, params: &AddTokenParams) -> Result<()> {
    let from = &mut ctx.accounts.owner;
    let to = &mut ctx.accounts.auction_vault_token_account;
    let token_program = ctx.accounts.token_program.to_account_info();

    let transfer = Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: from.to_account_info(),
    };

    let ctx: CpiContext<'_, '_, '_, '_, _> = CpiContext::new(token_program, transfer);
    anchor_spl::token::transfer(ctx, params.token_amount)?;
    Ok(())
}

#[derive(Accounts)]
pub struct AddToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub auction_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"auction", auction.owner.key().as_ref()],
        bump
    )]
    pub auction: Box<Account<'info, Auction>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = auction_token_mint,
        associated_token::authority = auction,
    )]
    pub auction_vault_token_account: Box<Account<'info, TokenAccount>>,
    pub auction_token_mint: Account<'info, Mint>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
