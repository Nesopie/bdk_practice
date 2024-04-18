use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Ord, PartialEq, Eq, PartialOrd, Clone)]
pub enum KeychainKind {
    External,
    Internal,
    IncomingSwapCoins(String),
    OutgoingSwapCoins(String),
}
