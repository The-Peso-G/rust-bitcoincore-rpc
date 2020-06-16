//! # rust-bitcoincore-rpc integration test
//!
//! The test methods are named to mention the methods tested.
//! Individual test methods don't use any methods not tested before or
//! mentioned in the test method name.
//!
//! The goal of this test is not to test the correctness of the server, but
//! to test the serialization of arguments and deserialization of responses.
//!

#![deny(unused)]

extern crate bitcoin;
extern crate bitcoincore_rpc;
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use bitcoincore_rpc::json;
use bitcoincore_rpc::jsonrpc::error::Error as JsonRpcError;
use bitcoincore_rpc::{Auth, Client, Error, RpcApi};

use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::hashes::hex::{FromHex, ToHex};
use bitcoin::hashes::Hash;
use bitcoin::secp256k1;
use bitcoin::util::hash::BitcoinHash;
use bitcoin::{
    Address, Amount, Network, OutPoint, PrivateKey, Script, SigHashType, SignedAmount, Transaction,
    TxIn, TxOut, Txid,
};

lazy_static! {
    static ref SECP: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
    static ref NET: Network = Network::Regtest;
    /// A random address not owned by the node.
    static ref RANDOM_ADDRESS: Address = "mgR9fN5UzZ64mSUUtk6NwxxS6kwVfoEtPG".parse().unwrap();
    /// The default fee amount to use when needed.
    static ref FEE: Amount = Amount::from_btc(0.001).unwrap();
}

/// Assert that the call returns a "deprecated" error.
macro_rules! assert_deprecated {
	($call:expr) => {
		match $call.unwrap_err() {
            Error::JsonRpc(JsonRpcError::Rpc(e)) if e.code == -32 => {}
            e => panic!("expected deprecated error for {}, got: {}", stringify!($call), e),
		}
	}
}

/// Assert that the call returns a "method not found" error.
macro_rules! assert_not_found {
	($call:expr) => {
		match $call.unwrap_err() {
			Error::JsonRpc(JsonRpcError::Rpc(e)) if e.code == -32601 => {}
            e => panic!("expected method not found error for {}, got: {}", stringify!($call), e),
		}
	}
}

static mut VERSION: usize = 0;
/// Get the version of the node that is running.
fn version() -> usize {
    unsafe { VERSION }
}

/// Quickly create a BTC amount.
fn btc<F: Into<f64>>(btc: F) -> Amount {
    Amount::from_btc(btc.into()).unwrap()
}
/// Quickly create a signed BTC amount.
fn sbtc<F: Into<f64>>(btc: F) -> SignedAmount {
    SignedAmount::from_btc(btc.into()).unwrap()
}

fn main() {
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set");
    let auth = if let Ok(cookie) = std::env::var("RPC_COOKIE") {
        Auth::CookieFile(cookie.into())
    } else if let Ok(user) = std::env::var("RPC_USER") {
        Auth::UserPass(user, std::env::var("RPC_PASS").unwrap_or_default())
    } else {
        panic!("Either RPC_COOKIE or RPC_USER + RPC_PASS must be set.");
    };

    let cl = Client::new(rpc_url, auth).unwrap();

    test_get_network_info(&cl);
    unsafe { VERSION = cl.version().unwrap() };
    println!("Version: {}", version());

    test_get_mining_info(&cl);
    test_get_blockchain_info(&cl);
    test_get_new_address(&cl);
    test_dump_private_key(&cl);
    test_generate(&cl);
    test_get_balance_generate_to_address(&cl);
    test_get_best_block_hash(&cl);
    test_get_block_count(&cl);
    test_get_block_hash(&cl);
    test_get_block(&cl);
    test_get_block_header_get_block_header_info(&cl);
    test_get_address_info(&cl);
    test_set_label(&cl);
    test_send_to_address(&cl);
    test_get_received_by_address(&cl);
    test_list_unspent(&cl);
    test_get_difficulty(&cl);
    test_get_connection_count(&cl);
    test_get_raw_transaction(&cl);
    test_get_raw_mempool(&cl);
    test_get_transaction(&cl);
    test_list_transactions(&cl);
    test_get_tx_out(&cl);
    test_get_tx_out_proof(&cl);
    test_lock_unspent_unlock_unspent(&cl);
    test_get_block_filter(&cl);
    test_sign_raw_transaction_with_send_raw_transaction(&cl);
    test_invalidate_block_reconsider_block(&cl);
    test_key_pool_refill(&cl);
    test_create_raw_transaction(&cl);
    test_fund_raw_transaction(&cl);
    test_test_mempool_accept(&cl);
    test_wallet_create_funded_psbt(&cl);
    test_combine_psbt(&cl);
    test_finalize_psbt(&cl);
    test_list_received_by_address(&cl);
    test_import_public_key(&cl);
    test_import_priv_key(&cl);
    test_import_address(&cl);
    test_import_address_script(&cl);
    test_estimate_smart_fee(&cl);
    test_ping(&cl);
    test_get_peer_info(&cl);
    test_rescan_blockchain(&cl);
    //TODO import_multi(
    //TODO verify_message(
    //TODO wait_for_new_block(&self, timeout: u64) -> Result<json::BlockRef> {
    //TODO wait_for_block(
    //TODO get_descriptor_info(&self, desc: &str) -> Result<json::GetDescriptorInfoResult> {
    //TODO derive_addresses(&self, descriptor: &str, range: Option<[u32; 2]>) -> Result<Vec<Address>> {
    //TODO encrypt_wallet(&self, passphrase: &str) -> Result<()> {
    //TODO get_by_id<T: queryable::Queryable<Self>>(
    //TODO add_multisig_address(
    //TODO load_wallet(&self, wallet: &str) -> Result<json::LoadWalletResult> {
    //TODO unload_wallet(&self, wallet: Option<&str>) -> Result<()> {
    //TODO create_wallet(
    //TODO backup_wallet(&self, destination: Option<&str>) -> Result<()> {
    test_stop(cl);
}

