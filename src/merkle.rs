use hash::*;

use std::collections::HashMap;
use std::iter::FromIterator;

pub type Data = Vec<u8>;

#[derive(Debug, Default)]
pub struct Merkle {
    pub total: usize,
    pub leaves: usize,

    pub tree: Vec<Key>,
    pub data: HashMap<Key, Data>,

    index: HashMap<Key, usize>,
}

#[derive(Debug, Clone)]
pub enum PathNode {
    Left(Key),
    Right(Key),
}

pub struct DataBundle {
    pub path: Vec<PathNode>,
    pub data: Data,
}

pub fn transfer(source: &Merkle, target: &mut Merkle, i: usize) -> bool {
    source
        .query_bundle(i)
        .map(|chunk| target.insert_bundle(&chunk))
        .unwrap_or(false)
}

impl Merkle {
    pub fn new() -> Self {
        Merkle::from_iter(vec![])
    }

    pub fn reserve(root: &[u8], n: usize) -> Self {
        let mut merkle = Merkle::from_iter(vec![vec![]; n]);
        merkle.tree.insert(0, root.to_vec());
        merkle.index.insert(root.to_vec(), 0);
        merkle
    }

    pub fn root(&self) -> &[u8] {
        &self.tree[0]
    }

    pub fn push_str(&mut self, text: &str) -> bool {
        self.push(text.as_bytes())
    }

    pub fn push(&mut self, bytes: &[u8]) -> bool {
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
                    let p = parent_of(lim);

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

    pub fn insert_bundle(&mut self, bundle: &DataBundle) -> bool {
        self.insert(&bundle.data[..], &bundle.path[..])
    }

    pub fn insert(&mut self, bytes: &[u8], path: &[PathNode]) -> bool {
        let key = hash(bytes);

        if self.verify_path(&key, path) {
            let mut target = 0;

            let mut path = path.to_vec();
            path.reverse();

            for neighbour_node in path {
                let neighbour = neighbour_node.child_of(target);
                let neighbour_key = neighbour_node.unwrap();

                if self.is_null(neighbour) {
                    self.tree[neighbour] = neighbour_key;
                } else {
                    assert_eq!(self.tree[neighbour], neighbour_key);
                }

                target = neighbour_of(neighbour);
            }

            if self.is_null(target) {
                self.tree[target] = key.clone();
            } else {
                assert_eq!(self.tree[target], key);
            }
            self.index.insert(key.clone(), target);
            self.data.insert(key, bytes.to_vec());
            true
        } else {
            false
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
                    let p = parent_of(i);
                    let neighbour_key = self.neighbour(i).unwrap().to_vec();
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
                        let q = parent_of(n - 1);

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

    pub fn query_bundle(&self, i: usize) -> Option<DataBundle> {
        self.ith_leaf(i).map(|key| {
            let path = self.path(&key[..]).unwrap();
            let data = self.data[&key[..]].clone();
            DataBundle { path, data }
        })
    }

    pub fn ith_leaf(&self, i: usize) -> Option<Key> {
        if i < self.leaves {
            Some(self.tree[self.total - i - 1].clone())
        } else {
            None
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
                i = parent_of(i);
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
            i = parent_of(i);
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

    fn is_null(&self, i: usize) -> bool {
        self.tree[i].is_empty()
    }
}

impl FromIterator<Data> for Merkle {
    fn from_iter<I: IntoIterator<Item = Data>>(leaves: I) -> Self {
        fn update_parent(tree: &mut Vec<Option<Key>>, i: usize, child: &[u8]) -> () {
            tree[parent_of(i)] = tree[parent_of(i)]
                .take()
                .map(|parent| hash_two(&parent, child))
                .or_else(|| Some(child.to_vec()));
        }

        let data: Vec<(Key, Data)> = leaves
            .into_iter()
            .map(|leaf| (hash(&leaf[..]), leaf))
            .collect();

        let keys: Vec<&Key> = data.iter().map(|(key, _)| key).collect();

        let leaves = keys.len();
        let total = if leaves > 0 { leaves * 2 - 1 } else { 0 };

        let mut tree: Vec<Option<Key>> = vec![None; total];
        let mut index = HashMap::new();
        let mut i = total;

        for leaf in keys {
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

        let data: HashMap<Key, Data> = data.iter().cloned().collect();

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
    fn child_of(&self, parent: usize) -> usize {
        match self {
            PathNode::Right(_) => parent * 2 + 1,
            PathNode::Left(_) => (parent + 1) * 2,
        }
    }

    fn unwrap(&self) -> Key {
        match self {
            PathNode::Right(key) => key.clone(),
            PathNode::Left(key) => key.clone(),
        }
    }
}

fn neighbour_of(i: usize) -> usize {
    if i % 2 == 0 {
        i - 1
    } else {
        i + 1
    }
}

fn parent_of(i: usize) -> usize {
    (i - 1) / 2
}
