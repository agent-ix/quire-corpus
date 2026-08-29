struct Store;

impl Store {
    fn exported(&self, id: u32) -> u32 {
        id
    }

    fn crate_visible(&self) {}

    fn private_helper(&self) {}
}
