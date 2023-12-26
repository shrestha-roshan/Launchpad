//! Launchpad program entrypoint

// #![allow(clippy::result_large_err)]
use anchor_lang::prelude::*;
mod error;
mod instructions;
mod state;

use instructions::*;

declare_id!("E1PSNCTQhcQYiHihtidJx6ArPrpAoCMM6378mJZquQED");

#[program]
mod launchpad {

    use super::*;

    pub fn init_auction(ctx: Context<InitAuction>, params: InitAuctionParams) -> Result<()> {
        init_auction::handler(ctx, params)
    }

    pub fn add_token(ctx: Context<AddToken>) -> Result<()> {
        add_token::handler(ctx)
    }

    pub fn buy_token_using_spl(ctx: Context<BuyTokensSpl>, spl_amount: u64) -> Result<()> {
        buy_token_using_spl::handler(ctx, spl_amount)
    }

    pub fn buy_token_using_sol(ctx: Context<BuyTokensSol>) -> Result<()> {
        buy_token_using_sol::handler(ctx)
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
        withdraw_funds::handler(ctx)
    }

    pub fn whitelist(ctx: Context<WhitelistUser>, params: WhitelistParams) -> Result<()> {
        whitelist::handler(ctx, params)
    }

    pub fn pre_sale_buy_using_spl(ctx: Context<PreSaleBuyUsingSpl>, spl_amount: u64) -> Result<()> {
        pre_sale_buy_using_spl::handler(ctx, spl_amount)
    }

    pub fn pre_sale_buy_using_sol(ctx: Context<PreSaleBuyUsingSol>) -> Result<()> {
        pre_sale_buy_using_sol::handler(ctx)
    }
}
