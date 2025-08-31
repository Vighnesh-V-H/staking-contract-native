use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
pub mod  instructions;
use instructions::*;

pub mod functions;
use functions::*;

pub mod accounts;
use accounts::*;

pub mod constants;
use constants::*;


entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
   
    let instruction = StakingInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    // Route to appropriate handler
    match instruction {
        StakingInstruction::InitializePool { reward_rate } => {
            initialize_pool(program_id, accounts, reward_rate)
        }
        StakingInstruction::Stake { amount } => {
            stake(program_id, accounts, amount)
        }
        StakingInstruction::Unstake { amount } => {
            unstake(program_id, accounts, amount)
        }
    
    }
}