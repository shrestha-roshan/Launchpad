//! Launchpad program entrypoint

// #![allow(clippy::result_large_err)]
use anchor_lang::prelude::*;
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
}
