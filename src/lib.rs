#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![feature(test)]

extern crate crypto;
extern crate test;

pub mod merkle;
mod hash;

#[cfg(test)]
mod tests {
    extern crate rand;

    use hash::hash;
    use merkle::Merkle;
    use std::iter::FromIterator;

    use tests::rand::{Rng};

    #[test]
    fn create() {
        check_tree(&tree_with_n_leaves(7));
    }

    #[test]
    fn insert() {
        let mut merkle = tree_with_n_leaves(7);
        merkle.insert("tx1");
        merkle.insert("tx8");
        merkle.insert("tx9");
        merkle.insert("tx10");
        merkle.insert("tx11");
        check_tree(&merkle);
    }

    #[test]
    fn delete() {
        let mut merkle = tree_with_n_leaves(7);
        merkle.delete("tx11");
        merkle.delete("tx3");
        merkle.delete("tx7");
        merkle.delete("tx4");
        merkle.delete("tx1");
        check_tree(&merkle);
    }

    #[test]
    fn automated_test() {
        let mut rng = rand::thread_rng();

        for n in 0..50 {
            let mut merkle = tree_with_n_leaves(n);
            check_tree(&merkle);

            for _ in 0..50 {
                let tx = format!("tx{}", rng.gen_range(0, n + 1));
                let key = hash(&tx);
                match rng.gen_range(0, 2) {
                    0 => {
                        merkle.insert(&tx);
                        assert!(merkle.path(&key).is_some());
                        assert!(merkle.data.get(&key).is_some());
                    },
                    1 => {
                        merkle.delete(&key);
                        assert!(merkle.path(&key).is_none());
                        assert!(merkle.data.get(&key).is_none());
                    },
                    _ => panic!("unexpected action")
                }
                check_tree(&merkle);
            }
        }
    }

    fn check_tree(merkle: &Merkle) {
        assert!(merkle.total == 0 || merkle.total == merkle.leaves * 2 - 1);
        assert!(merkle.verify_tree());

        assert!(merkle.path("absent_key").is_none());
        for key in merkle.data.keys() {
            let path = merkle.path(key).unwrap();
            assert!(merkle.verify_path(key, &path));
        }
    }

    fn tree_with_n_leaves(n: usize) -> Merkle {
        let range = (1..n + 1).collect::<Vec<usize>>();
        let txs = range.iter().map(|i| format!("tx{}", i));
        Merkle::from_iter(txs)
    }

    use test::Bencher;

    #[bench]
    fn construction_10000(bench: &mut Bencher) {
        bench.iter(|| tree_with_n_leaves(10000));
    }

    #[bench]
    fn insert_10000_to_empty(bench: &mut Bencher) {
        let mut tree = tree_with_n_leaves(0);
        bench.iter(|| {
            for i in 1..10001 {
                tree.insert(&format!("tx{}", i));
            }
        })
    }

    #[bench]
    fn delete_10000_to_empty_asc(bench: &mut Bencher) {
        let mut tree = tree_with_n_leaves(10000);
        bench.iter(|| {
            for i in 1..10001 {
                tree.delete(&format!("tx{}", i));
            }
        })
    }

    #[bench]
    fn delete_10000_to_empty_desc(bench: &mut Bencher) {
        let mut tree = tree_with_n_leaves(10000);
        bench.iter(|| {
            for i in (1..10001).rev() {
                tree.delete(&format!("tx{}", i));
            }
        })
    }

    #[bench]
    fn proof_10000(bench: &mut Bencher) {
        let tree = tree_with_n_leaves(10000);
        bench.iter(|| tree.path("tx5000"));
    }
}
