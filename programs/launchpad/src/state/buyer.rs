use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct Buyer {
    pub participate: bool,
}
