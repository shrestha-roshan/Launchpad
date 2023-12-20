use anchor_lang::prelude::*;

declare_id!("7xVm78pUZb9M8SNqtB2nMHFnkJXTL2HXnkQUJFBbv4ZP");

#[program]
pub mod lampbit_contracts {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
