use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct Auction {
    pub owner: Pubkey,
    pub name: String,
    pub enabled: bool,
    pub fixed_amount: bool,
    pub unit_price: u64,
    pub token_cap: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub remaining_tokens: u64,
    pub pay_with_native: bool,
    pub pre_sale: bool,
    pub pre_sale_start_time: i64,
    pub pre_sale_end_time: i64,
    pub ticket_price: u64,
}
