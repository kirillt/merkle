use hash::hash;

use std::iter::FromIterator;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Merkle {
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
        let data: HashMap<String,String> = leaves.into_iter().map(|tx| {
            let tx = tx.to_string();
            (hash(&tx), tx)
        }).collect();

        let tree = Vec::with_capacity(data.keys().len() * 2 - 1);
        Merkle { tree, data }
    }

}
