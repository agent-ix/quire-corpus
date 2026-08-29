pub struct Store;

impl Store {
    pub fn compute(&self, id: u64) -> u32 {
        id as u32
    }
}
