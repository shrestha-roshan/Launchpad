use crate::{
    error::LaunchpadError,
    state::{Auction, Buyer, Whitelist},
};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer as transfer_sol, Transfer as Tranfer_Sol};
use anchor_spl::token::{
    transfer as transfer_spl, Mint, Token, TokenAccount, Transfer as Transfer_Spl,
};

#[derive(Accounts)]
pub struct PreSaleBuyUsingSol<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 8 + std::mem::size_of::<Buyer>(),
        payer = buyer,
        seeds = [b"buyer", buyer.key().as_ref(), auction.key().as_ref()],
        bump,
    )]
    pub buyer_pda: Box<Account<'info, Buyer>>,
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
    pub auction_token: Box<Account<'info, Mint>>,
    // pub bid_token: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"whitelist", buyer.key().as_ref(), auction.key().as_ref()],
        bump
    )]
    pub whitelist_pda: Box<Account<'info, Whitelist>>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<PreSaleBuyUsingSol>, sol_amount: u64) -> Result<()> {
    let whitelist = &mut ctx.accounts.whitelist_pda;
    let auction = &mut ctx.accounts.auction;
    let auction_vault: &AccountInfo<'_> = &ctx.accounts.auction_vault;
    let auction_vault_token_account = ctx.accounts.auction_vault_token_account.clone();
    let buyer_auction_token_account = ctx.accounts.buyer_auction_token_account.clone();
    let buyer = ctx.accounts.buyer.clone();
    let token_program = ctx.accounts.token_program.as_ref();
    let system_program = ctx.accounts.system_program.as_ref();
    let buyer_pda = &mut ctx.accounts.buyer_pda.clone();

    // Ensure that the buyer has already participated in the auction
    if buyer_pda.participate {
        return Err(LaunchpadError::AlreadyParticipated.into());
    }

    // Check if sol amount using for buying is greater than 0
    if sol_amount == 0 {
        return Err(LaunchpadError::InvalidSolAmount.into());
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
    let auction_token_amount_to_buy: u64 = sol_amount / auction.unit_price;

    // Check if the sol is enough to buy at least one ticket_price
    if sol_amount != auction.ticket_price {
        return Err(LaunchpadError::InvalidSolFor1ticket.into());
    }

    // Ensure that the auction is enabled for sol payments
    if !auction.pay_with_native {
        return Err(LaunchpadError::NonNativeAuction.into());
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
    let trans_spl = Transfer_Spl {
        from: auction_vault_token_account.to_account_info(),
        to: buyer_auction_token_account.to_account_info(),
        authority: auction_vault.to_account_info(),
    };

    let ctx: CpiContext<'_, '_, '_, '_, _> = CpiContext::new_with_signer(
        token_program.to_account_info(),
        trans_spl,
        auction_vault_seed,
    );
    transfer_spl(ctx, auction_token_amount_to_buy)?;

    // Transfer sol from buyer to auction vault
    let trans_sol = Tranfer_Sol {
        from: buyer.to_account_info(),
        to: auction_vault.to_account_info(),
    };
    let ctx_sol: CpiContext<'_, '_, '_, '_, _> =
        CpiContext::new(system_program.to_account_info(), trans_sol);
    transfer_sol(ctx_sol, sol_amount)?;

    // Update state
    auction.remaining_tokens -= auction_token_amount_to_buy;

    // Update buyer state
    buyer_pda.participate = true;
    Ok(())
}
