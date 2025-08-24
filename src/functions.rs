use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, example_mocks::solana_sdk::system_program, program::{invoke, invoke_signed}, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar
};
use borsh::{BorshDeserialize , BorshSerialize};
use solana_system_interface::instruction::{create_account , transfer};

use crate::{accounts::{PoolAccount, StakeAccount}, constants::Seeds};


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

  
    let (expected_pda, bump) = Pubkey::find_program_address(&[Seeds::PoolSeed ,authority_info.key.as_ref() ], program_id);
    if pool_account.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }


    let space = std::mem::size_of::<PoolAccount>();

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

  
    let clock = Clock::get()?;
    let mut pool_data = PoolAccount::try_from_slice(&pool_account.data.borrow())?;
    
    // This check is redundant if we checked for empty data earlier, but good for safety.
    if pool_data.bump != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    pool_data.authority = *authority_info.key; 
    pool_data.last_update_time = clock.unix_timestamp;
    pool_data.reward_rate = reward_rate;
    pool_data.total_staked = 0;
    pool_data.bump = bump;
    pool_data.is_active = true;

    pool_data.serialize(&mut *pool_account.data.borrow_mut())?;

    Ok(())
}

pub fn stake(program_id: &Pubkey ,accounts : &[AccountInfo] , amount : u64 )->ProgramResult {
    let account_iter = &mut accounts.iter();

    let staker = next_account_info(account_iter)?;
    let stake_account  = next_account_info(account_iter)?;
    let pool_account  = next_account_info(account_iter)?;
    let system = next_account_info(account_iter)?;


   
    let mut pool_data = PoolAccount::try_from_slice(&pool_account.data.borrow())?;

    if !pool_data.is_active{
        return Err(ProgramError::IncorrectProgramId);
    }

    if pool_account.owner != program_id {
        return Err(ProgramError::IllegalOwner)
    } 

    if stake_account.owner != program_id {
        return Err(ProgramError::IllegalOwner)
    } 
  
    if !staker.is_signer {
        return Err(ProgramError::InvalidAccountOwner);
    } 

     let (expected_pda, bump) = Pubkey::find_program_address(&[Seeds::StakeSeed , staker.key.as_ref()], program_id);
    if stake_account.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    if !stake_account.data_is_empty() {

         let mut stake_data = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
        
        if stake_data.owner != *staker.key {
            return Err(ProgramError::IllegalOwner)
        }

        stake_data.amount = stake_data.amount
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

            invoke(
        &transfer(staker.key, stake_account.key, amount),
        &[
            staker.clone(),
            stake_account.clone(),
            system.clone(),
        ],
    )?;

    pool_data.serialize(&mut *pool_account.data.borrow_mut())?;
    stake_data.serialize(&mut *stake_account.data.borrow_mut())?;

        return  Ok(());
    }

        let space = std::mem::size_of::<StakeAccount>();
        let min_lamports = Rent::get()?.minimum_balance(space);

        let signer_seeds: &[&[u8]] = &[Seeds::StakeSeed, staker.key.as_ref(), &[bump]];

    

        invoke_signed(
            &create_account(
                staker.key,
                stake_account.key,
                min_lamports,
                space as u64,
                program_id,
            ),
            &[
                staker.clone(),
                stake_account.clone(),
                system.clone(),
            ],
            &[signer_seeds],
        )?;

        invoke(
        &transfer(staker.key, stake_account.key, amount),
        &[
            staker.clone(),
            stake_account.clone(),
            system.clone(),
        ],
    )?;
    


    let clock = Clock::get()?;

       pool_data.total_staked = pool_data.total_staked
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let stake_data = StakeAccount {
        owner: *staker.key,
        amount,
        bump,
        last_stake_time: clock.unix_timestamp,
    };

    pool_data.serialize(&mut *pool_account.data.borrow_mut())?;
    stake_data.serialize(&mut *stake_account.data.borrow_mut())?;

    Ok(())
}

