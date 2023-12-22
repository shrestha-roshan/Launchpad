use anchor_lang::prelude::*;

use crate::state::{whitelist::Whitelist, Auction};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WhitelistParams {
    pub whitelisted: bool,
    pub limit: u64,
}

#[derive(Accounts)]
pub struct WhitelistUser<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init_if_needed,   
        payer = owner,
        space = 8 + std::mem::size_of::<Whitelist>(),
        seeds = [b"whitelist", whitelist_user.key().as_ref(), auction.key().as_ref()],
        bump
    )]
    pub whitelist_pda: Box<Account<'info, Whitelist>>,
    #[account(
        mut,
        seeds = [b"auction",auction.name.as_bytes()],
        bump
    )]
    pub auction: Box<Account<'info, Auction>>,
    pub whitelist_user: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WhitelistUser>, params: WhitelistParams) -> Result<()> {
    let whitelist = &mut ctx.accounts.whitelist_pda;
    whitelist.whitelisted = params.whitelisted;
    whitelist.limit = params.limit;
    Ok(())
}