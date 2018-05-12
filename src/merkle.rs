use hash::*;

use std::collections::HashMap;
use std::iter::FromIterator;

pub type Data = Vec<u8>;

#[derive(Debug)]
pub struct Merkle {
    pub total: usize,
    pub leaves: usize,

    pub tree: Vec<Key>,

    pub data: HashMap<Key, Data>,

    index: HashMap<Key, usize>,
}

#[derive(Debug)]
pub enum PathNode {
    Left(Key),
    Right(Key),
}

impl Merkle {
    pub fn root(&self) -> &[u8] {
        &self.tree[0]
    }

    pub fn insert_str(&mut self, text: &str) -> bool {
        self.insert(text.as_bytes())
    }

    pub fn insert(&mut self, bytes: &[u8]) -> bool {
        let key = hash(bytes);
        let lim = self.total;

        match self.index.get(&key) {
            Some(_) => false,
            None => {
                self.data.insert(key.clone(), bytes.to_vec());

                if self.total == 0 {
                    self.tree.push(key.clone());
                    self.index.insert(key, 0);
                    self.total = 1;
                } else {
                    // there is no unpaired leaves in the tree
                    // when we insert new leaf -- it makes new pair with leaf from above
                    let p = parent(lim);

                    let old = self.tree[p].clone();
                    self.tree.push(old.clone());
                    self.tree.push(key.clone());

                    self.index.insert(old.clone(), lim);
                    self.index.insert(key.clone(), lim + 1);

                    self.update_parents(p, hash_two(&key, &old));

                    self.total += 2;
                }

                self.leaves += 1;
                true
            }
        }
    }

    pub fn delete(&mut self, key: &[u8]) -> bool {
        let n = self.total;
        let k = self.leaves;

        match self.index.get(key) {
            None => false,
            Some(&i) => {
                if i == 0 {
                    self.total = 0;
                    self.leaves = 0;
                    self.tree.pop();
                } else {
                    let p = parent(i);
                    let neighbour_key = self.neighbour(i).unwrap();
                    let farthest_left_leaf = self.tree.pop().unwrap();
                    let farthest_right_leaf = self.tree.pop().unwrap();

                    //easy cases are deletion of nearest leaf in odd trees
                    //and deletion of leaf from farthest pair
                    if k % 2 == 1 && i == n - k {
                        self.tree[i] = farthest_left_leaf.clone();
                        self.tree[i - 1] = farthest_right_leaf.clone();

                        self.index.insert(farthest_left_leaf, i);
                        self.index.insert(farthest_right_leaf, i - 1);

                        self.update_parents(p, neighbour_key);
                    } else if i == n - 1 || i == n - 2 {
                        assert!(
                            key == farthest_left_leaf.as_slice()
                                && neighbour_key == farthest_right_leaf.as_slice()
                                || key == farthest_right_leaf.as_slice()
                                    && neighbour_key == farthest_left_leaf.as_slice()
                        );

                        self.index.insert(neighbour_key.to_vec(), p);

                        self.update_parents(p, neighbour_key);
                    } else {
                        let q = parent(n - 1);

                        self.tree[p * 2 + 2] = farthest_left_leaf.clone();
                        self.tree[p * 2 + 1] = farthest_right_leaf.clone();

                        self.index.insert(farthest_left_leaf, p * 2 + 2);
                        self.index.insert(farthest_right_leaf, p * 2 + 1);

                        let farthest_parent_key = self.tree[q].clone();
                        self.tree[p] = farthest_parent_key.clone();
                        self.index.insert(neighbour_key.to_vec(), q);

                        self.update_parents(q, neighbour_key);
                        self.update_parents(p, farthest_parent_key);
                    }

                    self.leaves -= 1;
                    self.total -= 2;
                }

                self.index.remove(key);
                self.data.remove(key);
                true
            }
        }
    }

    pub fn verify_tree(&self) -> bool {
        if self.total < 1 {
            true
        } else {
            let last = self.total - 1;

            let hash_inv = |l: usize, r: usize| hash_two(&self.tree[r], &self.tree[l]);

            self.tree
                .iter()
                .enumerate()
                .map(|(i, key)| {
                    let (l, r) = (i * 2 + 1, (i + 1) * 2);
                    l > last || r > last || key == &hash_inv(l, r)
                })
                .all(|result| result)
        }
    }

    pub fn path_str(&self, key: &str) -> Option<Vec<PathNode>> {
        self.path(key.as_bytes())
    }

    pub fn path(&self, key: &[u8]) -> Option<Vec<PathNode>> {
        self.index.get(key).map(|target| {
            let mut i = *target;
            let mut result: Vec<PathNode> = vec![];
            while i > 0 {
                result.push(self.neighbour(i));
                i = parent(i);
            }
            result
        })
    }

    pub fn verify_path(&self, target: &Key, path: &[PathNode]) -> bool {
        let result = path.iter().fold(target.clone(), |acc, node| match *node {
            PathNode::Left(ref key) => hash_two(key, &acc[..]),
            PathNode::Right(ref key) => hash_two(&acc[..], key),
        });
        result == self.root()
    }

    fn update_parents(&mut self, from: usize, initial_key: Key) {
        let mut i = from;
        let mut updated_key = initial_key;
        while i > 0 {
            self.tree[i] = updated_key.clone();
            updated_key = match self.neighbour(i) {
                PathNode::Left(neighbour_key) => hash_two(&neighbour_key, &updated_key),
                PathNode::Right(neighbour_key) => hash_two(&updated_key, &neighbour_key),
            };
            i = parent(i);
        }
        self.tree[i] = updated_key.clone();
    }

    fn neighbour(&self, i: usize) -> PathNode {
        if i % 2 == 0 {
            PathNode::Right(self.tree[i - 1].clone())
        } else {
            PathNode::Left(self.tree[i + 1].clone())
        }
    }
}

impl FromIterator<Data> for Merkle {
    fn from_iter<I: IntoIterator<Item = Data>>(leaves: I) -> Self {
        fn update_parent(tree: &mut Vec<Option<Key>>, i: usize, child: &[u8]) -> () {
            tree[parent(i)] = tree[parent(i)]
                .take()
                .map(|parent| hash_two(&parent, child))
                .or_else(|| Some(child.to_vec()));
        }

        let data: HashMap<Key, Data> = leaves
            .into_iter()
            .map(|leaf| (hash(&leaf[..]), leaf))
            .collect();

        let leaves = data.keys().len();
        let total = if leaves > 0 { leaves * 2 - 1 } else { 0 };

        let mut tree: Vec<Option<Key>> = vec![None; total];
        let mut index = HashMap::new();
        let mut i = total;

        for leaf in data.keys() {
            i -= 1;

            tree[i] = Some(leaf.clone());
            index.insert(leaf.clone(), i);
            if i > 0 {
                update_parent(&mut tree, i, &leaf);
            }
        }

        while i > 1 {
            i -= 1;

            let key = tree[i].clone().unwrap();
            update_parent(&mut tree, i, &key);
        }

        let tree: Vec<Key> = tree.into_iter().map(|x| x.unwrap()).collect();

        Merkle {
            tree,
            data,
            total,
            leaves,
            index,
        }
    }
}

impl PathNode {
    fn unwrap(self) -> Key {
        match self {
            PathNode::Right(key) => key,
            PathNode::Left(key) => key,
        }
    }
}

fn parent(i: usize) -> usize {
    (i - 1) / 2
}
