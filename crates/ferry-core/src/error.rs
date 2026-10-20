//! Errors FERRY's engine reports.

/// Everything that can go wrong in `ferry-core`. Messages are written to be shown to a user.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("\"{0}\" is not a server address (expected host:port)")]
    InvalidServer(String),

    #[error("could not reach the server: {0}")]
    Connection(#[from] tonic::transport::Error),

    #[error("the server returned an error: {0}")]
    Server(#[from] tonic::Status),

    #[error("the recovery phrase is not valid: {0}")]
    InvalidPhrase(String),

    #[error("a wallet already exists in {0}")]
    WalletExists(String),

    #[error("no wallet found in {0}")]
    NoWallet(String),

    #[error("the wallet database failed: {0}")]
    Wallet(String),

    #[error("this recovery phrase does not belong to this wallet")]
    WrongPhrase,

    #[error("a migration is already in progress")]
    MigrationInProgress,

    #[error("the server rejected the transaction ({0})")]
    Rejected(String),

    #[error("syncing failed: {0}")]
    Sync(String),

    #[error("could not read or write wallet files: {0}")]
    Io(#[from] std::io::Error),
}
