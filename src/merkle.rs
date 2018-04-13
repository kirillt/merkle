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

impl Merkle {

    pub fn path(&self, key: &String) -> Option<Vec<String>> {
        unimplemented!();
    }

    #[allow(dead_code)]
    pub fn diff(&self, _other: &Vec<String>) -> Vec<String> {
        unimplemented!()
    }

}

impl<T: ToString> FromIterator<T> for Merkle {

    fn from_iter<I: IntoIterator<Item = T>>(leaves: I) -> Self {

        fn update_parent(tree: &mut Vec<Option<String>>, i: usize, child: &String) -> () {
            let i = (i - 1) / 2;

            tree[i] = tree[i].take()
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
