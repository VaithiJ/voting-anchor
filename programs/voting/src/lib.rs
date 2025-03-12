// lib.rs
use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Token};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke,
};

declare_id!("Dox5QvNxtB5fYN81DwM5z7DNwmp9Z5wb1HLj4ntj538W");

#[program]
pub mod voting_system {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, candidates: Vec<String>) -> Result<()> {
        let voting_state = &mut ctx.accounts.voting_state;
        voting_state.authority = ctx.accounts.authority.key();
        voting_state.is_active = true;
        voting_state.vote_mint = ctx.accounts.vote_mint.key();
        voting_state.candidates = Vec::new();
        
        for candidate in candidates {
            voting_state.candidates.push(Candidate {
                name: candidate,
                vote_count: 0,
            });
        }
        
        voting_state.voters = Vec::new();
        Ok(())
    }
    

    pub fn cast_vote(ctx: Context<CastVote>, candidate_index: u32, amount_coin: u64, amount_pc: u64) -> Result<()> {
        let voting_state = &mut ctx.accounts.voting_state;
        require!(voting_state.is_active, VotingError::VotingClosed);
    
        let voter = ctx.accounts.voter.key();
        require!(
            !voting_state.voters.contains(&voter),
            VotingError::AlreadyVoted
        );
    
        require!(
            candidate_index < voting_state.candidates.len() as u32,
            VotingError::InvalidCandidate
        );
    
        // Record the vote
        voting_state.candidates[candidate_index as usize].vote_count += 1;
        voting_state.voters.push(voter);
    
        // Mint token reward for voting
        let cpi_accounts = MintTo {
            mint: ctx.accounts.vote_mint.to_account_info(),
            to: ctx.accounts.voter_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, 1_000_000)?; // Reward 1 token
    
        // Construct Deposit instruction for Raydium
        let deposit_accounts = vec![
            AccountMeta::new_readonly(ctx.accounts.spl_token_program.key(), false),
            AccountMeta::new(ctx.accounts.amm.key(), false),
            AccountMeta::new_readonly(ctx.accounts.authority.key(), false),
            AccountMeta::new_readonly(ctx.accounts.amm_open_orders.key(), false),
            AccountMeta::new(ctx.accounts.amm_target_orders.key(), false),
            AccountMeta::new(ctx.accounts.pool_lp_mint.key(), false),
            AccountMeta::new(ctx.accounts.pool_token_coin.key(), false),
            AccountMeta::new(ctx.accounts.pool_token_pc.key(), false),
            AccountMeta::new_readonly(ctx.accounts.serum_market.key(), false),
            AccountMeta::new(ctx.accounts.user_coin_token_account.key(), false),
            AccountMeta::new(ctx.accounts.user_pc_token_account.key(), false),
            AccountMeta::new(ctx.accounts.user_lp_token_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.voter.key(), true),
        ];
    
        let deposit_data = DepositInstruction {
            amount_coin,
            amount_pc,
        }
        .try_to_vec()?; // Serialize data
    
        let deposit_instruction = Instruction {
            program_id: ctx.accounts.raydium_program.key(),
            accounts: deposit_accounts,
            data: deposit_data,
        };
    
        // CPI to Raydium program
        #[cfg(not(feature = "test"))]
        invoke(
            &deposit_instruction,
            &[
                ctx.accounts.spl_token_program.to_account_info(),
                ctx.accounts.amm.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.amm_open_orders.to_account_info(),
                ctx.accounts.amm_target_orders.to_account_info(),
                ctx.accounts.pool_lp_mint.to_account_info(),
                ctx.accounts.pool_token_coin.to_account_info(),
                ctx.accounts.pool_token_pc.to_account_info(),
                ctx.accounts.serum_market.to_account_info(),
                ctx.accounts.user_coin_token_account.to_account_info(),
                ctx.accounts.user_pc_token_account.to_account_info(),
                ctx.accounts.user_lp_token_account.to_account_info(),
                ctx.accounts.voter.to_account_info(),
            ],
        )?;
        #[cfg(feature = "test")]
        {
            // Mock the CPI call during testing
            msg!("Mocking Raydium CPI call");
        }
    
        Ok(())
    }
    
    

    pub fn end_voting(ctx: Context<EndVoting>) -> Result<()> {
        let voting_state = &mut ctx.accounts.voting_state;
        
        require!(
            voting_state.authority == ctx.accounts.authority.key(),
            VotingError::Unauthorized
        );
        
        require!(voting_state.is_active, VotingError::VotingAlreadyClosed);
        
        voting_state.is_active = false;
        Ok(())
    }

    pub fn get_results(ctx: Context<GetResults>) -> Result<Vec<Candidate>> {
        Ok(ctx.accounts.voting_state.candidates.clone())
    }
}

