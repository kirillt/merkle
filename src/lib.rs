#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![feature(iterator_flatten)]
#![feature(test)]

extern crate crypto;
extern crate test;

mod hash;
pub mod merkle;

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate rand;

    use hash::{hash, hash_str};
    use merkle::{transfer, Merkle};
    use std::iter::FromIterator;
    use std::str::from_utf8;

    use tests::rand::Rng;

    #[test]
    fn create() {
        check_tree(&tree_with_n_leaves(7));
    }

    #[test]
    fn push_generic() {
        let mut merkle = tree_with_n_leaves(7);
        merkle.push_str("tx1");
        merkle.push_str("tx8");
        merkle.push_str("tx9");
        merkle.push_str("tx10");
        merkle.push_str("tx11");
        check_tree(&merkle);
    }

    #[test]
    fn retrieve_by_index() {
        for n in 1..100 {
            let merkle = tree_with_n_leaves(n);
            for i in 0..n {
                let key = merkle.ith_leaf(i).unwrap();
                let chunk = merkle.data.get(&key).unwrap();
                assert_eq!(from_utf8(chunk), Ok(format!("tx{}", i + 1).as_str()));
            }
        }
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
        merkle.delete(&hash_str("tx11")); //non-existent
        merkle.delete(&hash_str("tx3"));
        merkle.delete(&hash_str("tx7"));
        merkle.delete(&hash_str("tx4"));
        merkle.delete(&hash_str("tx1"));
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
                let key = hash_str(&tx);
                match rng.gen_range(0, 2) {
                    0 => {
                        merkle.push_str(&tx);
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

    #[test]
    fn hash_of_null_is_null() {
        let null: &[u8] = &[];
        assert_eq!(null, hash(null).as_slice());
    }

    #[test]
    fn torrent_scenario() {
        let file = "some file, which we download over something like bittorent".as_bytes();
        let chunks: Vec<&[u8]> = file.chunks(4).collect();

        let seed: Merkle = Merkle::from_iter(chunks.iter().map(|chunk| chunk.to_vec()));

        //split data among some peers
        let mut peer_A: Merkle = Merkle::reserve(seed.root(), seed.leaves);
        let mut peer_B: Merkle = Merkle::reserve(seed.root(), seed.leaves);

        for i in 0..seed.leaves {
            let mut target = if i % 2 == 0 { &mut peer_A } else { &mut peer_B };

            let success = transfer(&seed, &mut target, i);
            assert!(success);
        }

        //join data from partial peers into some other peer
        let mut peer_C: Merkle = Merkle::reserve(seed.root(), peer_A.leaves);

        for i in 0..seed.leaves {
            let source = if i % 2 == 0 { &peer_A } else { &peer_B };

            let success = transfer(source, &mut peer_C, i);
            assert!(success);
        }

        //this peer contains full data
        assert!(peer_C.verify_tree());

        let downloaded: Vec<u8> = (0..peer_C.leaves)
            .flat_map(|i| peer_C.ith_leaf(i))
            .flat_map(|key| peer_C.data.get(&key))
            .cloned()
            .flatten()
            .collect();

        assert_eq!(&downloaded[..], file);
    }

    fn check_tree(merkle: &Merkle) {
        assert!(merkle.total == 0 || merkle.total == merkle.leaves * 2 - 1);
        assert!(merkle.verify_tree());

        assert!(merkle.path_str("absent_key").is_none());
        for key in merkle.data.keys() {
            let path = merkle.path(key).unwrap();
            assert!(merkle.verify_path(key, &path));
        }

        //for (key, i) in merkle.index.iter() {
        //    assert_eq!(merkle.tree.get(*i), Some(key));
        //}
    }

    fn vec_with_n_txs(n: usize) -> Vec<Vec<u8>> {
        let range: Vec<usize> = (1..n + 1).collect();
        range
            .iter()
            .map(|i| format!("tx{}", i).as_bytes().to_vec())
            .collect()
    }

    fn tree_with_n_leaves(n: usize) -> Merkle {
        Merkle::from_iter(vec_with_n_txs(n))
    }

    use test::Bencher;
    // Not precise benchmark, because we have to create new tree for every iteration
    // when we test push and deletion. But I hope this implementation reflects
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
        let txs: Vec<Vec<u8>> = vec_with_n_txs(n);
        bench.iter(|| Merkle::from_iter(txs.iter().cloned()))
        //todo: remove .cloned()
    }

    #[bench]
    fn push_to_250K(bench: &mut Bencher) {
        bench_insert(250_000, bench)
    }
    #[bench]
    fn push_to_500K(bench: &mut Bencher) {
        bench_insert(500_000, bench)
    }
    #[bench]
    fn push_to_1M(bench: &mut Bencher) {
        bench_insert(1_000_000, bench)
    }
    #[bench]
    fn push_to_2M(bench: &mut Bencher) {
        bench_insert(2_000_000, bench)
    }

    fn bench_insert(n: usize, bench: &mut Bencher) {
        let mut tree = tree_with_n_leaves(n);
        let mut rng = rand::thread_rng();

        bench.iter(|| {
            tree.push_str(&format!("tx{}", rng.gen_range(n + 2, n * 2)));
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
            tree.delete(&hash_str(&format!("tx{}", rng.gen_range(1, n + 1))));
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
            tree.path(&hash_str(&format!("tx{}", rng.gen_range(1, n + 1))));
        })
    }
}