fn test_get_network_info(cl: &Client) {
    let _ = cl.get_network_info().unwrap();
}

fn test_get_mining_info(cl: &Client) {
    let _ = cl.get_mining_info().unwrap();
}

fn test_get_blockchain_info(cl: &Client) {
    let info = cl.get_blockchain_info().unwrap();
    assert_eq!(&info.chain, "regtest");
}

fn test_get_new_address(cl: &Client) {
    let addr = cl.get_new_address(None, Some(json::AddressType::Legacy)).unwrap();
    assert_eq!(addr.address_type(), Some(bitcoin::AddressType::P2pkh));

    let addr = cl.get_new_address(None, Some(json::AddressType::Bech32)).unwrap();
    assert_eq!(addr.address_type(), Some(bitcoin::AddressType::P2wpkh));

    let addr = cl.get_new_address(None, Some(json::AddressType::P2shSegwit)).unwrap();
    assert_eq!(addr.address_type(), Some(bitcoin::AddressType::P2sh));
}

fn test_dump_private_key(cl: &Client) {
    let addr = cl.get_new_address(None, Some(json::AddressType::Bech32)).unwrap();
    let sk = cl.dump_private_key(&addr).unwrap();
    assert_eq!(addr, Address::p2wpkh(&sk.public_key(&SECP), *NET));
}

fn test_generate(cl: &Client) {
    if version() < 180000 {
        let blocks = cl.generate(4, None).unwrap();
        assert_eq!(blocks.len(), 4);
        let blocks = cl.generate(6, Some(45)).unwrap();
        assert_eq!(blocks.len(), 6);
    } else if version() < 190000 {
		assert_deprecated!(cl.generate(5, None));
    } else {
		assert_not_found!(cl.generate(5, None));
    }
}

fn test_get_balance_generate_to_address(cl: &Client) {
    let initial = cl.get_balance(None, None).unwrap();

    let blocks = cl.generate_to_address(500, &cl.get_new_address(None, None).unwrap()).unwrap();
    assert_eq!(blocks.len(), 500);
    assert_ne!(cl.get_balance(None, None).unwrap(), initial);
}

fn test_get_best_block_hash(cl: &Client) {
    let _ = cl.get_best_block_hash().unwrap();
}

fn test_get_block_count(cl: &Client) {
    let height = cl.get_block_count().unwrap();
    assert!(height > 0);
}

fn test_get_block_hash(cl: &Client) {
    let h = cl.get_block_count().unwrap();
    assert_eq!(cl.get_block_hash(h).unwrap(), cl.get_best_block_hash().unwrap());
}

