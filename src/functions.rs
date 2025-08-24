use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use borsh::{BorshDeserialize , BorshSerialize};
use solana_system_interface::instruction::create_account;


pub fn initialize_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    reward_rate: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let pool_account = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }


    if !pool_account.data_is_empty() {
         return Err(ProgramError::AccountAlreadyInitialized);
    }

    // --- PDA Check ---
    let (expected_pda, bump) = Pubkey::find_program_address(&[b"pool"], program_id);
    if pool_account.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // --- Create Account via CPI ---
    let space = std::mem::size_of::<PoolAccount>();
    // Use Rent::get() which is simpler than passing the sysvar account
    let min_lamports = Rent::get()?.minimum_balance(space);

    invoke_signed(
        &create_account(
            authority_info.key,
            pool_account.key,
            min_lamports,
            space as u64,
            program_id,
        ),
        &[
            authority_info.clone(),
            pool_account.clone(),
            system_program.clone(),
        ],
        &[&[b"pool", &[bump]]],
    )?;

    // --- Initialize State ---
    let clock = Clock::get()?;
    let mut pool_data = PoolAccount::try_from_slice(&pool_account.data.borrow())?;
    
    // This check is redundant if we checked for empty data earlier, but good for safety.
    if pool_data.bump != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    pool_data.authority = *authority_info.key; // CORRECT: Set signer as authority
    pool_data.last_update_time = clock.unix_timestamp;
    pool_data.reward_rate = reward_rate;
    pool_data.total_staked = 0;
    pool_data.bump = bump;

    pool_data.serialize(&mut *pool_account.data.borrow_mut())?;

    Ok(())
}


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolAccount {
    pub authority: Pubkey,          
    pub reward_rate: u64,           
    pub total_staked: u64,          
    pub last_update_time: i64,      
    pub bump: u8,                   
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StakeAccount {
    pub owner: Pubkey,   
    pub amount: u64,               
    pub reward_debt: u64,         
    pub last_stake_time: i64,       
    pub bump: u8,                   
}