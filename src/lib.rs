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
    fn automated_test() {
        for n in 0..100 {
            let merkle = tree_with_n_leaves(n);
            assert!(merkle.verify_tree());

            assert!(merkle.path("absent_key").is_none());
            for key in merkle.data.keys() {
                let path = merkle.path(key).unwrap();
                assert!(merkle.verify_path(key, &path));
            }
        }
    }

    fn tree_with_n_leaves(n: usize) -> Merkle {
        let range = (1..n + 1).collect::<Vec<usize>>();
        let txs = range.iter().map(|i| format!("tx{}", i));
        Merkle::from_iter(txs)
    }
}
