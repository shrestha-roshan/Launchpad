//! Launchpad program entrypoint

// #![allow(clippy::result_large_err)]
use anchor_lang::prelude::*;
mod error;
mod instructions;
mod state;

use instructions::*;

declare_id!("EZ5JUytewouCHUDxrvakcjUp5MDuuEMbLEpQHFabgewa");

#[program]
mod launchpad {

    use super::*;

    pub fn init_auction(ctx: Context<InitAuction>, params: InitAuctionParams) -> Result<()> {
        init_auction::handler(ctx, params)
    }

    pub fn add_token(ctx: Context<AddToken>, params: AddTokenParams) -> Result<()> {
        add_token::handler(ctx, params)
    }

    pub fn buy_token_using_spl(ctx: Context<BuyTokensSpl>, spl_amount: u64) -> Result<()> {
        buy_token_using_spl::handler(ctx, spl_amount)
    }

    /** Under Development **/
    // pub fn buy_token_using_sol(ctx: Context<BuyTokensSol>, sol_amount: u64) -> Result<()> {
    //     buy_token_using_sol::handler(ctx, sol_amount)
    // }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
        withdraw_funds::handler(ctx)
    }
}
