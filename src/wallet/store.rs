use bdk::{
    descriptor::IntoWalletDescriptor, miniscript::psbt::PsbtExt, signer::SignersContainer,
    SignOptions,
};
use bdk_chain::{
    indexed_tx_graph,
    keychain::{self, KeychainTxOutIndex},
    Append, BlockId, ChainOracle, ConfirmationTimeHeightAnchor, IndexedTxGraph, Persist,
    PersistBackend,
};
use bitcoin::{
    absolute::Height, key::Secp256k1, psbt::Psbt, Address, BlockHash, Network, OutPoint, Script,
    Transaction, TxIn, TxOut, Witness,
};
use bitcoind::bitcoincore_rpc::{Client, RpcApi};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    path::PathBuf,
    sync::Arc,
};

const DENOMINATION: u64 = 100_000_000;

use super::{
    error::WalletError, keychain_kind::KeychainKind, signer::into_wallet_descriptor_checked,
};

pub struct Oracle {
    pub rpc: Client,
}

impl ChainOracle for Oracle {
    type Error = bitcoind::Error;

    fn get_chain_tip(&self) -> Result<bdk_chain::BlockId, Self::Error> {
        let block_chain_info = self.rpc.get_blockchain_info()?;
        Ok(BlockId {
            hash: block_chain_info.best_block_hash,
            height: block_chain_info.blocks as u32,
        })
    }
    fn is_block_in_chain(
        &self,
        block: bdk_chain::BlockId,
        chain_tip: bdk_chain::BlockId,
    ) -> Result<Option<bool>, Self::Error> {
        Ok(Some(true))
    }
}

impl Default for Changeset {
    fn default() -> Self {
        Self {
            indexed_tx_graph: indexed_tx_graph::ChangeSet::default(),
        }
    }
}

