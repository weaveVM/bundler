use alloy::hex::FromHexError;
use alloy::network::{Ethereum, TransactionBuilderError};
use alloy::signers::local::LocalSignerError;
use alloy::transports::{RpcError, TransportErrorKind};
use eyre::ErrReport;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid Keystore Path or Password")]
    InvalidKeystore,
    #[error("Bundle must have envelopes")]
    EnvelopesNeeded,
    #[error("Bundle or envelope must have a private key")]
    PrivateKeyNeeded,
    #[error("Bundle was not able to be retrieved")]
    BundleRetrievalProblem,
    #[error("Address is not verified")]
    UnverifiedAddress,
    #[error("Bundle could not be created")]
    BundleNotCreated,
    #[error("Error reconstructing the Large Bundle")]
    LargeBundleReconstruction,
    #[error("Error retrieving envelope receipts of the Large Bundle")]
    LargeBundleChunksRetrieval,
    #[error("Large Bundle missing SuperAccount instance")]
    SuperAccountNeeded,
    #[error("SuperAccount instance missing chunkers")]
    ChunkersNeeded,
    #[error("Other")]
    Other(String),
    #[error("Error parsing private key")]
    PrivateKeyParsingError,
    #[error("Invalid RPC Url")]
    InvalidRpcUrl,
    #[error("There's been an issue with the current RPC call")]
    RpcTransportError(#[from] RpcError<TransportErrorKind>),
    #[error("Hex could not be parsed")]
    HexError(#[from] FromHexError),
    #[error("Signature or its keys have errors")]
    SigningError(#[from] LocalSignerError),
    #[error("Eyre Other")]
    ReportError(#[from] ErrReport),
    #[error("Allow Tx Error")]
    TransactionError(#[from] TransactionBuilderError<Ethereum>),
}
