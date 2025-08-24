use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize , BorshSerialize)]
pub enum StakingInstruction {
InitializePool{reward_rate : u64},
Stake {amount : u64},
Unstake{amount :u64},
ClaimRewards,
}