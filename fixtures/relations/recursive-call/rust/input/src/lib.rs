pub struct Store;

impl Store {
    pub fn recurse(&self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            self.recurse(n - 1)
        }
    }
}

pub fn plain_recursion(n: u32) -> u32 {
    if n == 0 {
        0
    } else {
        plain_recursion(n - 1)
    }
}
