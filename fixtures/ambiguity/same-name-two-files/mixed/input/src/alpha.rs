pub struct Handler;

impl Handler {
    pub fn handle(&self) {}
}

pub fn alpha_driver() {
    let h = Handler;
    h.handle();
}
