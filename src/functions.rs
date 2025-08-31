use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, example_mocks::solana_sdk::system_program, msg, program::{invoke, invoke_signed}, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar
};
use borsh::{BorshDeserialize , BorshSerialize};
use solana_system_interface::instruction::{allocate, create_account, transfer};

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

pub fn unstake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64
) -> ProgramResult {
    let account_itr = &mut accounts.iter();
    let signer = next_account_info(account_itr)?;
    let stake_account = next_account_info(account_itr)?;
    let pool_account = next_account_info(account_itr)?;
    let system = next_account_info(account_itr)?;

    let mut pool_account_data = PoolAccount::try_from_slice(&pool_account.data.borrow())?;
    let mut stake_account_data = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
   
    
    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if pool_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    
    if stake_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    if !pool_account_data.is_active {
        return Err(ProgramError::InvalidArgument);
    }

    let (pool_pda, bump1) = Pubkey::find_program_address(&[Seeds::PoolSeed, pool_account_data.authority.as_ref()], program_id);
    let (stake_pda, bump2) = Pubkey::find_program_address(&[Seeds::StakeSeed, stake_account_data.owner.as_ref()], program_id);

    if pool_pda != *pool_account.key {
        msg!("Invalid pool PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    if stake_account_data.owner != *signer.key {
        return Err(ProgramError::IllegalOwner);
    }
        
    if stake_pda != *stake_account.key {
        msg!("Invalid stake PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    if amount > stake_account_data.amount {
        return Err(ProgramError::InsufficientFunds);
    }

    let clock = Clock::get()?;
    let time_passed = (clock.unix_timestamp - stake_account_data.last_stake_time) as u64;

    let pending_rewards = if pool_account_data.total_staked == 0 {
        0
    } else {
        ((time_passed as u128) * (pool_account_data.reward_rate as u128) * (amount as u128) / (pool_account_data.total_staked as u128)) as u64
    };

    stake_account_data.amount = stake_account_data.amount
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    pool_account_data.total_staked = pool_account_data.total_staked
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let stake_signer_seeds: &[&[u8]] = &[Seeds::StakeSeed, signer.key.as_ref(), &[bump2]];

    invoke_signed(
        &transfer(stake_account.key, signer.key, amount),
        &[
            stake_account.clone(),
            signer.clone(),
            system.clone(),
        ],
        &[stake_signer_seeds],
    )?;

    if pending_rewards > 0 {
        let pool_signer_seeds: &[&[u8]] = &[Seeds::PoolSeed, pool_account_data.authority.as_ref(), &[bump1]];

        invoke_signed(
            &transfer(pool_account.key, signer.key, pending_rewards),
            &[
                pool_account.clone(),
                signer.clone(),
                system.clone(),
            ],
            &[pool_signer_seeds],
        )?;
    }

    if stake_account_data.amount == 0 {
       
        invoke_signed(
            &allocate(stake_account.key, 0),
            &[stake_account.clone(), system.clone()],
            &[stake_signer_seeds],
        )?;

        let remaining_lamports = **stake_account.lamports.borrow();

        invoke_signed(
            &transfer(stake_account.key, signer.key, remaining_lamports),
            &[
                stake_account.clone(),
                signer.clone(),
                system.clone(),
            ],
            &[stake_signer_seeds],
        )?;
    } else {
        stake_account_data.last_stake_time = clock.unix_timestamp;
        stake_account_data.serialize(&mut *stake_account.data.borrow_mut())?;
    }

    pool_account_data.serialize(&mut *pool_account.data.borrow_mut())?;

    Ok(())
}