fn test_get_block(cl: &Client) {
    let tip = cl.get_best_block_hash().unwrap();
    let block = cl.get_block(&tip).unwrap();
    let hex = cl.get_block_hex(&tip).unwrap();
    assert_eq!(block, deserialize(&Vec::<u8>::from_hex(&hex).unwrap()).unwrap());
    assert_eq!(hex, serialize(&block).to_hex());

    let tip = cl.get_best_block_hash().unwrap();
    let info = cl.get_block_info(&tip).unwrap();
    assert_eq!(info.hash, tip);
    assert_eq!(info.confirmations, 1);
}

fn test_get_block_header_get_block_header_info(cl: &Client) {
    let tip = cl.get_best_block_hash().unwrap();
    let header = cl.get_block_header(&tip).unwrap();
    let info = cl.get_block_header_info(&tip).unwrap();
    assert_eq!(header.bitcoin_hash(), info.hash);
    assert_eq!(header.version, info.version);
    assert_eq!(header.merkle_root, info.merkle_root);
    assert_eq!(info.confirmations, 1);
    assert_eq!(info.next_block_hash, None);
    assert!(info.previous_block_hash.is_some());
}

fn test_get_address_info(cl: &Client) {
    let addr = cl.get_new_address(None, Some(json::AddressType::Legacy)).unwrap();
    let info = cl.get_address_info(&addr).unwrap();
    assert!(!info.is_witness.unwrap());

    let addr = cl.get_new_address(None, Some(json::AddressType::Bech32)).unwrap();
    let info = cl.get_address_info(&addr).unwrap();
    assert!(!info.witness_program.unwrap().is_empty());

    let addr = cl.get_new_address(None, Some(json::AddressType::P2shSegwit)).unwrap();
    let info = cl.get_address_info(&addr).unwrap();
    assert!(!info.hex.unwrap().is_empty());
}

fn test_set_label(cl: &Client) {
    let addr = cl.get_new_address(Some("label"), None).unwrap();
    let info = cl.get_address_info(&addr).unwrap();
    assert_eq!(&info.label, "label");
    assert_eq!(info.labels[0].name, "label");

    cl.set_label(&addr, "other").unwrap();
    let info = cl.get_address_info(&addr).unwrap();
    assert_eq!(&info.label, "other");
    assert_eq!(info.labels[0].name, "other");
}

fn test_send_to_address(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let est = json::EstimateMode::Conservative;
    let _ = cl.send_to_address(&addr, btc(1), Some("cc"), None, None, None, None, None).unwrap();
    let _ = cl.send_to_address(&addr, btc(1), None, Some("tt"), None, None, None, None).unwrap();
    let _ = cl.send_to_address(&addr, btc(1), None, None, Some(true), None, None, None).unwrap();
    let _ = cl.send_to_address(&addr, btc(1), None, None, None, Some(true), None, None).unwrap();
    let _ = cl.send_to_address(&addr, btc(1), None, None, None, None, Some(3), None).unwrap();
    let _ = cl.send_to_address(&addr, btc(1), None, None, None, None, None, Some(est)).unwrap();
}

fn test_get_received_by_address(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let _ = cl.send_to_address(&addr, btc(1), None, None, None, None, None, None).unwrap();
    assert_eq!(cl.get_received_by_address(&addr, Some(0)).unwrap(), btc(1));
    assert_eq!(cl.get_received_by_address(&addr, Some(1)).unwrap(), btc(0));
    let _ = cl.generate_to_address(7, &cl.get_new_address(None, None).unwrap()).unwrap();
    assert_eq!(cl.get_received_by_address(&addr, Some(6)).unwrap(), btc(1));
    assert_eq!(cl.get_received_by_address(&addr, None).unwrap(), btc(1));
}

fn test_list_unspent(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let txid = cl.send_to_address(&addr, btc(1), None, None, None, None, None, None).unwrap();
    let unspent = cl.list_unspent(Some(0), None, Some(&[&addr]), None, None).unwrap();
    assert_eq!(unspent[0].txid, txid);
    assert_eq!(unspent[0].address.as_ref(), Some(&addr));
    assert_eq!(unspent[0].amount, btc(1));

    let txid = cl.send_to_address(&addr, btc(7), None, None, None, None, None, None).unwrap();
    let options = json::ListUnspentQueryOptions {
        minimum_amount: Some(btc(7)),
        maximum_amount: Some(btc(7)),
        ..Default::default()
    };
    let unspent = cl.list_unspent(Some(0), None, Some(&[&addr]), None, Some(options)).unwrap();
    assert_eq!(unspent.len(), 1);
    assert_eq!(unspent[0].txid, txid);
    assert_eq!(unspent[0].address.as_ref(), Some(&addr));
    assert_eq!(unspent[0].amount, btc(7));
}

