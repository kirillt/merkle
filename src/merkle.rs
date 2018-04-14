use hash::hash;

use std::iter::FromIterator;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Merkle {
    pub total: usize,
    pub leaves: usize,

    pub tree: Vec<String>,

    pub data: HashMap<String,String>
}

#[derive(Debug)]
pub enum PathNode {
    Left(String),
    Right(String)
}

impl Merkle {

    pub fn root(&self) -> &String {
        &self.tree[0]
    }

    pub fn verify_tree(&self) -> bool {
        let last = self.tree.len() - 1;

        self.tree.iter().enumerate()
            .map(|(i, key)| {
                let (l, r) = (i * 2 + 1, (i + 1) * 2);
                l > last || r > last ||
                    key == &hash(&(self.tree[r].clone() + &self.tree[l]))
            })
            .all(|result| result == true)
    }

    pub fn path(&self, key: &String) -> Option<Vec<PathNode>> {
        self.tree.iter().enumerate()
            .find(|&(_, elem)| elem == key)
            .map(|(mut i, _)| {
                let mut result: Vec<PathNode> = vec![];
                while i > 0 {
                    result.push(self.neighbour(i));
                    i = parent(i);
                }
                result
            })
    }

    pub fn verify_path(&self, target: &String, path: &Vec<PathNode>) -> bool {
        let result = path.iter().fold(target.clone(), |acc, node| {
            let mut buffer = String::new();
            match node {
                &PathNode::Left(ref key) => {
                    buffer.push_str(key);
                    buffer.push_str(&acc);
                },
                &PathNode::Right(ref key) => {
                    buffer.push_str(&acc);
                    buffer.push_str(key);
                }
            }
            hash(&buffer)
        });
        &result == self.root()
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

        fn update_parent(tree: &mut Vec<Option<String>>, i: usize, child: &String) -> () {
            tree[parent(i)] = tree[parent(i)].take()
                .map(|parent| hash(&(parent + child.as_str())))
                .or(Some(child.clone()));
        };

        let data: HashMap<String,String> = leaves.into_iter().map(|key| {
            let key = key.to_string();
            (hash(&key), key)
        }).collect();

        let leaves = data.keys().len();
        let total = leaves * 2 - 1;

        let mut tree: Vec<Option<String>> = vec![None; total];
        let mut i = total;

        for leaf in data.keys() {
            i -= 1;

            tree[i] = Some(leaf.clone());
            update_parent(&mut tree, i, &leaf);
        }

        while i > 1 {
            i -= 1;

            let key = tree[i].clone().unwrap();
            update_parent(&mut tree, i, &key);
        }

        let tree: Vec<String> = tree.into_iter()
            .map(|x| x.unwrap())
            .collect();

        Merkle { leaves, total, tree, data }
    }

}

fn parent(i: usize) -> usize {
    (i - 1) / 2
}