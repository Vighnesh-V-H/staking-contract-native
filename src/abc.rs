// =============================================================================
// TARGET PROGRAM (The program being called)
// =============================================================================

use anchor_lang::prelude::*;

// Declare the program ID for the target program
declare_id!("TargetProgram1111111111111111111111111111111");

#[program]
pub mod target_program {
    use super::*;

    // This function will be called by another program via CPI
    pub fn update_counter(
        ctx: Context<UpdateCounter>, 
        increment_value: u64
    ) -> Result<()> {
        // Get a mutable reference to our counter account
        let counter = &mut ctx.accounts.counter;
        
        // Update the counter value
        counter.value += increment_value;
        counter.authority = ctx.accounts.authority.key();
        
        // Emit an event for logging
        emit!(CounterUpdated {
            old_value: counter.value - increment_value,
            new_value: counter.value,
            authority: ctx.accounts.authority.key(),
        });
        
        Ok(())
    }

    // Initialize a new counter account
    pub fn initialize_counter(ctx: Context<InitializeCounter>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.value = 0;
        counter.authority = ctx.accounts.authority.key();
        counter.bump = *ctx.bumps.get("counter").unwrap();
        Ok(())
    }
}

// Account validation struct for update_counter instruction
#[derive(Accounts)]
pub struct UpdateCounter<'info> {
    // Counter account that will be modified (must be mutable)
    #[account(
        mut,
        seeds = [b"counter", authority.key().as_ref()],
        bump = counter.bump,
    )]
    pub counter: Account<'info, Counter>,
    
    // The authority that can update the counter (must sign)
    pub authority: Signer<'info>,
}

// Account validation struct for initialize_counter instruction
#[derive(Accounts)]
pub struct InitializeCounter<'info> {
    // Counter account being created
    #[account(
        init,
        payer = authority,
        space = Counter::LEN,
        seeds = [b"counter", authority.key().as_ref()],
        bump,
    )]
    pub counter: Account<'info, Counter>,
    
    // Authority creating the counter (pays for account creation)
    #[account(mut)]
    pub authority: Signer<'info>,
    
    // System program needed for account creation
    pub system_program: Program<'info, System>,
}

// Counter account data structure
#[account]
pub struct Counter {
    pub value: u64,        // 8 bytes
    pub authority: Pubkey, // 32 bytes
    pub bump: u8,          // 1 byte
}

impl Counter {
    // Calculate space needed for the account
    pub const LEN: usize = 8 + 8 + 32 + 1; // discriminator + value + authority + bump
}

// Event emitted when counter is updated
#[event]
pub struct CounterUpdated {
    pub old_value: u64,
    pub new_value: u64,
    pub authority: Pubkey,
}

// =============================================================================
// CALLER PROGRAM (The program making CPI calls)
// =============================================================================

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction,
    program::invoke,
};

// Declare the program ID for the caller program
declare_id!("CallerProgram1111111111111111111111111111111");

#[program]
pub mod caller_program {
    use super::*;

    // Function that calls the target program
    pub fn call_target_program(
        ctx: Context<CallTargetProgram>,
        increment_value: u64,
    ) -> Result<()> {
        // Method 1: Using Anchor's CPI helper (recommended)
        call_target_with_anchor_cpi(&ctx, increment_value)?;
        
        // Method 2: Using manual instruction construction
        // call_target_with_manual_instruction(&ctx, increment_value)?;
        
        Ok(())
    }

