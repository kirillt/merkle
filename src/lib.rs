#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![feature(test)]

extern crate crypto;
extern crate test;

mod hash;
pub mod merkle;

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate rand;

    use hash::hash;
    use merkle::Merkle;
    use std::iter::FromIterator;

    use tests::rand::Rng;

    #[test]
    fn create() {
        check_tree(&tree_with_n_leaves(7));
    }

    #[test]
    fn insert_generic() {
        let mut merkle = tree_with_n_leaves(7);
        merkle.insert("tx1");
        merkle.insert("tx8");
        merkle.insert("tx9");
        merkle.insert("tx10");
        merkle.insert("tx11");
        check_tree(&merkle);
    }

    #[test]
    fn delete_odd_leaf() {
        let mut merkle = tree_with_n_leaves(7);

        let odd_leaf = merkle.tree[6].clone();
        merkle.delete(&odd_leaf);

        assert!(merkle.path(&odd_leaf).is_none());
        check_tree(&merkle);
    }

    #[test]
    fn delete_farthest_leaves() {
        let mut merkle = tree_with_n_leaves(7);

        let farthest_leaf_even = merkle.tree[12].clone();
        merkle.delete(&farthest_leaf_even);
        assert!(merkle.path(&farthest_leaf_even).is_none());
        check_tree(&merkle);

        let farthest_leaf_odd = merkle.tree[9].clone();
        merkle.delete(&farthest_leaf_odd);
        assert!(merkle.path(&farthest_leaf_odd).is_none());
        check_tree(&merkle);
    }

    #[test]
    fn delete_generic() {
        let mut merkle = tree_with_n_leaves(7);
        merkle.delete(&hash("tx11")); //non-existent
        merkle.delete(&hash("tx3"));
        merkle.delete(&hash("tx7"));
        merkle.delete(&hash("tx4"));
        merkle.delete(&hash("tx1"));
        assert_eq!(merkle.leaves, 3);
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
                    }
                    1 => {
                        merkle.delete(&key);
                        assert!(merkle.path(&key).is_none());
                        assert!(merkle.data.get(&key).is_none());
                    }
                    _ => panic!("unexpected action"),
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

        //for (key, i) in merkle.index.iter() {
        //    assert_eq!(merkle.tree.get(*i), Some(key));
        //}
    }

    fn vec_with_n_txs(n: usize) -> Vec<String> {
        let range = (1..n + 1).collect::<Vec<usize>>();
        range.iter().map(|i| format!("tx{}", i)).collect()
    }

    fn tree_with_n_leaves(n: usize) -> Merkle {
        Merkle::from_iter(vec_with_n_txs(n))
    }

    use test::Bencher;
    // Not precise benchmark, because we have to create new tree for every iteration
    // when we test insertion and deletion. But I hope this implementation reflects
    // amortized time.

    #[bench]
    fn construction_of_1K(bench: &mut Bencher) {
        bench_construction(1_000, bench)
    }
    #[bench]
    fn construction_of_2K(bench: &mut Bencher) {
        bench_construction(2_000, bench)
    }
    #[bench]
    fn construction_of_4K(bench: &mut Bencher) {
        bench_construction(4_000, bench)
    }
    #[bench]
    fn construction_of_8K(bench: &mut Bencher) {
        bench_construction(8_000, bench)
    }

    fn bench_construction(n: usize, bench: &mut Bencher) {
        let txs = vec_with_n_txs(n);
        bench.iter(|| Merkle::from_iter(txs.iter()))
    }

    #[bench]
    fn insert_to_250K(bench: &mut Bencher) {
        bench_insert(250_000, bench)
    }
    #[bench]
    fn insert_to_500K(bench: &mut Bencher) {
        bench_insert(500_000, bench)
    }
    #[bench]
    fn insert_to_1M(bench: &mut Bencher) {
        bench_insert(1_000_000, bench)
    }
    #[bench]
    fn insert_to_2M(bench: &mut Bencher) {
        bench_insert(2_000_000, bench)
    }

    fn bench_insert(n: usize, bench: &mut Bencher) {
        let mut tree = tree_with_n_leaves(n);
        let mut rng = rand::thread_rng();

        bench.iter(|| {
            tree.insert(&format!("tx{}", rng.gen_range(n + 2, n * 2)));
        })
    }

    #[bench]
    fn delete_from_250K(bench: &mut Bencher) {
        bench_delete(250_000, bench)
    }
    #[bench]
    fn delete_from_500K(bench: &mut Bencher) {
        bench_delete(500_000, bench)
    }
    #[bench]
    fn delete_from_1M(bench: &mut Bencher) {
        bench_delete(1_000_000, bench)
    }
    #[bench]
    fn delete_from_2M(bench: &mut Bencher) {
        bench_delete(2_000_000, bench)
    }

    fn bench_delete(n: usize, bench: &mut Bencher) {
        let mut tree = tree_with_n_leaves(n);
        let mut rng = rand::thread_rng();
        bench.iter(|| {
            tree.delete(&hash(&format!("tx{}", rng.gen_range(1, n + 1))));
        })
    }

    #[bench]
    fn proof_250K(bench: &mut Bencher) {
        bench_proof(250_000, bench)
    }
    #[bench]
    fn proof_500K(bench: &mut Bencher) {
        bench_proof(250_000, bench)
    }
    #[bench]
    fn proof_1M(bench: &mut Bencher) {
        bench_proof(250_000, bench)
    }
    #[bench]
    fn proof_2M(bench: &mut Bencher) {
        bench_proof(250_000, bench)
    }

    fn bench_proof(n: usize, bench: &mut Bencher) {
        let tree = tree_with_n_leaves(n);
        let mut rng = rand::thread_rng();
        bench.iter(|| {
            tree.path(&hash(&format!("tx{}", rng.gen_range(1, n + 1))));
        })
    }
}
