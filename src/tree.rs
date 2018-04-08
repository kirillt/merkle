use std::iter::FromIterator;
use std::cmp;

#[derive(Clone)]
#[derive(Debug)]
pub enum Tree<T> {
    Nil,

    Leaf(T),

    Branch {
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
        value: T
    }
}

impl<T> Tree<T> {

    fn join(left: T, right: T) -> Tree<Option<T>> {
        Tree::union(Tree::Leaf(Some(left)),
                    Tree::Leaf(Some(right)))
    }

    fn union(left: Tree<Option<T>>, right: Tree<Option<T>>) -> Tree<Option<T>> {
        Tree::Branch {
            left: Box::new(left),
            right: Box::new(right),
            value: None
        }
    }

    fn from_forest(forest: Vec<Tree<Option<T>>>) -> Tree<Option<T>> {
        if forest.is_empty() {
            return Tree::Nil;
        }

        let mut result = Tree::grow(forest);
        while result.len() > 1 {
            result = Tree::grow(result);
        }
        result.swap_remove(0)
    }

    fn from_leaves(values: Vec<T>) -> Tree<Option<T>> {
        Tree::from_forest(values.into_iter()
            .map(|v| Tree::Leaf(Some(v)))
            .collect())
    }

    fn grow(forest: Vec<Tree<Option<T>>>) -> Vec<Tree<Option<T>>> {
        let mut result = Vec::new();
        let mut left = None;
        for tree in forest.into_iter() {
            if left.is_none() {
                left = Some(tree);
            } else {
                result.push(Tree::union(left.unwrap(), tree));
                left = None;
            }
        }
        if left.is_some() {
            result.push(left.unwrap());
        }
        result
    }

    fn fold<R>(self, on_nil: &Fn() -> R, on_leaf: &Fn(T) -> R,
                     on_branch: &Fn(R,R,T) -> R) -> R {
        match self {
            Tree::Branch { left, right, value } => on_branch(
                left.fold(on_nil, on_leaf, on_branch),
                right.fold(on_nil, on_leaf, on_branch),
                value),
            Tree::Leaf(v) => on_leaf(v),
            Tree::Nil => on_nil()
        }
    }

    fn map<R>(self, f: &Fn(T) -> R) -> Tree<R> {
        self.fold(&|| Tree::Nil, &|x| Tree::Leaf(f(x)),
                  &|l,r,v| Tree::Branch {
                      left: Box::new(l), right: Box::new(r),
                      value: f(v)
                  })
    }

}

impl<T> FromIterator<T> for Tree<Option<T>> {

    fn from_iter<I: IntoIterator<Item = T>>(leaves: I) -> Self {
        Tree::from_leaves(leaves.into_iter().collect())
    }

}

pub fn test() {
    println!("1 x 2\n\t{:?}\n",
             Tree::join(1,2));

    let mut leaves: Vec<usize> = Vec::new();
    for i in 1..10 {
        leaves.push(i);
        println!("{:?}\n\t{:?}\n", leaves,
                 Tree::from_leaves(leaves.clone()));
    }

    let nine_leaves: Tree<Option<usize>> = Tree::from_iter(1..10);
    let sum = nine_leaves.clone().fold(&|| 0, &|x| x.unwrap(), &|l,r, _| l + r);
    println!("sum must be 45: {}", sum);
    let height = nine_leaves.clone().fold(&|| 0, &|_| 0, &|l,r,_| 1 + cmp::max(l,r));
    println!("height must be 4: {}", height);
    println!();

    let stringified: Tree<String> = nine_leaves.map(&|opt| match opt {
        Some(v) => format!("{}", v),
        None => "?".to_string()
    });
    println!("stringified:\n{:?}", stringified);
}