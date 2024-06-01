use anchor_lang::prelude::*;
use crate::errors::*;
use crate::state::{GlobalState, RoundState};
use crate::constants::{ GLOBAL_STATE_SEED, VAULT_SEED, ROUND_STATE_SEED };

use solana_program::{program::{invoke, invoke_signed}, system_instruction};

use std::mem::size_of;


pub fn initialize(ctx: Context<Initialize>, fee: u64) -> Result<()> {
    let accts = ctx.accounts;

    accts.global_state.owner = accts.owner.key();
    accts.global_state.vault = accts.vault.key();
    accts.global_state.total_round = 0;
    accts.global_state.fee = fee;
    accts.global_state.round_states = Vec::new();

    Ok(())
}

pub fn create_round(ctx: Context<Create>, round_index: u32, amount: u64) -> Result<()> {
    let accts = ctx.accounts;

    if accts.global_state.total_round + 1 != round_index {
        return Err(RPSGame::InvalidIndex.into());
    }
    if amount <= 0 {
        return Err(RPSGame::ZeroAmount.into());
    }
    
    accts.round.creator = accts.user.key();
    accts.round.deposit_amount = amount;
    accts.round.joiner = Pubkey::default();
    accts.round.status = false;
    accts.round.creator_result = 0;
    accts.round.joiner_result = 0;
    accts.round.start_time = 0;
    accts.round.round_index = round_index;
    accts.round.is_creator_claimed = false;
    accts.round.is_joiner_claimed = false;

    // Add the public key of the new RoundState account to the round_states vector
    accts.global_state.round_states.push(accts.round.key());

    // Increment the total_round counter in GlobalState
    accts.global_state.total_round += 1;

    invoke(
        &system_instruction::transfer(
            &accts.user.key(),
            &accts.vault.key(),
            amount
        ),
        &[
            accts.user.to_account_info().clone(),
            accts.vault.clone(),
            accts.system_program.to_account_info().clone(),
        ],
    )?;

    Ok(())
}

pub fn join_round(ctx: Context<Join>, round_index: u32) -> Result<()> {
    let accts = ctx.accounts;

    //Check if round started
    if accts.round.status {
        return Err(RPSGame::InvalidStart.into());
    }

    //check round_index is correct
    if accts.global_state.total_round < round_index {
        return Err(RPSGame::InvalidIndex.into());
    }

    accts.round.joiner = accts.user.key();
    accts.round.status = true;
    accts.round.start_time = accts.clock.unix_timestamp;

    invoke(
        &system_instruction::transfer(
            &accts.user.key(),
            &accts.vault.key(),
            accts.round.deposit_amount
        ),
        &[
            accts.user.to_account_info().clone(),
            accts.vault.clone(),
            accts.system_program.to_account_info().clone(),
        ],
    )?;

    Ok(())
}

pub fn play(ctx: Context<Play>, round_index: u32, is_creator: bool, result:u8) -> Result<()> {
    let accts = ctx.accounts;

    //Check round_index is correct
    if accts.global_state.total_round < round_index {
        return Err(RPSGame::InvalidIndex.into());
    }

    //Check player is correct
    let player = if is_creator {
        accts.round.creator
    } else {
        accts.round.joiner
    };
    if player != accts.user.key() {
        return Err(RPSGame::InvalidPlayer.into());
    }

    //Check player already played
    let player_result = if is_creator {
        accts.round.creator_result
    } else {
        accts.round.joiner_result
    };
    if player_result != 0 {
        return Err(RPSGame::InvalidPlay.into());
    }

    //Check time is valid
    if accts.round.start_time < accts.clock.unix_timestamp - (10 * 60) {
        return Err(RPSGame::InvalidTime.into());
    }

    //Set result of player
    if is_creator {
        accts.round.creator_result = result
    } else {
        accts.round.joiner_result = result
    }

    Ok(())
}

