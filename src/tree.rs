use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Debug)]
pub struct TreeBranch<C> {
    pub node: C,
    /// `None` designates a root node
    pub parent: Option<Weak<TreeBranch<C>>>,
    pub children: RefCell<Vec<Rc<TreeBranch<C>>>>,
}
