use crate::state::Buyer;
use crate::{error::LaunchpadError, state::Auction};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_lang::system_program::{transfer as transfer_sol, Transfer as Transfer_Sol};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer as transfer_spl, Mint, Token, TokenAccount, Transfer as Transfer_Spl},
};

#[derive(Accounts)]
pub struct BuyTokensSol<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
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
        init_if_needed,
        space = 8 + std::mem::size_of::<Buyer>(),
        payer = buyer,
        seeds = [b"buyer", buyer.key().as_ref(), auction.key().as_ref()],
        bump,
    )]
    pub buyer_pda: Box<Account<'info, Buyer>>,
    #[account(
        mut,
        constraint = auction_vault_token_account.owner == auction_vault.key(),
        constraint = auction_vault_token_account.mint == auction_token.key()
    )]
    pub auction_vault_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = buyer_auction_token_account.owner == buyer.key(),
        constraint = buyer_auction_token_account.mint == auction_token.key()
    )]
    pub buyer_auction_token_account: Box<Account<'info, TokenAccount>>,
    pub auction_token: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyTokensSol>) -> Result<()> {
    let auction: &mut Box<Account<'_, Auction>> = &mut ctx.accounts.auction;
    let auction_vault: &AccountInfo<'_> = &ctx.accounts.auction_vault;
    let buyer = &ctx.accounts.buyer;
    let auction_vault_token_account = &ctx.accounts.auction_vault_token_account;
    let token_program = ctx.accounts.token_program.as_ref();
    let buyer_auction_token_account = &ctx.accounts.buyer_auction_token_account;
    let system_program = ctx.accounts.system_program.as_ref();
    let buyer_pda = &mut ctx.accounts.buyer_pda;

    // ticket_price (in SOL) calc: funding_demand / no.of tickets
    let ticket_price = (auction.funding_demand * LAMPORTS_PER_SOL) / (auction.tokens_in_pool/auction.token_quantity_per_ticket);
    
    // Ensure that the buyer has not participated in the auction
    // This is to restrict same user to participate in same auction multiple times
    if buyer_pda.participate {
        return Err(LaunchpadError::AlreadyParticipated.into());
    }

    // Ensure that the auction is enabled for sol payments
    if !auction.pay_with_native {
        return Err(LaunchpadError::NonNativeAuction.into());
    }

    // Ensure if the pre sale has ended
    if auction.pre_sale && ctx.accounts.clock.unix_timestamp < auction.pre_sale_end_time {
        return Err(LaunchpadError::PreSaleNotEnded.into());
    }

    // Ensure that the auction is enabled and live
    if !(auction.enabled
        && (ctx.accounts.clock.unix_timestamp > auction.start_time
            && ctx.accounts.clock.unix_timestamp < auction.end_time))
    {
        return Err(LaunchpadError::InvalidAuction.into());
    }

    // amount of tokens to send to buyer
    let auction_token_amount_to_buy = auction.token_quantity_per_ticket * LAMPORTS_PER_SOL;

    // Ensure there are enough tokens remaining for the buyer
    let remaining_tokens_in_auction_pool = auction.remaining_tokens * LAMPORTS_PER_SOL;
    if remaining_tokens_in_auction_pool < auction_token_amount_to_buy {
        return Err(LaunchpadError::InsufficientTokens.into());
    }

    // Transfer sol from buyer to auction
    let trns_sol = Transfer_Sol {
        from: buyer.to_account_info(),
        to: auction_vault.to_account_info(),
    };

    let ctx_sol: CpiContext<'_, '_, '_, '_, _> =
        CpiContext::new(system_program.to_account_info(), trns_sol);
    transfer_sol(ctx_sol, ticket_price)?;

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
    let trns_spl = Transfer_Spl {
        from: auction_vault_token_account.to_account_info(),
        to: buyer_auction_token_account.to_account_info(),
        authority: auction_vault.to_account_info(),
    };

    let ctx_spl: CpiContext<'_, '_, '_, '_, _> = CpiContext::new_with_signer(
        token_program.to_account_info(),
        trns_spl,
        auction_vault_seed,
    );
    transfer_spl(ctx_spl, auction_token_amount_to_buy)?;

    // Update the remaining tokens in the auction
    auction.remaining_tokens -= auction_token_amount_to_buy/LAMPORTS_PER_SOL;

    // Update the buyer account
    buyer_pda.participate = true;

    Ok(())
}
