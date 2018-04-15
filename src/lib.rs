#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate crypto;

mod hash;
pub mod merkle;

#[cfg(test)]
mod tests {
    use merkle::Merkle;
    use std::iter::FromIterator;

    #[test]
    fn merkle_tree_is_valid_for_any_size() {
        for n in 0..100 {
            let range = (1..n + 1).collect::<Vec<usize>>();
            let txs = range.iter().map(|i| format!("tx{}", i));
            let merkle = Merkle::from_iter(txs);
            assert!(merkle.verify_tree());
        }
    }

    #[test]
    fn it_works() { //todo
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
}
