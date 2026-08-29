pub trait Persist {
    fn save(&self) -> u32;
}

pub struct Store;

impl Store {
    pub fn exported(&self, id: u32) -> u32 {
        id
    }

    pub(crate) fn crate_visible(&self) {}

    fn private_helper(&self) {}
}

impl Persist for Store {
    fn save(&self) -> u32 {
        0
    }
}
