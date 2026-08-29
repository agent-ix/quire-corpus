use crate::store::Store;

pub fn drive() {
    let store = Store;
    store.upsert();
}
