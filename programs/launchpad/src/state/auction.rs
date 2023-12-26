use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct Auction {
    pub owner: Pubkey,
    pub name: String,
    pub enabled: bool,
    pub fixed_amount: bool,
    pub start_time: i64,
    pub end_time: i64,
    pub pay_with_native: bool,
    pub pre_sale: bool,
    pub pre_sale_start_time: i64,
    pub pre_sale_end_time: i64,
    pub tokens_in_pool: u64,  // token_amount or total tokens allocated by auction owner
    pub remaining_tokens: u64,
    pub token_quantity_per_ticket: u64,  // no. of tokens in one ticket
    pub funding_demand: u64, // in SOL (return on investment)
}
