use crate::{
    error::LaunchpadError,
    state::{Auction, Whitelist},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct PreSaleBuy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        mut,
        constraint = buyer_bid_token_account.owner == buyer.key(),
        constraint = buyer_bid_token_account.mint == bid_token.key()
    )]
    pub buyer_bid_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = buyer_auction_token_account.owner == buyer.key(),
        constraint = buyer_auction_token_account.mint == auction_token.key()
    )]
    pub buyer_auction_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"auction", auction.name.as_bytes()],
        bump
    )]
    pub auction: Box<Account<'info, Auction>>,
    #[account(
        mut,
        seeds = [b"auction_vault", auction.key().as_ref()],
        bump,
    )]
    /// CHECK: seeds has been checked
    pub auction_vault: AccountInfo<'info>,
    #[account(
        mut,
        constraint = auction_vault_token_account.owner == auction_vault.key(),
        constraint = auction_vault_token_account.mint == auction_token.key()
    )]
    pub auction_vault_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = auction_vault_bid_token_account.owner == auction_vault.key(),
        constraint = auction_vault_bid_token_account.mint == bid_token.key()
    )]
    pub auction_vault_bid_token_account: Box<Account<'info, TokenAccount>>,
    pub auction_token: Box<Account<'info, Mint>>,
    pub bid_token: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"whitelist", buyer.key().as_ref(), auction.key().as_ref()],
        bump
    )]
    pub whitelist_pda: Box<Account<'info, Whitelist>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<PreSaleBuy>, spl_amount: u64) -> Result<()> {
    let whitelist = &mut ctx.accounts.whitelist_pda;
    let auction = &mut ctx.accounts.auction;
    let auction_vault: &AccountInfo<'_> = &ctx.accounts.auction_vault;
    let buyer = ctx.accounts.buyer.clone();
    let auction_vault_token_account = ctx.accounts.auction_vault_token_account.clone();
    let auction_vault_spl_account = ctx.accounts.auction_vault_bid_token_account.clone();
    let buyer_spl_account = ctx.accounts.buyer_bid_token_account.clone();
    let token_program = ctx.accounts.token_program.as_ref();
    let buyer_auction_token_account = ctx.accounts.buyer_auction_token_account.clone();

    // Check if token amount to buy is greater than 0
    if spl_amount == 0 {
        return Err(LaunchpadError::InvalidTokenAmount.into());
    }

    // Ensure if the auction presale is enabled
    if !auction.pre_sale {
        return Err(LaunchpadError::PreSaleNotEnabled.into());
    }

    // Ensure if the the buyer is whitelisted
    if !whitelist.whitelisted {
        return Err(LaunchpadError::NotWhitelisted.into());
    }

    // Ensure presale time is valid
    let current_ts = ctx.accounts.clock.unix_timestamp as i64;
    if current_ts < auction.pre_sale_start_time && current_ts > auction.pre_sale_end_time {
        return Err(LaunchpadError::InvalidPresaleTime.into());
    }

    // amount of token to send to buyer
    let auction_token_amount_to_buy = spl_amount * auction.unit_price;

    // Ensure if the buyer is within the limit
    if auction_token_amount_to_buy > whitelist.limit {
        return Err(LaunchpadError::ExceedsLimit.into());
    }

    // Ensure that the auction is enabled for spl payments
    if auction.pay_with_native {
        return Err(LaunchpadError::NonSplAuction.into());
    }

    // Ensure that the auction is initialized and live
    if !(auction.enabled && (current_ts > auction.start_time && current_ts < auction.end_time)) {
        return Err(LaunchpadError::InvalidAuction.into());
    }

    // Ensure there are enough tokens remaining for the buyer
    if auction.remaining_tokens < auction_token_amount_to_buy {
        return Err(LaunchpadError::InsufficientTokens.into());
    }

    // Generate auction seed
    let auction_key = auction.key();

    let (_, bump_seed) = Pubkey::find_program_address(
        &["auction_vault".as_bytes(), auction_key.as_ref()],
        ctx.program_id,
    );
    let auction_vault_seed: &[&[&[_]]] = &[&[
        "auction_vault".as_bytes(),
        auction_key.as_ref(),
        &[bump_seed],
    ]];

    // Perform the token transfer to the buyer
    let transfer = Transfer {
        from: auction_vault_token_account.to_account_info(),
        to: buyer_auction_token_account.to_account_info(),
        authority: auction_vault.to_account_info(),
    };

    let ctx: CpiContext<'_, '_, '_, '_, _> =
        CpiContext::new_with_signer(token_program.to_account_info(), transfer, auction_vault_seed);
    anchor_spl::token::transfer(ctx, auction_token_amount_to_buy)?;

    // Transfer spl from buyer to auction
    let transfer_spl = Transfer {
        from: buyer_spl_account.to_account_info(),
        to: auction_vault_spl_account.to_account_info(),
        authority: buyer.to_account_info(),
    };

    let ctx: CpiContext<'_, '_, '_, '_, _> =
        CpiContext::new(token_program.to_account_info(), transfer_spl);
    anchor_spl::token::transfer(ctx, spl_amount)?;

    // Update state
    auction.remaining_tokens -= auction_token_amount_to_buy;
    whitelist.limit -= auction_token_amount_to_buy;

    Ok(())
}
