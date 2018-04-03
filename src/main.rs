extern crate crypto;

use std::collections::HashMap;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

fn sha256(value: &str) -> String {
    let mut sha = Sha256::new();
    sha.input_str(value);
    sha.result_str()
}

fn hash(value: &str) -> String {
    sha256(&sha256(value))
}

fn main() {
    let txs = vec!["transaction1", "transaction2", "transaction3"];
    let txs: HashMap<_,_> = txs.into_iter().map(|value| (hash(value), value)).collect();

    println!("{:?}", txs);
}
