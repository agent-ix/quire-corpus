pub trait Persist {
    fn flush(&self);
}

pub struct Disk;
impl Persist for Disk {
    fn flush(&self) {}
}

pub struct Memory;
impl Persist for Memory {
    fn flush(&self) {}
}

pub fn run(sink: &dyn Persist) {
    sink.flush();
}
