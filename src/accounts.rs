use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolAccount {
    pub authority: Pubkey,          
    pub reward_rate: u64,           
    pub total_staked: u64,          
    pub last_update_time: i64,      
    pub bump: u8,
    pub is_active :bool                   
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StakeAccount {
    pub owner: Pubkey,   
    pub amount: u64,               
    // pub reward_debt: u64,         
    pub last_stake_time: i64,       
    pub bump: u8,                   
}