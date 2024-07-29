use std::{cell::RefCell, rc::Weak};

#[derive(Debug)]
pub struct TreeBranch<C> {
    pub node: C,
    /// `None` designates a root node
    pub parent: Option<Weak<TreeBranch<C>>>,
    pub children: RefCell<Vec<Weak<TreeBranch<C>>>>,
}
