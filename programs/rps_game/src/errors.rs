use anchor_lang::error_code;

#[error_code]
pub enum RPSGame {
    #[msg("RPSGameError: Not allowed owner")]
    NotAllowedOwner,

    #[msg("RPSGameError: Invalid Amount")]
    InvalidAmount,

    #[msg("RPSGameError: Invalid Index")]
    InvalidIndex,

    #[msg("RPSGameError: Should depsoit than 0")]
    ZeroAmount,

    #[msg("RPSGameError: Already Started")]
    InvalidStart,

    #[msg("RPSGameError: Already Played")]
    InvalidPlay,

    #[msg("RPSGameError: Invalid Player")]
    InvalidPlayer,

    #[msg("RPSGameError: Invalid Time")]
    InvalidTime,

    #[msg("RPSGameError: Invalid Claim Time")]
    InvalidClaim
}