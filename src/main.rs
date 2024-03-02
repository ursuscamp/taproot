use std::env::{args, Args};

use bitcoin::{
    absolute::LockTime,
    address::{NetworkChecked, NetworkUnchecked},
    opcodes::{
        all::{OP_ADD, OP_EQUAL, OP_PUSHNUM_1, OP_PUSHNUM_3},
        OP_TRUE,
    },
    script,
    taproot::{LeafVersion, TaprootBuilder, TaprootSpendInfo},
    transaction::Version,
    Address, Amount, Network, OutPoint, Script, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
    Txid, Witness, XOnlyPublicKey,
};
use secp256k1::SECP256K1;
use sha2::Digest;

static NUMS_INPUT: &str = "hello world";

fn main() {
    let nums_point = nums_point();
    println!("NUMS Point: {nums_point}");
    let tsi = build_taproot(nums_point);
    // println!("{tsi:#?}");

    let address = Address::p2tr(SECP256K1, nums_point, tsi.merkle_root(), Network::Regtest);
    println!("Send 1 BTC to: {address}");

    let mut args = args();
    args.next();

    let txid: Txid = args
        .next()
        .expect("Missing txid for spend")
        .parse()
        .expect("invalid txid");

    let vout: u32 = args
        .next()
        .expect("missing vout for spend")
        .parse()
        .expect("invalid vout");

    let next_address: Address<NetworkUnchecked> = args
        .next()
        .expect("missing address")
        .parse()
        .expect("invalid address");

    let spend_tx = make_spend(tsi, txid, vout, next_address.assume_checked());

    println!(
        "Spend TX: {}",
        hex::encode(bitcoin::consensus::serialize(&spend_tx))
    );
}

/// Calculate the NUMS point for a keyless taproot spend.
fn nums_point() -> XOnlyPublicKey {
    let mut hashed = sha2::Sha256::digest(NUMS_INPUT.as_bytes());
    let mut pk = XOnlyPublicKey::from_slice(hashed.as_slice()).ok();

    while pk.is_none() {
        hashed = sha2::Sha256::digest(hashed.as_slice());
        pk = XOnlyPublicKey::from_slice(hashed.as_slice()).ok();
    }
    pk.unwrap()
}

fn locking_script() -> ScriptBuf {
    script::Builder::new()
        .push_opcode(OP_PUSHNUM_1)
        .push_opcode(OP_ADD)
        .push_opcode(OP_PUSHNUM_3)
        .push_opcode(OP_EQUAL)
        .into_script()
}

fn build_taproot(internal_key: XOnlyPublicKey) -> TaprootSpendInfo {
    TaprootBuilder::new()
        .add_leaf(0, locking_script())
        .unwrap()
        .finalize(SECP256K1, internal_key)
        .unwrap()
}

fn make_spend(
    tsi: TaprootSpendInfo,
    txid: Txid,
    vout: u32,
    spend_address: Address<NetworkChecked>,
) -> Transaction {
    let script = locking_script();
    let control_block = tsi
        .control_block(&(script.clone(), LeafVersion::TapScript))
        .unwrap();

    let mut witness = Witness::new();
    witness.push([0x02]);
    witness.push(script.to_bytes());
    witness.push(control_block.serialize());

    Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint { txid, vout },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ZERO,
            witness,
        }],
        output: vec![TxOut {
            value: Amount::from_btc(1.0).unwrap() - Amount::from_sat(200),
            script_pubkey: spend_address.script_pubkey(),
        }],
    }
}
