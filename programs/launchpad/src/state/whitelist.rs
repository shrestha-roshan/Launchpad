use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct Whitelist {
    pub whitelisted: bool,
}
