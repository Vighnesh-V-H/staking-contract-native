use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize , BorshSerialize)]
pub enum StakingInstruction {
InitializePool{reward_rate : u64},
Stake {amount : u32},
Unstake{amount :u32},
ClaimRewards,
}