fn test_get_difficulty(cl: &Client) {
    let _ = cl.get_difficulty().unwrap();
}

fn test_get_connection_count(cl: &Client) {
    let _ = cl.get_connection_count().unwrap();
}

fn test_get_raw_transaction(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let txid = cl.send_to_address(&addr, btc(1), None, None, None, None, None, None).unwrap();
    let tx = cl.get_raw_transaction(&txid, None).unwrap();
    let hex = cl.get_raw_transaction_hex(&txid, None).unwrap();
    assert_eq!(tx, deserialize(&Vec::<u8>::from_hex(&hex).unwrap()).unwrap());
    assert_eq!(hex, serialize(&tx).to_hex());

    let info = cl.get_raw_transaction_info(&txid, None).unwrap();
    assert_eq!(info.txid, txid);

    let blocks = cl.generate_to_address(7, &cl.get_new_address(None, None).unwrap()).unwrap();
    let _ = cl.get_raw_transaction_info(&txid, Some(&blocks[0])).unwrap();
}

fn test_get_raw_mempool(cl: &Client) {
    let _ = cl.get_raw_mempool().unwrap();
}

fn test_get_transaction(cl: &Client) {
    let txid =
        cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();
    let tx = cl.get_transaction(&txid, None).unwrap();
    assert_eq!(tx.amount, sbtc(-1.0));
    assert_eq!(tx.info.txid, txid);

    let fake = Txid::hash(&[1, 2]);
    assert!(cl.get_transaction(&fake, Some(true)).is_err());
}

fn test_list_transactions(cl: &Client) {
    let _ = cl.list_transactions(None, None, None, None).unwrap();
    let _ = cl.list_transactions(Some("l"), None, None, None).unwrap();
    let _ = cl.list_transactions(None, Some(3), None, None).unwrap();
    let _ = cl.list_transactions(None, None, Some(3), None).unwrap();
    let _ = cl.list_transactions(None, None, None, Some(true)).unwrap();
}

fn test_get_tx_out(cl: &Client) {
    let txid =
        cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();
    let out = cl.get_tx_out(&txid, 0, Some(false)).unwrap();
    assert!(out.is_none());
    let out = cl.get_tx_out(&txid, 0, Some(true)).unwrap();
    assert!(out.is_some());
    let _ = cl.get_tx_out(&txid, 0, None).unwrap();
}

fn test_get_tx_out_proof(cl: &Client) {
    let txid1 =
        cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();
    let txid2 =
        cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();
    let blocks = cl.generate_to_address(7, &cl.get_new_address(None, None).unwrap()).unwrap();
    let proof = cl.get_tx_out_proof(&[txid1, txid2], Some(&blocks[0])).unwrap();
    assert!(!proof.is_empty());
}

fn test_lock_unspent_unlock_unspent(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let txid = cl.send_to_address(&addr, btc(1), None, None, None, None, None, None).unwrap();

    assert!(cl.lock_unspent(&[OutPoint::new(txid, 0)]).unwrap());
    assert!(cl.unlock_unspent(&[OutPoint::new(txid, 0)]).unwrap());
}

fn test_get_block_filter(cl: &Client) {
    let blocks = cl.generate_to_address(7, &cl.get_new_address(None, None).unwrap()).unwrap();
    if version() >= 190000 {
		let _ = cl.get_block_filter(&blocks[0]).unwrap();
    } else {
		assert_not_found!(cl.get_block_filter(&blocks[0]));
	}
}

