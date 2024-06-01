mod errors;
mod events;
mod instructions;
mod state;
mod constants;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("4S5kqHKSoJEPMutUhUhzcKzC3k7tFsyVnUUMHtyPXxzq");

#[program]
pub mod rps_game {
    use super::*;
    // owner functions
    pub fn initialize(ctx: Context<Initialize>, fee: u64) -> Result<()> {
        instructions::initialize(ctx, fee)
    }

    pub fn update_owner(ctx: Context<SetData>, new_owner: Pubkey) -> Result<()> {
        instructions::update_owner(ctx, new_owner)
    }

    pub fn update_fee(ctx: Context<Update>, new_fee: u64) -> Result<()> {
        instructions::update_fee(ctx, new_fee)
    }

    //  user function
    pub fn create_round(ctx: Context<Create>, round_index: u32, amount: u64) -> Result<()> {
        instructions::create_round(ctx, round_index, amount)
    }

    pub fn join_round(ctx: Context<Join>, round_index: u32) -> Result<()> {
        instructions::join_round(ctx, round_index)
    }

    pub fn play(ctx: Context<Play>, round_index: u32, is_creator: bool, result: u8) -> Result<()> {
        instructions::play(ctx, round_index, is_creator, result)
    }

    pub fn claim(ctx: Context<Claim>, round_index: u32, is_creator: bool) -> Result<()> {
        instructions::claim(ctx, round_index, is_creator)
    }
}
