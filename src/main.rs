use std::{collections::BTreeMap, path::PathBuf, str::FromStr};

use bitcoin::{bip32::ExtendedPrivKey, block, Address};
use bitcoind::bitcoincore_rpc::{Auth, Client, RpcApi};
use wallet::{error::WalletError, keychain_kind::KeychainKind, store::WalletStore};

pub mod wallet;

//create an index
//create a indexed graph
//create a client
//create a wallet
//create a transaction
//apply the transaction to the graph
//reflect these changes in the persistent backend

fn main() -> Result<(), WalletError> {
    let auth = Auth::UserPass("admin1".to_string(), "123".to_string());

    let client = Client::new("http://localhost:18443/wallet/wallet-1", auth)?;

    let seed_phrase = "joke toward behave milk rich solution label never aerobic you tennis expect";
    let seed = bip39::Mnemonic::parse(seed_phrase)?;
    let seed = seed.to_seed("");
    let master = ExtendedPrivKey::new_master(bitcoin::Network::Regtest, &seed)?;

    println!("master: {}", master);

    let external_descriptor = "wpkh(tprv8ZgxMBicQKsPdiwByQQc3iwmoDUh6UTTJPZwVj6J8Vqe4UMpf2HVDybW145pn46RFNjh2MqERd68Kz9c4jQutAC3EnV3XGmuecSZ8wgfaXZ/84'/1'/0'/0/*)";
    let internal_descriptor = "wpkh(tprv8ZgxMBicQKsPdiwByQQc3iwmoDUh6UTTJPZwVj6J8Vqe4UMpf2HVDybW145pn46RFNjh2MqERd68Kz9c4jQutAC3EnV3XGmuecSZ8wgfaXZ/84'/1'/0'/1/*)";
    // let (incoming_swapcoin_descriptor, _) = ExtendedDescriptor::parse_descriptor(&secp, "wpkh(tprv8ZgxMBicQKsPdiwByQQc3iwmoDUh6UTTJPZwVj6J8Vqe4UMpf2HVDybW145pn46RFNjh2MqERd68Kz9c4jQutAC3EnV3XGmuecSZ8wgfaXZ/84'/1'/0'/0/*)")?;

    // client.generate_to_address(1, );

    let mut descriptors = BTreeMap::<KeychainKind, &str>::new();
    descriptors.insert(KeychainKind::External, external_descriptor);
    descriptors.insert(KeychainKind::Internal, internal_descriptor);

    let path = PathBuf::from_str("./wallets")?;

    let file_name = PathBuf::from_str("wallet-1")?;

    let mut store = WalletStore::new::<&str>(
        bitcoin::Network::Regtest,
        descriptors,
        client,
        file_name,
        path,
    )?;

    let wallet_name = store.file_name.to_str().unwrap().to_string();

    if store.oracle.rpc.list_wallets()?.contains(&wallet_name) {
        println!("Wallet already loaded");
    } else {
        // let args = [
        //     Value::String(wallet_name.clone()),
        //     Value::Bool(true),  // Disable Private Keys
        //     Value::Bool(false), // Create a blank wallet
        //     Value::Null,        // Optional Passphrase
        //     Value::Bool(false), // Avoid Reuse
        //     Value::Bool(true),  // Descriptor Wallet
        // ];

        // let val: Value = client.call("createwallet", &args)?;
        // println!("{}", val);
    }

    // store.increment_index(&client);
    // store.send_to_address(
    //     Address::from_str("bcrt1q7xy4m2ga3n50966e83v7zyq9qhmzuf5ludnhfj")
    //         .unwrap()
    //         .assume_checked(),
    //     1,
    // )?;

    let address = store.get_address()?;

    println!("{}", address);
    println!("{}", address.script_pubkey());

    store.send_to_address(store.get_address()?, 10000)?;

    // store.generate_to_address();

    // let block_hash = client.generate_to_address(1, &address)?[0];

    // store.insert_block_relevant(&block_hash, &client)?;

    // let (internal_)
    // let index = KeychainTxOutIndex::<KeychainKind>::new(0);
    // let tx_graph =
    //     IndexedTxGraph::<ConfirmationTimeHeightAnchor, KeychainTxOutIndex<KeychainKind>>::new(
    //         index,
    //     );

    // let mut path = PathBuf::new();
    // path.push("./");

    Ok(())
}