pub fn claim(ctx: Context<Claim>, round_index: u32, is_creator: bool) -> Result<()> {
    let accts = ctx.accounts;

    //Check round_index is correct
    if accts.global_state.total_round < round_index {
        return Err(RPSGame::InvalidIndex.into());
    }

    //Check player is correct
    let player = if is_creator {
        accts.round.creator
    } else {
        accts.round.joiner
    };
    if player != accts.user.key() {
        return Err(RPSGame::InvalidPlayer.into());
    }

    //Check is claimed
    let is_claimed = if is_creator {
        accts.round.is_creator_claimed
    } else {
        accts.round.is_joiner_claimed
    };

    if is_claimed {
        return Err(RPSGame::InvalidClaim.into());
    };

    //Check claim is valid
    if (accts.round.creator_result == 0 || accts.round.joiner_result == 0) && accts.round.start_time > accts.clock.unix_timestamp - (10 * 60) {
        return Err(RPSGame::InvalidClaim.into());
    }


    //Set is claimed value
    if is_creator {
        accts.round.is_creator_claimed = true;
    } else  {
        accts.round.is_joiner_claimed = true;
    }

    let mut winner = Pubkey::default();
    let mut is_winner_set = false;
    let mut claim_amount = 0;

    if accts.round.creator_result == 0 && accts.round.joiner_result != 0{
        winner = accts.round.joiner;
        is_winner_set = true;
    } else if accts.round.creator_result != 0 && accts.round.joiner_result == 0 {
        winner = accts.round.creator;
        is_winner_set = true;
    } else if accts.round.creator_result != 0 && accts.round.joiner_result != 0 {
        // Determine the winner based on the game logic
        match (accts.round.creator_result, accts.round.joiner_result) {
            // Rock vs Scissors or Scissors vs Paper or Paper vs Rock
            (1, 3) | (3, 2) | (2, 1) => {
                winner = accts.round.creator;
                is_winner_set = true;
            },
            // Scissors vs Rock or Paper vs Scissors or Rock vs Paper
            (3, 1) | (2, 3) | (1, 2) => {
                winner = accts.round.joiner;
                is_winner_set = true;
            },
            _ => {}
        }
    }

    if is_winner_set {
        if winner == accts.user.key(){
            claim_amount = accts.round.deposit_amount * 2;
        }
    } else {
        claim_amount = accts.round.deposit_amount
    }

    //Refuse if claim_amount is 0
    if claim_amount == 0 {
        return Err(RPSGame::InvalidClaim.into());
    }

    // send sol to vault
    let transfer_amount = claim_amount * (1000 - accts.global_state.fee) / 1000;
    let fee_amount = claim_amount * accts.global_state.fee / 1000;

    let (_, bump) = Pubkey::find_program_address(&[VAULT_SEED], &crate::ID);

    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.user.key(), transfer_amount),
        &[
            accts.vault.to_account_info().clone(),
            accts.user.to_account_info().clone(),
            accts.system_program.to_account_info().clone(),
        ],
        &[&[VAULT_SEED, &[bump]]],
    )?;

    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.owner.key(), fee_amount),
        &[
            accts.vault.to_account_info().clone(),
            accts.owner.to_account_info().clone(),
            accts.system_program.to_account_info().clone(),
        ],
        &[&[VAULT_SEED, &[bump]]],
    )?;

    Ok(())
}

pub fn update_fee(ctx: Context<Update>, new_fee: u64) -> Result<()> {
    let accts = ctx.accounts;
    require!(accts.global_state.owner == accts.owner.key(), RPSGame::NotAllowedOwner);

    accts.global_state.fee = new_fee;
    Ok(())
}

pub fn update_owner(ctx: Context<SetData>, new_owner: Pubkey) -> Result<()> {
    let accts = ctx.accounts;

    if accts.global_state.owner != accts.owner.key() {
        return Err(RPSGame::NotAllowedOwner.into());
    }

    accts.global_state.owner = new_owner;

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        space = 9600
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(round_index:u32)]
pub struct Create<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        address = global_state.vault
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    #[account(
        init,
        payer = user,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes(), user.key().as_ref()],
        bump,
        space = 8 + size_of::<RoundState>()
    )]
    pub round: Account<'info, RoundState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(round_index:u32)]
pub struct Join<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        address = global_state.vault
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    #[account(
        mut,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes(), round.creator.as_ref()],
        bump,
    )]
    pub round: Account<'info, RoundState>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(round_index:u32)]
pub struct Play<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes(), round.creator.as_ref()],
        bump,
    )]
    pub round: Account<'info, RoundState>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(round_index:u32)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// CHECK: doc comment explaining why no checks through types are necessary.
    #[account(
        mut,
        address = global_state.owner
    )]
    pub owner: AccountInfo<'info>,
    
    #[account(
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        address = global_state.vault
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    #[account(
        mut,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes(), round.creator.as_ref()],
        bump,
    )]
    pub round: Account<'info, RoundState>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED], 
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetData<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
}