fn test_sign_raw_transaction_with_send_raw_transaction(cl: &Client) {
    let sk = PrivateKey {
        network: Network::Regtest,
        key: secp256k1::SecretKey::new(&mut secp256k1::rand::thread_rng()),
        compressed: true,
    };
    let addr = Address::p2wpkh(&sk.public_key(&SECP), Network::Regtest);

    let options = json::ListUnspentQueryOptions {
        minimum_amount: Some(btc(2)),
        ..Default::default()
    };
    let unspent = cl.list_unspent(Some(6), None, None, None, Some(options)).unwrap();
    let unspent = unspent.into_iter().nth(0).unwrap();

    let tx = Transaction {
        version: 1,
        lock_time: 0,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: unspent.txid,
                vout: unspent.vout,
            },
            sequence: 0xFFFFFFFF,
            script_sig: Script::new(),
            witness: Vec::new(),
        }],
        output: vec![TxOut {
            value: (unspent.amount - *FEE).as_sat(),
            script_pubkey: addr.script_pubkey(),
        }],
    };

    let input = json::SignRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        script_pub_key: unspent.script_pub_key,
        redeem_script: None,
        amount: Some(unspent.amount),
    };
    let res = cl.sign_raw_transaction_with_wallet(&tx, Some(&[input]), None).unwrap();
    assert!(res.complete);
    let txid = cl.send_raw_transaction(&res.transaction().unwrap()).unwrap();

    let tx = Transaction {
        version: 1,
        lock_time: 0,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: txid,
                vout: 0,
            },
            script_sig: Script::new(),
            sequence: 0xFFFFFFFF,
            witness: Vec::new(),
        }],
        output: vec![TxOut {
            value: (unspent.amount - *FEE - *FEE).as_sat(),
            script_pubkey: RANDOM_ADDRESS.script_pubkey(),
        }],
    };

    let res =
        cl.sign_raw_transaction_with_key(&tx, &[sk], None, Some(SigHashType::All.into())).unwrap();
    assert!(res.complete);
    let _ = cl.send_raw_transaction(&res.transaction().unwrap()).unwrap();
}

fn test_invalidate_block_reconsider_block(cl: &Client) {
    let hash = cl.get_best_block_hash().unwrap();
    cl.invalidate_block(&hash).unwrap();
    cl.reconsider_block(&hash).unwrap();
}

fn test_key_pool_refill(cl: &Client) {
    cl.key_pool_refill(Some(100)).unwrap();
    cl.key_pool_refill(None).unwrap();
}

fn test_create_raw_transaction(cl: &Client) {
    let options = json::ListUnspentQueryOptions {
        minimum_amount: Some(btc(2)),
        ..Default::default()
    };
    let unspent = cl.list_unspent(Some(6), None, None, None, Some(options)).unwrap();
    let unspent = unspent.into_iter().nth(0).unwrap();

    let input = json::CreateRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        sequence: None,
    };
    let mut output = HashMap::new();
    output.insert(RANDOM_ADDRESS.to_string(), btc(1));

    let tx =
        cl.create_raw_transaction(&[input.clone()], &output, Some(500_000), Some(true)).unwrap();
    let hex = cl.create_raw_transaction_hex(&[input], &output, Some(500_000), Some(true)).unwrap();
    assert_eq!(tx, deserialize(&Vec::<u8>::from_hex(&hex).unwrap()).unwrap());
    assert_eq!(hex, serialize(&tx).to_hex());
}

fn test_fund_raw_transaction(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let mut output = HashMap::new();
    output.insert(RANDOM_ADDRESS.to_string(), btc(1));

    let options = json::FundRawTransactionOptions {
        change_address: Some(addr),
        change_position: Some(0),
        change_type: None,
        include_watching: Some(true),
        lock_unspents: Some(true),
        fee_rate: Some(*FEE),
        subtract_fee_from_outputs: Some(vec![0]),
        replaceable: Some(true),
        conf_target: None,
        estimate_mode: None,
    };
    let tx = cl.create_raw_transaction_hex(&[], &output, Some(500_000), Some(true)).unwrap();
    let funded = cl.fund_raw_transaction(tx, Some(&options), Some(false)).unwrap();
    let _ = funded.transaction().unwrap();

    let options = json::FundRawTransactionOptions {
        change_address: None,
        change_position: Some(0),
        change_type: Some(json::AddressType::Legacy),
        include_watching: Some(true),
        lock_unspents: Some(true),
        fee_rate: None,
        subtract_fee_from_outputs: Some(vec![0]),
        replaceable: Some(true),
        conf_target: Some(2),
        estimate_mode: Some(json::EstimateMode::Conservative),
    };
    let tx = cl.create_raw_transaction_hex(&[], &output, Some(500_000), Some(true)).unwrap();
    let funded = cl.fund_raw_transaction(tx, Some(&options), Some(false)).unwrap();
    let _ = funded.transaction().unwrap();
}

