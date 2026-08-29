pub trait Persist {
    fn save(&self);
}

pub struct Disk;
impl Persist for Disk {
    fn save(&self) {}
}

pub struct Memory;
impl Persist for Memory {
    fn save(&self) {}
}

pub fn run<T: Persist>(sink: &T) {
    sink.save();
}
