use anchor_lang::prelude::*;
use crate::{state::auction::Auction, error::LaunchpadError};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitAuctionParams {
    pub name: String,
    pub enabled: bool,
    pub fixed_amount: bool,
    pub start_time: i64,
    pub end_time: i64,
    pub pay_with_native: bool,
    pub pre_sale: bool,
    pub pre_sale_start_time: i64,
    pub pre_sale_end_time: i64,
    pub tokens_in_pool: u64,  // pool of total tokens
    pub token_quantity_per_ticket: u64,  // no. of tokens in one ticket
    pub funding_demand: u64, // in SOL (return on investment)
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
    #[account(
        init_if_needed,
        payer = owner,
        space = 0,
        seeds = [b"auction_vault", auction.key().as_ref()],
        bump,
    )]
    /// CHECK: seeds has been checked
    pub auction_vault: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitAuction>, params: InitAuctionParams) -> Result<()> {
    let auction = &mut ctx.accounts.auction;

    // Ensure auction end time is greater than auction start time
    if params.start_time >= params.end_time {
        return Err(LaunchpadError::InvalidAuctionTimes.into());
    }
    
    // Ensure pre-sale end time is greater than pre-sale start time
    if params.pre_sale_start_time >= params.pre_sale_end_time {
        return Err(LaunchpadError::InvalidPresaleTime.into());
    }

    // Ensure pre-sale time doesn't surpass auction time
    if params.pre_sale_end_time >= params.start_time {
        return Err(LaunchpadError::InvalidPresaleTime.into());
    }

    auction.owner = *ctx.accounts.owner.key;
    auction.name = params.name;
    auction.enabled = params.enabled;
    auction.fixed_amount = params.fixed_amount;
    auction.start_time = params.start_time;
    auction.end_time = params.end_time;
    auction.pay_with_native = params.pay_with_native;
    auction.pre_sale = params.pre_sale;
    auction.pre_sale_start_time = params.pre_sale_start_time;
    auction.pre_sale_end_time = params.pre_sale_end_time;
    auction.tokens_in_pool = params.tokens_in_pool;
    auction.remaining_tokens = params.tokens_in_pool;
    auction.token_quantity_per_ticket = params.token_quantity_per_ticket;
    auction.funding_demand = params.funding_demand;
    Ok(())
}