fn test_test_mempool_accept(cl: &Client) {
    let options = json::ListUnspentQueryOptions {
        minimum_amount: Some(btc(2)),
        ..Default::default()
    };
    let unspent = cl.list_unspent(Some(6), None, None, None, Some(options)).unwrap();
    let unspent = unspent.into_iter().nth(0).unwrap();

    let input = json::CreateRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        sequence: Some(0xFFFFFFFF),
    };
    let mut output = HashMap::new();
    output.insert(RANDOM_ADDRESS.to_string(), unspent.amount - *FEE);

    let tx =
        cl.create_raw_transaction(&[input.clone()], &output, Some(500_000), Some(false)).unwrap();
    let res = cl.test_mempool_accept(&[&tx]).unwrap();
    assert!(!res[0].allowed);
    assert!(res[0].reject_reason.is_some());
    let signed =
        cl.sign_raw_transaction_with_wallet(&tx, None, None).unwrap().transaction().unwrap();
    let res = cl.test_mempool_accept(&[&signed]).unwrap();
    assert!(res[0].allowed, "not allowed: {:?}", res[0].reject_reason);
}

fn test_wallet_create_funded_psbt(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let options = json::ListUnspentQueryOptions {
        minimum_amount: Some(btc(2)),
        ..Default::default()
    };
    let unspent = cl.list_unspent(Some(6), None, None, None, Some(options)).unwrap();
    let unspent = unspent.into_iter().nth(0).unwrap();

    let input = json::CreateRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        sequence: None,
    };
    let mut output = HashMap::new();
    output.insert(RANDOM_ADDRESS.to_string(), btc(1));

    let options = json::WalletCreateFundedPsbtOptions {
        change_address: None,
        change_position: Some(1),
        change_type: Some(json::AddressType::Legacy),
        include_watching: Some(true),
        lock_unspent: Some(true),
        fee_rate: Some(*FEE),
        subtract_fee_from_outputs: vec![0],
        replaceable: Some(true),
        conf_target: None,
        estimate_mode: None,
    };
    let _ = cl
        .wallet_create_funded_psbt(
            &[input.clone()],
            &output,
            Some(500_000),
            Some(options),
            Some(true),
        )
        .unwrap();

    let options = json::WalletCreateFundedPsbtOptions {
        change_address: Some(addr),
        change_position: Some(1),
        change_type: None,
        include_watching: Some(true),
        lock_unspent: Some(true),
        fee_rate: None,
        subtract_fee_from_outputs: vec![0],
        replaceable: Some(true),
        conf_target: Some(3),
        estimate_mode: Some(json::EstimateMode::Conservative),
    };
    let psbt = cl
        .wallet_create_funded_psbt(&[input], &output, Some(500_000), Some(options), Some(true))
        .unwrap();
    assert!(!psbt.psbt.is_empty());
}

fn test_combine_psbt(cl: &Client) {
    let options = json::ListUnspentQueryOptions {
        minimum_amount: Some(btc(2)),
        ..Default::default()
    };
    let unspent = cl.list_unspent(Some(6), None, None, None, Some(options)).unwrap();
    let unspent = unspent.into_iter().nth(0).unwrap();
    let input = json::CreateRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        sequence: None,
    };
    let mut output = HashMap::new();
    output.insert(RANDOM_ADDRESS.to_string(), btc(1));
    let psbt1 = cl
        .wallet_create_funded_psbt(&[input.clone()], &output, Some(500_000), None, Some(true))
        .unwrap();

    let psbt = cl.combine_psbt(&[psbt1.psbt.clone(), psbt1.psbt]).unwrap();
    assert!(!psbt.is_empty());
}

fn test_finalize_psbt(cl: &Client) {
    let options = json::ListUnspentQueryOptions {
        minimum_amount: Some(btc(2)),
        ..Default::default()
    };
    let unspent = cl.list_unspent(Some(6), None, None, None, Some(options)).unwrap();
    let unspent = unspent.into_iter().nth(0).unwrap();
    let input = json::CreateRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        sequence: None,
    };
    let mut output = HashMap::new();
    output.insert(RANDOM_ADDRESS.to_string(), btc(1));
    let psbt = cl
        .wallet_create_funded_psbt(&[input.clone()], &output, Some(500_000), None, Some(true))
        .unwrap();

    let res = cl.finalize_psbt(&psbt.psbt, Some(true)).unwrap();
    assert!(!res.complete);
    //TODO(stevenroose) add sign psbt and test hex field
    //assert!(res.hex.is_some());
}

