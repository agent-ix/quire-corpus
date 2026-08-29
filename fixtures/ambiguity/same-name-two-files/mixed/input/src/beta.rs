pub struct Handler;

impl Handler {
    pub fn handle(&self) {}
}

pub fn beta_driver() {
    let h = Handler;
    h.handle();
}
