pub union Bits {
    int: u32,
    float: f32,
}

pub async fn fetch() {}

pub mod outer {
    pub mod inner {
        pub fn deep() {}
    }
}

pub fn host() {
    fn nested() {}
}