fn test_list_received_by_address(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let txid = cl.send_to_address(&addr, btc(1), None, None, None, None, None, None).unwrap();

    let _ = cl.list_received_by_address(Some(&addr), None, None, None).unwrap();
    let _ = cl.list_received_by_address(Some(&addr), None, Some(true), None).unwrap();
    let _ = cl.list_received_by_address(Some(&addr), None, None, Some(true)).unwrap();
    let _ = cl.list_received_by_address(None, Some(200), None, None).unwrap();

    let res = cl.list_received_by_address(Some(&addr), Some(0), None, None).unwrap();
    assert_eq!(res[0].txids, vec![txid]);
}

fn test_import_public_key(cl: &Client) {
    let sk = PrivateKey {
        network: Network::Regtest,
        key: secp256k1::SecretKey::new(&mut secp256k1::rand::thread_rng()),
        compressed: true,
    };
    cl.import_public_key(&sk.public_key(&SECP), None, None).unwrap();
    cl.import_public_key(&sk.public_key(&SECP), Some("l"), None).unwrap();
    cl.import_public_key(&sk.public_key(&SECP), None, Some(false)).unwrap();
}

fn test_import_priv_key(cl: &Client) {
    let sk = PrivateKey {
        network: Network::Regtest,
        key: secp256k1::SecretKey::new(&mut secp256k1::rand::thread_rng()),
        compressed: true,
    };
    cl.import_private_key(&sk, None, None).unwrap();
    cl.import_private_key(&sk, Some("l"), None).unwrap();
    cl.import_private_key(&sk, None, Some(false)).unwrap();
}

fn test_import_address(cl: &Client) {
    let sk = PrivateKey {
        network: Network::Regtest,
        key: secp256k1::SecretKey::new(&mut secp256k1::rand::thread_rng()),
        compressed: true,
    };
    let addr = Address::p2pkh(&sk.public_key(&SECP), Network::Regtest);
    cl.import_address(&addr, None, None).unwrap();
    cl.import_address(&addr, Some("l"), None).unwrap();
    cl.import_address(&addr, None, Some(false)).unwrap();
}

fn test_import_address_script(cl: &Client) {
    let sk = PrivateKey {
        network: Network::Regtest,
        key: secp256k1::SecretKey::new(&mut secp256k1::rand::thread_rng()),
        compressed: true,
    };
    let addr = Address::p2pkh(&sk.public_key(&SECP), Network::Regtest);
    cl.import_address_script(&addr.script_pubkey(), None, None, None).unwrap();
    cl.import_address_script(&addr.script_pubkey(), Some("l"), None, None).unwrap();
    cl.import_address_script(&addr.script_pubkey(), None, Some(false), None).unwrap();
    cl.import_address_script(&addr.script_pubkey(), None, None, Some(true)).unwrap();
}

fn test_estimate_smart_fee(cl: &Client) {
    let mode = json::EstimateMode::Unset;
    let res = cl.estimate_smart_fee(3, Some(mode)).unwrap();

    // With a fresh node, we can't get fee estimates.
    if let Some(errors) = res.errors {
        if errors == &["Insufficient data or no feerate found"] {
            println!("Cannot test estimate_smart_fee because no feerate found!");
            return;
        } else {
            panic!("Unexpected error(s) for estimate_smart_fee: {:?}", errors);
        }
    }

    assert!(res.fee_rate.is_some(), "no fee estimate available: {:?}", res.errors);
    assert!(res.fee_rate.unwrap() >= btc(0));
}

fn test_ping(cl: &Client) {
    let _ = cl.ping().unwrap();
}

fn test_get_peer_info(cl: &Client) {
    let info = cl.get_peer_info().unwrap();
    if info.is_empty() {
        panic!("No peers are connected so we can't test get_peer_info");
    }
}

fn test_rescan_blockchain(cl: &Client) {
    let count = cl.get_block_count().unwrap() as usize;
    assert!(count > 21);
    let (start, stop) = cl.rescan_blockchain(Some(count - 20), Some(count - 1)).unwrap();
    assert_eq!(start, count - 20);
    assert_eq!(stop, Some(count - 1));
}

fn test_stop(cl: Client) {
    println!("Stopping: '{}'", cl.stop().unwrap());
}
