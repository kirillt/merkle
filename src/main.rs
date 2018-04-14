#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate crypto;

mod hash;
mod merkle;

use merkle::Merkle;
use std::iter::FromIterator;

fn main() {
    let txs = vec!["tx1", "tx2", "tx3", "tx4", "tx5", "tx6", "tx7"];

    let merkle_tree = Merkle::from_iter(txs);

    println!("Data: {:?}", merkle_tree.data);
    println!("Tree: {:?}", merkle_tree.tree);
    println!("Verification passed? {}", merkle_tree.verify_tree());
    println!();

    for key in merkle_tree.data.keys() {
        let path = merkle_tree.path(key);
        println!("Path of {} is {:?}", key, path);
        println!(
            "Verification passed? {}",
            merkle_tree.verify_path(key, &path.unwrap())
        );
        println!();
    }
}
