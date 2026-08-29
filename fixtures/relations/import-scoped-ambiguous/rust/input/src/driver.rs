use crate::store::Store;
use crate::cache::Cache;

pub fn drive() {
    let handle = external_thing();
    handle.upsert();
}
