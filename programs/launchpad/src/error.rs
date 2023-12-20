use anchor_lang::error_code;

#[error_code]
pub enum LaunchpadError {
    #[msg("Invalid Auction")]
    InvalidAuction,
    #[msg("Invalid Token")]
    InvalidToken,
    #[msg("Insufficient Token")]
    InsufficientTokens,
}
