#[derive(Debug)]
#[derive(Clone)]
pub enum Tree<T> {
    Nil,

    Leaf(T),

    Join {
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
        value: T
    }
}

impl<T> Tree<T> {
    fn leaf(value: T) -> Tree<T> {
        Tree::Leaf(value)
    }

    fn join(left: T, right: T) -> Tree<Option<T>> {
        Tree::union(Tree::leaf(Some(left)),
                    Tree::leaf(Some(right)))
    }

    fn union(left: Tree<Option<T>>, right: Tree<Option<T>>) -> Tree<Option<T>> {
        Tree::Join {
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
            .map(|v| Tree::leaf(Some(v)))
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
}