use hash::hash;

use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Debug)]
pub struct Merkle {
    pub total: usize,
    pub leaves: usize,

    pub tree: Vec<String>,

    pub data: HashMap<String, String>,

    index: HashMap<String, usize>,
}

#[derive(Debug)]
pub enum PathNode {
    Left(String),
    Right(String),
}

impl Merkle {
    pub fn root(&self) -> &String {
        &self.tree[0]
    }

    pub fn insert(&mut self, value: &str) -> bool {
        let key = hash(value);
        let lim = self.total;

        match self.index.get(&key) {
            Some(_) => false,
            None => {
                self.data.insert(key.clone(), value.to_string());

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

                    self.update_parents(p, hash(&(key + &old)));

                    self.total += 2;
                }

                self.leaves += 1;
                true
            }
        }
    }

    pub fn delete(&mut self, key: &str) -> bool {
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
                            key == farthest_left_leaf && neighbour_key == farthest_right_leaf
                                || key == farthest_right_leaf
                                    && neighbour_key == farthest_left_leaf
                        );

                        self.index.insert(neighbour_key.clone(), p);

                        self.update_parents(p, neighbour_key);
                    } else {
                        let q = parent(n - 1);

                        self.tree[p * 2 + 2] = farthest_left_leaf.clone();
                        self.tree[p * 2 + 1] = farthest_right_leaf.clone();

                        self.index.insert(farthest_left_leaf, p * 2 + 2);
                        self.index.insert(farthest_right_leaf, p * 2 + 1);

                        let farthest_parent_key = self.tree[q].clone();
                        self.tree[p] = farthest_parent_key.clone();
                        self.index.insert(neighbour_key.clone(), q);

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

            let hash_inv = |l: usize, r: usize| hash(&(self.tree[r].clone() + &self.tree[l]));

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

    pub fn path(&self, key: &str) -> Option<Vec<PathNode>> {
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

    pub fn verify_path(&self, target: &str, path: &[PathNode]) -> bool {
        let result = path.iter().fold(target.to_string(), |acc, node| {
            let mut buffer = String::new();
            match *node {
                PathNode::Left(ref key) => {
                    buffer.push_str(key);
                    buffer.push_str(&acc);
                }
                PathNode::Right(ref key) => {
                    buffer.push_str(&acc);
                    buffer.push_str(key);
                }
            }
            hash(&buffer)
        });
        &result == self.root()
    }

    fn update_parents(&mut self, from: usize, initial_key: String) {
        let mut i = from;
        let mut updated_key = initial_key;
        while i > 0 {
            self.tree[i] = updated_key.clone();
            updated_key = match self.neighbour(i) {
                PathNode::Left(neighbour_key) => hash(&(neighbour_key + &updated_key)),
                PathNode::Right(neighbour_key) => hash(&(updated_key + &neighbour_key)),
            };
            i = parent(i);
        }
        self.tree[i] = updated_key;
    }

    fn neighbour(&self, i: usize) -> PathNode {
        if i % 2 == 0 {
            PathNode::Right(self.tree[i - 1].clone())
        } else {
            PathNode::Left(self.tree[i + 1].clone())
        }
    }
}

impl<T: ToString> FromIterator<T> for Merkle {
    fn from_iter<I: IntoIterator<Item = T>>(leaves: I) -> Self {
        fn update_parent(tree: &mut Vec<Option<String>>, i: usize, child: &str) -> () {
            tree[parent(i)] = tree[parent(i)]
                .take()
                .map(|parent| hash(&(parent + child)))
                .or_else(|| Some(child.to_string()));
        }

        let data: HashMap<String, String> = leaves
            .into_iter()
            .map(|key| {
                let key = key.to_string();
                (hash(&key), key)
            })
            .collect();

        let leaves = data.keys().len();
        let total = if leaves > 0 { leaves * 2 - 1 } else { 0 };

        let mut tree: Vec<Option<String>> = vec![None; total];
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

        let tree: Vec<String> = tree.into_iter().map(|x| x.unwrap()).collect();

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
    fn unwrap(self) -> String {
        match self {
            PathNode::Right(key) => key,
            PathNode::Left(key) => key,
        }
    }
}

fn parent(i: usize) -> usize {
    (i - 1) / 2
}
