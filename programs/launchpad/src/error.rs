use anchor_lang::error_code;

#[error_code]
pub enum LaunchpadError {
    #[msg("Invalid Auction")]
    InvalidAuction,
    #[msg("Invalid Token")]
    InvalidToken,
    #[msg("Insufficient Token")]
    InsufficientTokens,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Auction Not Ended")]
    AuctionNotEnded,
    #[msg("Auction Disabled")]
    AuctionDisabled,
    #[msg("Non Native Auction")]
    NonNativeAuction,
    #[msg("Non SPL Auction")]
    NonSplAuction,
}
