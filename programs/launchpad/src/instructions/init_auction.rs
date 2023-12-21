use anchor_lang::prelude::*;

use crate::state::auction::Auction;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitAuctionParams {
    pub name: String,
    pub enabled: bool,
    pub fixed_amount: bool,
    pub start_time: i64,
    pub end_time: i64,
    pub unit_price: u64,
    pub token_cap: u64,
    pub pay_with_native: bool,
}

pub fn handler(ctx: Context<InitAuction>, params: InitAuctionParams) -> Result<()> {
    let auction = &mut ctx.accounts.auction;
    auction.owner = *ctx.accounts.owner.key;
    auction.name = params.name;
    auction.enabled = params.enabled;
    auction.fixed_amount = params.fixed_amount;
    auction.unit_price = params.unit_price;
    auction.start_time = params.start_time;
    auction.end_time = params.end_time;
    auction.token_cap = params.token_cap;
    auction.remaining_tokens = params.token_cap;
    auction.pay_with_native = params.pay_with_native;
    Ok(())
}

#[derive(Accounts)]
#[instruction(params: InitAuctionParams)]
pub struct InitAuction<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init_if_needed,   
        payer = owner,
        space = 8 + std::mem::size_of::<Auction>(),
        seeds = [b"auction", params.name.as_bytes()],
        bump
    )]
    pub auction: Box<Account<'info, Auction>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