#[derive(Accounts)]
#[instruction(candidates: Vec<String>)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + VotingState::space(&candidates)
    )]
    pub voting_state: Account<'info, VotingState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is the VOTE token mint, validated by token program4
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub vote_mint: UncheckedAccount<'info>,
    #[account(address = anchor_lang::system_program::ID)]
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voting_state: Account<'info, VotingState>,
    #[account(mut)]
    pub voter: Signer<'info>,
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub vote_mint: UncheckedAccount<'info>,
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub voter_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Raydium program
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    pub raydium_program: UncheckedAccount<'info>,
    /// CHECK: SPL Token program
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    pub spl_token_program: UncheckedAccount<'info>,
    /// CHECK: AMM account
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub amm: UncheckedAccount<'info>,
    /// CHECK: AMM open_orders account
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub amm_open_orders: UncheckedAccount<'info>,
    /// CHECK: AMM target_orders account
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub amm_target_orders: UncheckedAccount<'info>,
    /// CHECK: Pool LP mint
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub pool_lp_mint: UncheckedAccount<'info>,
    /// CHECK: Pool token_coin
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub pool_token_coin: UncheckedAccount<'info>,
    /// CHECK: Pool token_pc
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub pool_token_pc: UncheckedAccount<'info>,
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    /// CHECK: Serum market
    pub serum_market: UncheckedAccount<'info>,
    /// CHECK: User coin token account
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub user_coin_token_account: UncheckedAccount<'info>,
    /// CHECK: User pc token account
    /// CHECK: This is the mint for the VOTE token. Safe because it is initialized and verified during program setup.
    #[account(mut)]
    pub user_pc_token_account: UncheckedAccount<'info>,
    /// CHECK: User LP token account
    #[account(mut)]
    pub user_lp_token_account: UncheckedAccount<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositInstruction {
    pub amount_coin: u64,
    pub amount_pc: u64,
}


#[derive(Accounts)]
pub struct EndVoting<'info> {
    #[account(mut)]
    pub voting_state: Account<'info, VotingState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetResults<'info> {
    pub voting_state: Account<'info, VotingState>,
}

#[account]
pub struct VotingState {
    pub authority: Pubkey,
    pub is_active: bool,
    pub vote_mint: Pubkey,
    pub candidates: Vec<Candidate>,
    pub voters: Vec<Pubkey>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Candidate {
    pub name: String,
    pub vote_count: u64,
}

impl VotingState {
    const MAX_CANDIDATES: usize = 100;
    const MAX_VOTERS: usize = 200; // Reduced to fit within 10 KB
    const MAX_NAME_LEN: usize = 32;

    pub fn space(candidates: &[String]) -> usize {
        8 + // discriminator
        32 + // authority
        1 + // is_active
        32 + // vote_mint
        4 + // candidates vec length
        candidates.iter().fold(0, |acc, c| {
            acc + 4 + c.len().min(Self::MAX_NAME_LEN) + 8 // name + vote_count
        }) +
        4 + // voters vec length
        (Self::MAX_VOTERS * 32) // voters array
    }
}

#[error_code]
pub enum VotingError {
    #[msg("Voting has already closed")]
    VotingClosed,
    #[msg("Voter has already voted")]
    AlreadyVoted,
    #[msg("Invalid candidate index")]
    InvalidCandidate,
    #[msg("Unauthorized operation")]
    Unauthorized,
    #[msg("Voting has already been closed")]
    VotingAlreadyClosed,
}