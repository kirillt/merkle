extern crate crypto;

mod hash;
mod tree;
mod merkle;

use hash::hash;
use merkle::Merkle;
use std::iter::FromIterator;

fn main() {
    let merkle_tree = Merkle::from_iter(
        vec!["tx1", "tx2", "tx3",
             "tx4", "tx5", "tx6",
             "tx7"]);

    println!("Tree: {:?}", merkle_tree.tree);
    println!("Data: {:?}", merkle_tree.data);

    let tx = "tx3";
    let key = hash(tx);
    let path = merkle_tree.path(&key);
    println!("Path of {} with key {}: {:?}", tx, key, path);
}
