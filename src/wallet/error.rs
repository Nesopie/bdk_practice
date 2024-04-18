use std::convert::Infallible;

use bdk::{descriptor::DescriptorError, miniscript::psbt::UtxoUpdateError, signer::SignerError};
use bdk_chain::miniscript::Error;
use bitcoin::psbt::PartiallySignedTransaction;

#[derive(Debug)]
pub enum WalletError {
    File(std::io::Error),
    Cbor(serde_cbor::Error),
    BIP39(bip39::Error),
    BIP32(bitcoin::bip32::Error),
    Rpc(bitcoind::bitcoincore_rpc::Error),
    Descriptor(DescriptorError),
    HardenedKey,
    Multipath,
    Miniscript(Error),
    Infallible,
    Address(bitcoin::address::Error),
    Hex(bitcoin::hashes::hex::Error),
    Psbt(bitcoin::psbt::Error),
    Signer(SignerError),
    PsbtFinalise(
        (
            PartiallySignedTransaction,
            Vec<bdk::miniscript::psbt::Error>,
        ),
    ),
    DescriptorConversion(bdk::miniscript::descriptor::ConversionError),
    UtxoUpdate(UtxoUpdateError),
}

impl From<std::io::Error> for WalletError {
    fn from(e: std::io::Error) -> Self {
        Self::File(e)
    }
}

impl From<bitcoind::bitcoincore_rpc::Error> for WalletError {
    fn from(value: bitcoind::bitcoincore_rpc::Error) -> Self {
        Self::Rpc(value)
    }
}

impl From<bitcoin::bip32::Error> for WalletError {
    fn from(value: bitcoin::bip32::Error) -> Self {
        Self::BIP32(value)
    }
}

impl From<bip39::Error> for WalletError {
    fn from(value: bip39::Error) -> Self {
        Self::BIP39(value)
    }
}

impl From<serde_cbor::Error> for WalletError {
    fn from(value: serde_cbor::Error) -> Self {
        Self::Cbor(value)
    }
}

impl From<DescriptorError> for WalletError {
    fn from(value: DescriptorError) -> Self {
        Self::Descriptor(value)
    }
}

impl From<Error> for WalletError {
    fn from(value: Error) -> Self {
        Self::Miniscript(value)
    }
}

impl From<Infallible> for WalletError {
    fn from(_: Infallible) -> Self {
        Self::Infallible
    }
}

impl From<bitcoin::address::Error> for WalletError {
    fn from(value: bitcoin::address::Error) -> Self {
        Self::Address(value)
    }
}

impl From<bitcoin::hashes::hex::Error> for WalletError {
    fn from(value: bitcoin::hashes::hex::Error) -> Self {
        Self::Hex(value)
    }
}

impl From<bitcoin::psbt::Error> for WalletError {
    fn from(value: bitcoin::psbt::Error) -> Self {
        Self::Psbt(value)
    }
}

impl From<SignerError> for WalletError {
    fn from(value: SignerError) -> Self {
        Self::Signer(value)
    }
}

impl
    From<(
        PartiallySignedTransaction,
        Vec<bdk::miniscript::psbt::Error>,
    )> for WalletError
{
    fn from(
        value: (
            PartiallySignedTransaction,
            Vec<bdk::miniscript::psbt::Error>,
        ),
    ) -> Self {
        Self::PsbtFinalise((value.0, value.1))
    }
}

impl From<bdk::miniscript::descriptor::ConversionError> for WalletError {
    fn from(value: bdk::miniscript::descriptor::ConversionError) -> Self {
        Self::DescriptorConversion(value)
    }
}

impl From<UtxoUpdateError> for WalletError {
    fn from(value: UtxoUpdateError) -> Self {
        Self::UtxoUpdate(value)
    }
}
