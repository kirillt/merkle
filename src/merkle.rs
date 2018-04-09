use tree::Tree;
use hash::hash;

use std::iter::FromIterator;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Merkle {
    pub tree: Tree<String>,
    pub data: HashMap<String,String>
}

impl Merkle {

    pub fn path(&self, key: &String) -> Option<Vec<String>> {
        fn append(x: &String, mut p: Vec<String>) -> Vec<String> {
            p.push(x.clone());
            p
        };

        fn find_in(tree: &Tree<String>, key: &String) -> Option<Vec<String>> {
            match tree {
                &Tree::Nil => None,
                &Tree::Leaf(ref k) => if k == key { Some(vec![]) } else { None },
                &Tree::Branch { ref left, ref right, .. } => {
                    let append_left = |p| append(left.value(), p);
                    let append_right = |p| append(right.value(), p);

                    find_in(left, key).map(append_right).or_else(||
                        find_in(right, key).map(append_left))
                }
            }
        }

        find_in(&self.tree, key)
    }

    #[allow(dead_code)]
    pub fn diff(&self, _other: &Tree<String>) -> Vec<String> {
        unimplemented!()
    }


    fn propagate(tree: Tree<Option<String>>) -> Tree<String> {
        tree.into_map_separately(&|x| x.unwrap(), &|l,r,_| hash(&format!("{}{}",l,r)))
    }

}

impl<T: ToString> FromIterator<T> for Merkle {

    fn from_iter<I: IntoIterator<Item = T>>(leaves: I) -> Self {
        let data: HashMap<String,String> = leaves.into_iter().map(|tx| {
            let tx = tx.to_string();
            (hash(&tx), tx)
        }).collect();

        let base: Tree<Option<String>> = Tree::from_iter(data.keys().map(|k| k.clone()));
        Merkle {
            tree: Merkle::propagate(base),
            data
        }
    }

}
