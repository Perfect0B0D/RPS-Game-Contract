use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct GlobalState {
    pub owner: Pubkey, // the pubkey of owner
    pub vault: Pubkey,
    pub total_round: u32,
    pub fee: u64,
    pub round_states: Vec<Pubkey>, // Vector of RoundState public keys
}

#[account]
#[derive(Default)]
pub struct RoundState {
   pub round_index: u32,
   pub creator: Pubkey,
   pub deposit_amount: u64,
   pub joiner: Pubkey,
   pub status: bool,
   pub creator_result: u8,
   pub joiner_result: u8,
   pub start_time: i64,
   pub is_creator_claimed: bool,
   pub is_joiner_claimed: bool,
}

