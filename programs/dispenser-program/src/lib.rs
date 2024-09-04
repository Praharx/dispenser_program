use anchor_lang::prelude::*;

declare_id!("JE3gJhzpJKpWAHXzMmNSEpRwR5PrfYeVFfoESCkt7GVU");

use anchor_lang::solana_program::hash::hash;
use anchor_lang::solana_program::system_instruction;


#[program]
pub mod dispenser_program {
    use super::*;

    pub fn initialize_escrow(
        ctx: Context<InitializeEscrow>,
        escrow_id: u64,
        winners: Vec<Pubkey>, // Accept winners as public keys
        prizes: Vec<u64>
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        // Ensure the number of prizes matches the number of winners
        require!(
            winners.len() == prizes.len(),
            ErrorCode::MismatchedPrizesAndWinners
        );

        // Ensure the number of winners does not exceed the maximum limit
        let max_winners = 5; // Adjust this limit as needed
        require!(
            winners.len() <= max_winners,
            ErrorCode::TooManyWinners
        );

        // Convert winners' public keys to hashes
        let hashed_winners: Vec<[u8; 32]> = winners
            .iter()
            .map(|winner_pubkey| {
                let winner_hash = hash(winner_pubkey.as_ref());
                winner_hash.to_bytes().try_into().unwrap()
            })
            .collect();

        // Set escrow details
        escrow.host = *ctx.accounts.host.key;
        escrow.hashed_winners = hashed_winners;
        escrow.prizes = prizes.clone();
        escrow.total_amount = prizes.iter().sum(); 
        escrow.escrow_vault = *ctx.accounts.escrow_vault.key;
        escrow.escrow_id = escrow_id;
        
        // Transfer SOL from host to escrow PDA account
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.host.key(),
            &ctx.accounts.escrow_vault.key(),  
            escrow.total_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.host.to_account_info(),
                ctx.accounts.escrow_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info()   
            ],
        )?;

        Ok(())
    }

    pub fn withdraw_prize(ctx: Context<WithdrawPrize>,escrow_id: u64,  winner_pubkey: Pubkey ) -> Result<()> {
        // Extract the necessary immutable data first
        let hashed_winners = ctx.accounts.escrow.hashed_winners.clone(); // Clone required data before mutable borrow
        let prize_list = ctx.accounts.escrow.prizes.clone(); // Clone the prize list

        // Verify the hashed winner's pubkey against stored hashes
        let winner_hash = hash(winner_pubkey.as_ref());

        // Find the index of the winner
        let winner_index = hashed_winners
            .iter()
            .position(|&h| h == winner_hash.to_bytes())
            .ok_or(ErrorCode::Unauthorized)?;

        // Get the prize amount for the winner
        let prize_amount = prize_list[winner_index];

        // Check if the prize has already been withdrawn (assuming we mark withdrawn with 0)
        require!(prize_amount > 0, ErrorCode::PrizeAlreadyClaimed);

        // Transfer prize amount to the winner
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.escrow_vault.key(),
            &ctx.accounts.winner.key(),
            prize_amount,
        );
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_ix,
            &[
                ctx.accounts.escrow_vault.to_account_info(),
                ctx.accounts.winner.to_account_info(),
            ],
            &[&[
                b"escrow_vault", 
                ctx.accounts.escrow.host.as_ref(), 
                &escrow_id.to_le_bytes(), // Pass the escrow_id here
                &[ctx.bumps.escrow_vault]
            ]], // Seeds and bump for the escrow PDA
        )?;

        // Now mutably borrow the escrow to update the prize status
        let escrow = &mut ctx.accounts.escrow;
        escrow.prizes[winner_index] = 0; // Mark the prize as claimed by setting it to 0

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(escrow_id:u64)]
pub struct InitializeEscrow<'info> {
    #[account(mut)]
    pub host: Signer<'info>,
    #[account(
        init,
        seeds = [b"escrow", host.key().as_ref(), &escrow_id.to_le_bytes().as_ref()],  // PDA derived using a seed and host's public key
        bump, // Auto-generated bump
        payer = host,
        space = 8 + 32 + (32 * 10) + (8 * 10) + 8 // Fixed maximum space for up to 10 winners
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        mut,
        seeds= [b"escrow_vault", host.key().as_ref(),  &escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(escrow_id:u64)]
pub struct WithdrawPrize<'info> { 
    #[account(
        mut,
        seeds = [b"escrow", escrow.host.key().as_ref(), &escrow_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>, 
    #[account(
        mut,
        seeds = [b"escrow_vault",escrow.host.key().as_ref(),  &escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_vault: SystemAccount<'info>,
    #[account(mut)]
    pub winner: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[account]
pub struct Escrow {
    pub host: Pubkey,
    pub escrow_vault: Pubkey,
    pub hashed_winners: Vec<[u8; 32]>, // Store SHA-256 hashes of the winners' pubkeys
    pub prizes: Vec<u64>, // Store the prize amounts corresponding to each winner
    pub total_amount: u64, // Total prize amount in SOL
    pub escrow_id: u64
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized: Caller is not one of the selected winners.")]
    Unauthorized,
    #[msg("Mismatched number of prizes and winners.")]
    MismatchedPrizesAndWinners,
    #[msg("The prize has already been claimed.")]
    PrizeAlreadyClaimed,
    #[msg("Too many winners exceeding the maximum limit.")]
    TooManyWinners,
}