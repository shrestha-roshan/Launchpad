use crate::{error::LaunchpadError, state::Auction};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_lang::system_program::{transfer as transfer_sol, Transfer as Tranfer_Sol};
use anchor_spl::token::{
    transfer as transfer_spl, Mint, Token, TokenAccount, Transfer as Transfer_Spl,
};

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
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
        constraint = creator_auction_token_account.owner == creator.key(),
        constraint = creator_auction_token_account.mint == auction_token.key()
    )]
    pub creator_auction_token_account: Box<Account<'info, TokenAccount>>,
    pub auction_token: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawFunds>) -> Result<()> {
    let auction = &mut ctx.accounts.auction.clone();
    let auction_vault: &AccountInfo<'_> = &ctx.accounts.auction_vault;
    let creator = ctx.accounts.creator.clone();

    // Ensure that the withdrawal is done by the auction creator
    if *creator.key != auction.owner {
        return Err(LaunchpadError::Unauthorized.into());
    }

    // Ensure that the auction has ended
    if ctx.accounts.clock.unix_timestamp < auction.end_time {
        return Err(LaunchpadError::AuctionNotEnded.into());
    }

    // Ensure that the auction has not been disabled
    if !auction.enabled {
        return Err(LaunchpadError::AuctionDisabled.into());
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

    // Transfer if there are any remaining tokens
    if auction.remaining_tokens > 0 {
        let trans_spl = Transfer_Spl {
            from: ctx.accounts.auction_vault_token_account.to_account_info(),
            to: ctx.accounts.creator_auction_token_account.to_account_info(),
            authority: auction_vault.to_account_info(),
        };
        let ctx: CpiContext<'_, '_, '_, '_, _> = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            trans_spl,
            auction_vault_seed,
        );
        transfer_spl(ctx, auction.remaining_tokens)?;
    }

    //Transfer sol if tokens have been sold
    if auction.token_cap != auction.remaining_tokens && auction.pay_with_native {
        let sol_amount = (auction.token_cap - auction.remaining_tokens)
            * (auction.unit_price / LAMPORTS_PER_SOL);
        let trans_sol = Tranfer_Sol {
            from: auction_vault.to_account_info(),
            to: ctx.accounts.creator.to_account_info(),
        };
        let ctx: CpiContext<'_, '_, '_, '_, _> = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            trans_sol,
            auction_vault_seed,
        );
        transfer_sol(ctx, sol_amount)?;
    }

    // Reset the auction state
    auction.owner = Pubkey::default();
    auction.enabled = false;
    auction.fixed_amount = false;
    auction.unit_price = 0;
    auction.start_time = 0;
    auction.end_time = 0;
    auction.token_cap = 0;
    auction.remaining_tokens = 0;
    auction.pay_with_native = false;
    auction.pre_sale = false;
    auction.pre_sale_start_time = 0;
    auction.pre_sale_end_time = 0;
    Ok(())
}
