use crate::leaf::Leaf;

pub struct Mid;

impl Mid {
    pub fn step(&self) -> u32 {
        let leaf = Leaf;
        leaf.compute()
    }
}
