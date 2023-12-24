use anchor_lang::prelude::*;
use crate::{state::{whitelist::Whitelist, Auction}, error::LaunchpadError};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WhitelistParams {
    pub whitelisted: bool,
    pub limit: u64,
}

#[derive(Accounts)]
pub struct WhitelistUser<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init_if_needed,   
        payer = creator,
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
    /// CHECK:
    pub whitelist_user: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WhitelistUser>, params: WhitelistParams) -> Result<()> {
    // Ensure that the creator is the owner of the auction
    if ctx.accounts.creator.key != &ctx.accounts.auction.owner {
        return Err(LaunchpadError::InvalidAuction.into());
    }
    let whitelist = &mut ctx.accounts.whitelist_pda;
    whitelist.whitelisted = params.whitelisted;
    whitelist.limit = params.limit;
    Ok(())
}