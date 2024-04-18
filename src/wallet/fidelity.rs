use bitcoin::{absolute::LockTime, OutPoint, PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Hash)]
pub struct FidelityBond {
    pub outpoint: OutPoint,
    pub amount: u64,
    pub lock_time: LockTime,
    pub pubkey: PublicKey,
    // Height at which the bond was confirmed.
    pub conf_height: u32,
    // Cert expiry denoted in multiple of difficulty adjustment period (2016 blocks)
    pub cert_expiry: u64,
}
