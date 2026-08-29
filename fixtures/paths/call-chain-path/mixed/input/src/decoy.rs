pub struct Decoy;

impl Decoy {
    pub fn compute(&self) -> u32 {
        2
    }
}

pub fn decoy_entry() -> u32 {
    let decoy = Decoy;
    decoy.compute()
}