impl Append for Changeset {
    fn is_empty(&self) -> bool {
        self.indexed_tx_graph.is_empty()
    }
    fn append(&mut self, other: Self) {
        Append::append(&mut self.indexed_tx_graph, other.indexed_tx_graph)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Changeset {
    indexed_tx_graph: indexed_tx_graph::ChangeSet<
        ConfirmationTimeHeightAnchor,
        keychain::ChangeSet<KeychainKind>,
    >,
}

pub struct WalletStore {
    signers: BTreeMap<KeychainKind, Arc<SignersContainer>>,
    pub indexed_graph:
        IndexedTxGraph<ConfirmationTimeHeightAnchor, KeychainTxOutIndex<KeychainKind>>,
    network: Network,
    pub oracle: Oracle,
    pub file_name: PathBuf,
    path: PathBuf,
    pub persist: Persist<KeychainStore, Changeset>,
}

impl
    From<
        indexed_tx_graph::ChangeSet<
            ConfirmationTimeHeightAnchor,
            keychain::ChangeSet<KeychainKind>,
        >,
    > for Changeset
{
    fn from(
        indexed_tx_graph: indexed_tx_graph::ChangeSet<
            ConfirmationTimeHeightAnchor,
            keychain::ChangeSet<KeychainKind>,
        >,
    ) -> Self {
        Self { indexed_tx_graph }
    }
}

impl WalletStore {
    pub fn new<E: IntoWalletDescriptor>(
        network: Network,
        descriptors: BTreeMap<KeychainKind, E>,
        rpc: Client,
        file_name: PathBuf,
        path: PathBuf,
    ) -> Result<Self, WalletError> {
        let keychain_index = KeychainTxOutIndex::<KeychainKind>::default();
        let mut indexed_graph = IndexedTxGraph::new(keychain_index);

        let secp = Secp256k1::new();

        //iterate over each keychain and create a signer for that keychain
        let mut signers = BTreeMap::<KeychainKind, Arc<SignersContainer>>::new();
        for (keychain_kind, descriptor) in descriptors {
            let (descriptor, keymap) = into_wallet_descriptor_checked(descriptor, &secp, network)?;
            let signer = Arc::new(SignersContainer::build(keymap, &descriptor, &secp));

            indexed_graph
                .index
                .add_keychain(keychain_kind.clone(), descriptor);
            signers.insert(keychain_kind, signer);
        }

        let mut keychain_store = KeychainStore {
            file_name: file_name.clone(),
            path: path.clone(),
        };

        let initial_changeset = match keychain_store.load_from_persistence()? {
            Some(changeset) => changeset,
            None => Changeset::default(),
        };

        // println!("initial changeset: {:?}", initial_changeset);

        let mut persist = Persist::<KeychainStore, Changeset>::new(keychain_store);

        indexed_graph.apply_changeset(initial_changeset.indexed_tx_graph);

        persist.stage(Changeset::from(indexed_graph.initial_changeset()));
        persist.commit()?;

        Ok(Self {
            signers,
            indexed_graph,
            oracle: Oracle { rpc },
            network,
            file_name,
            path,
            persist,
        })
    }

    pub fn add_descriptor<E: IntoWalletDescriptor>(
        &mut self,
        keychain: KeychainKind,
        descriptor_to_add: E,
    ) -> Result<(), WalletError> {
        let (descriptor, keymap) =
            into_wallet_descriptor_checked(descriptor_to_add, &Secp256k1::new(), self.network)?;

        let signer = Arc::new(SignersContainer::build(
            keymap,
            &descriptor,
            &Secp256k1::new(),
        ));

        self.indexed_graph
            .index
            .add_keychain(keychain.clone(), descriptor);
        self.signers.insert(keychain, signer);

        Ok(())
    }

    pub fn descriptors(&self) -> Vec<String> {
        let descriptors = self
            .indexed_graph
            .index
            .keychains()
            .iter()
            .map(|(_, descriptor)| descriptor.to_string())
            .collect::<Vec<String>>();

        descriptors
    }

    pub fn increment_external_index(&mut self) -> Result<(), WalletError> {
        //map through the keys of the descriptor

        // let descriptors = self.descriptors();
        // let wallet_name = self.file_name.to_str().unwrap();

        // let import_requests = descriptors
        //     .iter()
        //     .map(|descriptor| ImportMultiRequest {
        //         timestamp: Timestamp::Now,
        //         descriptor: Some(&descriptor),
        //         range: Some((0, 10)),
        //         watchonly: Some(true),
        //         label: Some(wallet_name),
        //         ..Default::default()
        //     })
        //     .collect::<Vec<ImportMultiRequest>>();

        // let result = rpc.import_multi(
        //     &import_requests,
        //     Some(
        //         &(bitcoind::bitcoincore_rpc::json::ImportMultiOptions {
        //             rescan: Some(false),
        //         }),
        //     ),
        // );

        // println!("{:?}", result);

        // for r in result {
        //     println!("{:?}", r);
        // }

        let ((next_index, _), changeset) = self
            .indexed_graph
            .index
            .reveal_next_spk(&KeychainKind::External);

        println!("Next unused SPK: {:?}", next_index);

        println!("{:?}", changeset);

        self.indexed_graph.index.apply_changeset(changeset.clone());
        self.persist
            .stage(Changeset::from(indexed_tx_graph::ChangeSet::from(
                changeset,
            )));

        println!("{:?}", self.persist.staged());

        match self.persist.commit() {
            Ok(_) => {}
            Err(err) => println!("{:?}", err),
        }

        Ok(())
    }

    pub fn generate_to_address(&self) -> Result<(), WalletError> {
        let address = &self.get_address().unwrap();

        self.oracle.rpc.generate_to_address(1, address)?;

        println!("Generated 1 block to {:?}", address);

        Ok(())
    }

    pub fn send_to_address(&mut self, to_address: Address, amount: u64) -> Result<(), WalletError> {
        let utxos = self.get_utxos(KeychainKind::External).unwrap();

        let mut sum = 0;
        let mut outpoints = vec![];

        for (outpoint, txout) in utxos {
            if sum < amount {
                sum += txout.value;

                outpoints.push(outpoint);
            };
        }

        println!("sum: {:?}", sum);

        let index = &mut self.indexed_graph.index;

        let last_revealed_index = index
            .last_revealed_index(&KeychainKind::External)
            .unwrap_or(0);

        let ((next_external_index, from_address), external_changeset) =
            index.reveal_next_spk(&KeychainKind::External);

        println!("Next unused SPK: {:?}", next_external_index);
        println!("from_address: {:?}", from_address);
        println!("{:?}", external_changeset);

        let ((next_internal_index, change_address), internal_changeset) =
            index.reveal_next_spk(&KeychainKind::Internal);

        println!("Next unused SPK: {:?}", next_internal_index);
        println!("change_address: {:?}", change_address);
        println!("{:?}", internal_changeset);

        let mut inputs = vec![];

        for outpoint in outpoints {
            let input = TxIn {
                previous_output: outpoint,
                script_sig: Default::default(),
                sequence: bitcoin::Sequence::max_value(),
                witness: Witness::default(),
            };

            inputs.push(input);
        }

        let mut outputs = vec![];

        let send_output = TxOut {
            value: amount,
            script_pubkey: to_address.script_pubkey(),
        };

        let change_output = TxOut {
            value: (sum - amount),
            script_pubkey: change_address.to_owned(),
        };

        outputs.push(send_output);
        outputs.push(change_output);

        let tx = Transaction {
            version: 2,
            lock_time: bitcoin::absolute::LockTime::Blocks(Height::min_value()),
            input: inputs,
            output: outputs,
        };

        // signer.signers().get
        if let Some(external_signer_container) = self.signers.get(&KeychainKind::External) {
            let secp = Secp256k1::new();
            let external_signers = external_signer_container.signers();
            for external_signer in external_signers {
                let mut psbt = Psbt::from_unsigned_tx(tx.clone())?;
                // psbt.inputs[0].bip32_derivation = external_signer.
                let descriptor = self
                    .indexed_graph
                    .index
                    .keychains()
                    .get(&KeychainKind::External)
                    .unwrap();

                let derived_descriptor = descriptor.at_derivation_index(last_revealed_index)?;

                println!("{:?}", psbt.inputs.get_mut(0).unwrap().witness_utxo);
                println!("{:?}", psbt.inputs.get_mut(0).unwrap().non_witness_utxo);

                psbt.update_input_with_descriptor(0, &derived_descriptor)?;
                // let secret = external_signer.descriptor_secret_key().unwrap();

                // psbt.sign(&secret, &secp);

                external_signer.sign_transaction(&mut psbt, &SignOptions::default(), &secp)?;

                let finalized_psbt = psbt.finalize(&secp)?;

                println!("final: {:?}", finalized_psbt);
            }
        }

        println!("{:#?}", tx);

        Ok(())
    }

    pub fn get_address(&self) -> Result<Address, WalletError> {
        let index = &self.indexed_graph.index;

        // let ((_, script), _) = index.next_unused_spk(&KeychainKind::External);
        let last_revealed_index = index
            .last_revealed_index(&KeychainKind::External)
            .unwrap_or(0);

        let address = Address::from_script(
            index
                .spk_at_index(KeychainKind::External, last_revealed_index)
                .unwrap(),
            self.network,
        )?;

        // let from_address = Address::from_script(&script, self.network).unwrap();

        Ok(address)
    }

    pub fn insert_block_relevant(
        &mut self,
        block_hash: &BlockHash,
        rpc: &Client,
    ) -> Result<Option<Changeset>, WalletError> {
        let block = rpc.get_block(block_hash)?;

        let block_info = rpc.get_block_info(block_hash)?;
        let height = block_info.height as u32;

        let changeset = self.indexed_graph.apply_block_relevant(&block, height);

        self.indexed_graph.apply_changeset(changeset.clone());

        self.persist
            .stage_and_commit(Changeset::try_from(changeset)?)
    }

    pub fn get_utxos(
        &mut self,
        keychain: KeychainKind,
    ) -> Result<Vec<(OutPoint, &TxOut)>, WalletError> {
        let tx_graph = self.indexed_graph.graph();

        let chain_tip = self.oracle.get_chain_tip().unwrap();

        let all_utxos = tx_graph
            .all_txouts()
            .filter(|(outpoint, _)| {
                match tx_graph.try_get_chain_spend::<Oracle>(&self.oracle, chain_tip, *outpoint) {
                    Ok(Some(_)) => false,
                    _ => true,
                }
            })
            .collect::<Vec<(OutPoint, &TxOut)>>();

        let revealed_spks = (0..=self
            .indexed_graph
            .index
            .last_revealed_index(&keychain)
            .unwrap_or(0))
            .map(|index| {
                self.indexed_graph
                    .index
                    .spk_at_index(keychain.clone(), index)
            })
            .filter(|spk| spk.is_some())
            .map(|spk| spk.unwrap())
            .collect::<Vec<&Script>>();

        println!("revealed_spks: {:#?}", revealed_spks);
        println!("all_utxos: {:#?}", all_utxos);

        let utxos = all_utxos
            .into_iter()
            .filter(|utxo| revealed_spks.contains(&utxo.1.script_pubkey.as_script()))
            .collect::<Vec<(OutPoint, &TxOut)>>();

        println!("utxos: {:?}", utxos);

        Ok(utxos)
    }
}

pub struct KeychainStore {
    pub file_name: PathBuf,
    path: PathBuf,
}

impl PersistBackend<Changeset> for KeychainStore {
    type LoadError = WalletError;
    type WriteError = WalletError;

    fn write_changes(&mut self, changeset: &Changeset) -> Result<(), Self::WriteError> {
        let path = self.path.join(&self.file_name);

        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        let wallet_file = OpenOptions::new().write(true).create(true).open(&path)?;

        let writer = BufWriter::new(wallet_file);
        Ok(serde_cbor::to_writer(writer, &changeset)?)
    }

    fn load_from_persistence(&mut self) -> Result<Option<Changeset>, Self::LoadError> {
        //check if the file exists, if not then create it

        if !self.path.join(&self.file_name).exists() {
            return Ok(None);
        }

        let wallet_file = OpenOptions::new()
            .read(true)
            .open(&self.path.join(&self.file_name))?;
        let reader = BufReader::new(wallet_file);
        let changeset: Changeset = serde_cbor::from_reader(reader)?;
        Ok(Some(changeset))
    }
}
