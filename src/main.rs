extern crate crypto;

mod hash;
mod tree;
mod merkle;

use std::collections::HashMap;
use hash::*;

fn main() {
    let txs = vec!["transaction1", "transaction2", "transaction3"];
    let txs: HashMap<_,_> = txs.into_iter().map(|tx| (hash(tx), tx)).collect();

    println!("{:?}", txs);

    tree::test();
}
