use bitcoin::{ecdsa::Signature, secp256k1::SecretKey, PublicKey, ScriptBuf, Transaction};

/// Defines the length of the Preimage.
pub const PREIMAGE_LEN: usize = 32;

/// Type for Preimage.
pub type Preimage = [u8; PREIMAGE_LEN];

/// Represents an incoming swapcoin.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct IncomingSwapCoin {
    pub my_privkey: SecretKey,
    pub other_pubkey: PublicKey,
    pub other_privkey: Option<SecretKey>,
    pub contract_tx: Transaction,
    pub contract_redeemscript: ScriptBuf,
    pub hashlock_privkey: SecretKey,
    pub funding_amount: u64,
    pub others_contract_sig: Option<Signature>,
    pub hash_preimage: Option<Preimage>,
}

/// Represents an outgoing swapcoin.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OutgoingSwapCoin {
    pub my_privkey: SecretKey,
    pub other_pubkey: PublicKey,
    pub contract_tx: Transaction,
    pub contract_redeemscript: ScriptBuf,
    pub timelock_privkey: SecretKey,
    pub funding_amount: u64,
    pub others_contract_sig: Option<Signature>,
    pub hash_preimage: Option<Preimage>,
}
