//! Launchpad program entrypoint

#![allow(clippy::result_large_err)]
use anchor_lang::prelude::*;
mod instructions;
mod state;

use {anchor_lang::prelude::*, instructions::*};

declare_id!("LPD1BCWvd499Rk7aG5zG8uieUTTqba1JaYkUpXjUN9q");

#[program]
pub mod launchpad {
    use super::*;

    pub fn init_auction(ctx: Context<InitAuction>) -> Result<()> {
        deposit_sol::handler(ctx, amount)
    }
}
