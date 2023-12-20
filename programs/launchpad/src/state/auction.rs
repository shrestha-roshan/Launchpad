use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct Auction {
    pub owner: Pubkey,
    pub name: String,
    pub enabled: bool,
    pub fixed_amount: bool,
    pub unit_price: i64,
    pub token_cap: u64,
    // time of creation, also used as current wall clock time for testing
    pub start_time: i64,
    pub end_time: i64,
}
