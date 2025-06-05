
use anchor_lang::prelude::*;

#[error_code]
pub enum ContractError {    
    #[msg("Calculation Error.")]
    CalcError,

    #[msg("MathOverflow.")]
    MathOverflow,

    #[msg("Invalid Authority.")]
    InvalidAuthority,

    #[msg("Invalid Colleague.")]
    InvalidColleague,

    #[msg("Exceed Max Num Reward Tokens.")]
    ExceedMaxNumRewardTokens,

    #[msg("Invalid Fee Receiver.")]
    InvalidFeeReceiver,

    #[msg("Invalid Mint.")]
    InvalidMint,

    #[msg("Rarity Already Exist.")]
    RarityAlreadyExist,

    #[msg("Player Not In Pending List.")]
    PlayerNotInPendingList,

    #[msg("No Reward Token Left.")]
    NoRewardTokenLeft,

    #[msg("Flip Not Available.")]
    FlipNotAvailable,

    #[msg("Reward Token Not Exist.")]
    RewardTokenNotExist,

    #[msg("Reward Token Not Empty.")]
    RewardTokenNotEmpty,

    #[msg("Exceed Max Num Reveal Pending Players.")]
    ExceedMaxNumRevealPendingPlayers,

    #[msg("Invalid Payment Token Mint.")]
    InvalidPaymentTokenMint,

    #[msg("Reward Token Mismatch.")]
    RewardTokenMismatch,

    #[msg("Invalid Program Id.")]
    InvalidProgramId,

    #[msg("Invalid Fee Config.")]
    InvalidFeeConfig,
}
