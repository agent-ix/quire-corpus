use crate::barrel::Store;

pub fn drive() {
    let handle = external_thing();
    handle.upsert();
}
