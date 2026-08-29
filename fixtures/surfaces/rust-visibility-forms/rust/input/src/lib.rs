pub trait Persist {
    fn trait_item(&self);
}

pub struct Store;

impl Persist for Store {
    fn trait_impl_item(&self) {}
}

pub mod inner {
    pub(super) fn super_visible() {}
    pub(crate) fn crate_visible() {}
    pub fn fully_public() {}
    fn module_private() {}
}
