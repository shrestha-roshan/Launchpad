//! Launchpad program entrypoint

// #![allow(clippy::result_large_err)]
use anchor_lang::prelude::*;
mod error;
mod instructions;
mod state;

use instructions::*;

declare_id!("DzdeoE6iqKdGvznDQB8xoT8d9wKriqF2zyuYrLVuWfG2");

#[program]
mod launchpad {

    use super::*;

    pub fn init_auction(ctx: Context<InitAuction>, params: InitAuctionParams) -> Result<()> {
        init_auction::handler(ctx, &params)
    }

    pub fn add_token(ctx: Context<AddToken>, params: AddTokenParams) -> Result<()> {
        add_token::handler(ctx, &params)
    }

    pub fn buy_token_using_usdt(ctx: Context<BuyTokensUsdt>, usdc_amount: u64) -> Result<()> {
        buy_token_using_usdt::handler(ctx, usdc_amount)
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
        withdraw_funds::handler(ctx)
    }
}
