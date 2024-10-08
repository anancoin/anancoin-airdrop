pub mod context;
pub mod error;
pub mod state;

use anchor_lang::prelude::*;
use anchor_spl::token::{close_account, transfer};

use crate::context::*;
use crate::error::ErrorCode;

declare_id!("sEs461DSnSm3m9ssbWZ6j8WXrWJm96owZsAgwVSVKqB");

pub const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

#[program]
pub mod anancoin_airdrop {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        token_amount: u64, // airdrop token amount
        token_decimals: u8,
        token_limit: u64,
        bump: u8,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.bump = bump;
        config.token_mint = ctx.accounts.token_mint.key();
        config.token_account = ctx.accounts.token_account.key();
        config.token_pda = ctx.accounts.token_pda.key();
        config.token_decimals = token_decimals;
        if token_limit > 0 {
            config.token_limit = token_limit * 10u64.pow(token_decimals as u32);
        }

        if token_amount > 0 {
            let airdrop_token_amount = token_amount * 10u64.pow(token_decimals as u32);
            transfer(
                ctx.accounts.transfer_token_to_pda_cpicontext(),
                airdrop_token_amount.into(),
            )?;
        }

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, token_amount: u64) -> Result<()> {
        let config = &ctx.accounts.config;
        
        require!(
            token_amount > 0, 
            ErrorCode::ClaimTokenAmountIsZero
        );

        require!(
            token_amount <= config.token_limit,
            ErrorCode::ClaimTokenAmountOutOfLimit
        );

        let airdrop_token_amount =
            token_amount * 10u64.pow(config.token_decimals as u32);
        let pda_token_cur_amount = ctx.accounts.pda_account.amount;
        require!(
            pda_token_cur_amount >= airdrop_token_amount,
            ErrorCode::TokenAmountNotEnough
        );

        let pda_seeds = &[
            config.authority.as_ref(),
            config.token_mint.as_ref(),
            &[config.bump],
        ];

        transfer(
            ctx.accounts
                .transfer_token_to_user_cpicontext()
                .with_signer(&[pda_seeds.as_ref()]),
                airdrop_token_amount.into(),
        )?;

        Ok(())
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        let config = &ctx.accounts.config;

        let pda_seeds = &[
            config.authority.as_ref(),
            config.token_mint.as_ref(),
            &[config.bump],
        ];

        close_account(
            ctx.accounts
                .close_pda_token_account_cpicontext()
                .with_signer(&[pda_seeds.as_ref()]),
        )?;

        let amount = **ctx.accounts.config.to_account_info().try_borrow_mut_lamports()?;
        let config_account = &mut ctx.accounts.config;
        let authority_account = &mut ctx.accounts.authority;
        **config_account.to_account_info().try_borrow_mut_lamports()? -= amount;
        **authority_account.to_account_info().try_borrow_mut_lamports()? += amount;

        Ok(())
    }
}