    // Function that demonstrates calling with PDA authority
    pub fn call_target_with_pda(
        ctx: Context<CallTargetWithPda>,
        increment_value: u64,
    ) -> Result<()> {
        // Prepare the seeds for PDA signing
        let authority_key = ctx.accounts.user_authority.key();
        let seeds = &[
            b"pda_authority",
            authority_key.as_ref(),
            &[ctx.accounts.pda_authority.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // Create the accounts struct for the target program
        let cpi_accounts = target_program::cpi::accounts::UpdateCounter {
            counter: ctx.accounts.counter.to_account_info(),
            authority: ctx.accounts.pda_authority.to_account_info(),
        };

        // Create the CPI context with signer (PDA)
        let cpi_program = ctx.accounts.target_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program,
            cpi_accounts,
            signer_seeds,
        );

        // Make the CPI call
        target_program::cpi::update_counter(cpi_ctx, increment_value)?;

        // Update our caller program's state
        let caller_state = &mut ctx.accounts.caller_state;
        caller_state.total_calls += 1;
        caller_state.last_increment = increment_value;

        Ok(())
    }

    // Initialize caller program state
    pub fn initialize_caller_state(ctx: Context<InitializeCallerState>) -> Result<()> {
        let caller_state = &mut ctx.accounts.caller_state;
        caller_state.authority = ctx.accounts.authority.key();
        caller_state.total_calls = 0;
        caller_state.last_increment = 0;
        caller_state.bump = *ctx.bumps.get("caller_state").unwrap();
        Ok(())
    }

    // Initialize PDA authority
    pub fn initialize_pda_authority(ctx: Context<InitializePdaAuthority>) -> Result<()> {
        let pda_authority = &mut ctx.accounts.pda_authority;
        pda_authority.user_authority = ctx.accounts.user_authority.key();
        pda_authority.bump = *ctx.bumps.get("pda_authority").unwrap();
        Ok(())
    }
}

// Helper function using Anchor CPI
fn call_target_with_anchor_cpi(
    ctx: &Context<CallTargetProgram>,
    increment_value: u64,
) -> Result<()> {
    // Create the accounts struct for the target program's instruction
    let cpi_accounts = target_program::cpi::accounts::UpdateCounter {
        counter: ctx.accounts.counter.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    // Get the target program account info
    let cpi_program = ctx.accounts.target_program.to_account_info();

    // Create the CPI context
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Make the actual CPI call
    target_program::cpi::update_counter(cpi_ctx, increment_value)
}

// Helper function using manual instruction construction
fn call_target_with_manual_instruction(
    ctx: &Context<CallTargetProgram>,
    increment_value: u64,
) -> Result<()> {
    // Manually construct the instruction data
    // First 8 bytes are the instruction discriminator
    let mut instruction_data = vec![0u8; 8];
    instruction_data.extend_from_slice(&increment_value.to_le_bytes());

    // Create the instruction
    let instruction = Instruction {
        program_id: ctx.accounts.target_program.key(),
        accounts: vec![
            anchor_lang::solana_program::instruction::AccountMeta::new(
                ctx.accounts.counter.key(),
                false,
            ),
            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                ctx.accounts.authority.key(),
                true,
            ),
        ],
        data: instruction_data,
    };

    // Invoke the instruction
    invoke(
        &instruction,
        &[
            ctx.accounts.counter.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
    )
}

// Account validation for basic CPI call
#[derive(Accounts)]
pub struct CallTargetProgram<'info> {
    // The counter account in the target program
    #[account(
        mut,
        seeds = [b"counter", authority.key().as_ref()],
        bump,
        seeds::program = target_program.key(),
    )]
    pub counter: Account<'info, target_program::Counter>,

    // Authority that can update the counter
    pub authority: Signer<'info>,

    // The target program we're calling
    /// CHECK: This is the target program ID
    pub target_program: AccountInfo<'info>,
}

// Account validation for PDA-signed CPI call
#[derive(Accounts)]
pub struct CallTargetWithPda<'info> {
    // The counter account in the target program (owned by PDA)
    #[account(
        mut,
        seeds = [b"counter", pda_authority.key().as_ref()],
        bump,
        seeds::program = target_program.key(),
    )]
    pub counter: Account<'info, target_program::Counter>,

    // PDA that acts as authority for the target program
    #[account(
        seeds = [b"pda_authority", user_authority.key().as_ref()],
        bump = pda_authority.bump,
    )]
    pub pda_authority: Account<'info, PdaAuthority>,

    // The actual user authority
    pub user_authority: Signer<'info>,

    // Caller program's state
    #[account(
        mut,
        seeds = [b"caller_state", user_authority.key().as_ref()],
        bump = caller_state.bump,
    )]
    pub caller_state: Account<'info, CallerState>,

    // The target program we're calling
    /// CHECK: This is the target program ID
    pub target_program: AccountInfo<'info>,
}

// Account validation for initializing caller state
#[derive(Accounts)]
pub struct InitializeCallerState<'info> {
    #[account(
        init,
        payer = authority,
        space = CallerState::LEN,
        seeds = [b"caller_state", authority.key().as_ref()],
        bump,
    )]
    pub caller_state: Account<'info, CallerState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// Account validation for initializing PDA authority
#[derive(Accounts)]
pub struct InitializePdaAuthority<'info> {
    #[account(
        init,
        payer = user_authority,
        space = PdaAuthority::LEN,
        seeds = [b"pda_authority", user_authority.key().as_ref()],
        bump,
    )]
    pub pda_authority: Account<'info, PdaAuthority>,

    #[account(mut)]
    pub user_authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// Caller program's state account
#[account]
pub struct CallerState {
    pub authority: Pubkey,
    pub total_calls: u64,
    pub last_increment: u64,
    pub bump: u8,
}

impl CallerState {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 1;
}

// PDA authority account
#[account]
pub struct PdaAuthority {
    pub user_authority: Pubkey,
    pub bump: u8,
}

impl PdaAuthority {
    pub const LEN: usize = 8 + 32 + 1;
}