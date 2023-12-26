use crate::{
    error::LaunchpadError,
    state::{Auction, Buyer},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct BuyTokensSpl<'info> {
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
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyTokensSpl>, spl_amount: u64) -> Result<()> {
    let auction = &mut ctx.accounts.auction;
    let auction_vault: &AccountInfo<'_> = &ctx.accounts.auction_vault;
    let buyer = ctx.accounts.buyer.clone();
    let buyer_pda = &mut ctx.accounts.buyer_pda.clone();
    let auction_vault_token_account = ctx.accounts.auction_vault_token_account.clone();
    let auction_vault_spl_account = ctx.accounts.auction_vault_bid_token_account.clone();
    let buyer_spl_account = ctx.accounts.buyer_bid_token_account.clone();
    let token_program = ctx.accounts.token_program.as_ref();
    let buyer_auction_token_account = ctx.accounts.buyer_auction_token_account.clone();

    // ticket_price (in SOL) calc: funding_demand / no.of tickets
    let ticket_price = auction.funding_demand / (auction.tokens_in_pool/auction.token_quantity_per_ticket);

    // Ensure that the auction is enabled for spl payments
    if auction.pay_with_native {
        return Err(LaunchpadError::NonSplAuction.into());
    }

    // Ensure that the buyer has already participated in the auction
    if buyer_pda.participate {
        return Err(LaunchpadError::AlreadyParticipated.into());
    }

    // Check if the spl token is enough to buy at least one ticket_price
    if spl_amount != ticket_price {
        return Err(LaunchpadError::InvalidSolFor1ticket.into());
    }

    // Check if token amount to buy is greater than 0
    if spl_amount == 0 {
        return Err(LaunchpadError::InvalidTokenAmount.into());
    }

    // Ensure that the auction is initialized and live
    if !(auction.enabled
        && (ctx.accounts.clock.unix_timestamp > auction.start_time
            && ctx.accounts.clock.unix_timestamp < auction.end_time))
    {
        return Err(LaunchpadError::InvalidAuction.into());
    }

    // Ensure if the pre sale has been ended
    if auction.pre_sale && ctx.accounts.clock.unix_timestamp < auction.pre_sale_end_time {
        return Err(LaunchpadError::PreSaleNotEnded.into());
    }

    // amount of token to send to buyer
    // let auction_token_amount_to_buy = spl_amount / auction.unit_price;
    let auction_token_amount_to_buy = auction.token_quantity_per_ticket;

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

    let ctx: CpiContext<'_, '_, '_, '_, _> = CpiContext::new_with_signer(
        token_program.to_account_info(),
        transfer,
        auction_vault_seed,
    );
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

    // Update the remaining tokens in the auction
    auction.remaining_tokens -= auction_token_amount_to_buy;

    // Update the buyer account
    buyer_pda.participate = true;

    Ok(())
}
