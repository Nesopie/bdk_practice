use bdk::{
    descriptor::{self, ExtendedDescriptor, IntoWalletDescriptor},
    keys::{DescriptorPublicKey, KeyMap},
    miniscript::{
        descriptor::{DescriptorXKey, Wildcard},
        ForEachKey,
    },
    Wallet,
};
use bitcoin::{bip32::ChildNumber, key::Secp256k1, secp256k1::All, Network};

use super::error::WalletError;

pub fn into_wallet_descriptor_checked<E: IntoWalletDescriptor>(
    descriptor: E,
    secp: &Secp256k1<All>,
    network: Network,
) -> Result<(ExtendedDescriptor, KeyMap), WalletError> {
    let (descriptor, keymap) = descriptor.into_wallet_descriptor(secp, network)?;

    let descriptor_contains_hardened_key = descriptor.for_any_key(|k| {
        if let DescriptorPublicKey::XPub(DescriptorXKey {
            derivation_path,
            wildcard,
            ..
        }) = k
        {
            return *wildcard == Wildcard::Hardened
                || derivation_path.into_iter().any(ChildNumber::is_hardened);
        }

        false
    });

    if descriptor_contains_hardened_key {
        return Err(WalletError::HardenedKey);
    }

    if descriptor.is_multipath() {
        return Err(WalletError::Multipath);
    }

    descriptor.sanity_check()?;

    return Ok((descriptor, keymap));
}
