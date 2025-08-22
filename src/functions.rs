use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
 clock::Clock, entrypoint::ProgramResult, instruction::{AccountMeta, Instruction}, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::{Sysvar, SysvarSerialize}
    };


use solana_system_interface::instruction::create_account;




pub fn initialize_pool( 
    program_id: &Pubkey,
    accounts: &[AccountInfo] , 
    reward_rate :u64
)->ProgramResult{

let account_iter = &mut accounts.iter();
let pool_account = next_account_info(account_iter)?;
let authority_info = next_account_info(account_iter)?;
let system_program = next_account_info(account_iter)?;
let rent_sysvar = next_account_info(account_iter)?;



  if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
   
    if pool_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

if PoolAccount::try_from_slice(&pool_account.data.borrow()).is_ok() {
    return Err(ProgramError::AccountAlreadyInitialized);
}



let clock = Clock::get()?;
let time = clock.unix_timestamp;

 let (expected_pda, bump) = Pubkey::find_program_address(&[b"pool"], program_id);
    if pool_account.key != &expected_pda {
        return Err(ProgramError::InvalidAccountData);
    }
 


 let rent = &Rent::from_account_info(rent_sysvar)?;   
 let space = std::mem::size_of::<PoolAccount>();
 let minlamports = rent.minimum_balance(space);

let ix: Instruction = create_account(
    authority_info.key,         
    pool_account.key,            
    minlamports,              
    space as u64,                
    program_id,               
);


invoke_signed(
    &ix,
    &[
        authority_info.clone(),     
        pool_account.clone(),    
        system_program.clone(),    
    ],
    &[&[b"pool", &[bump]]],
)?;


let pool_account_onchain = PoolAccount {
    authority :*program_id,
    last_update_time :time,
    reward_rate :reward_rate,
    total_staked :0,
    bump :bump
};



pool_account_onchain.serialize(&mut *pool_account.data.borrow_mut())?;

Ok(())
}

pub fn stake(
    program_id: &Pubkey,
    accounts: &[AccountInfo] ,
    amount :u32
)->ProgramResult{


Ok(())
}


pub fn unstake(
    program_id: &Pubkey,
    accounts: &[AccountInfo] ,
    amount :u32
)->ProgramResult{


Ok(())
}

pub fn claim_rewards(program_id :&Pubkey , accounts  : &[AccountInfo])->ProgramResult{



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