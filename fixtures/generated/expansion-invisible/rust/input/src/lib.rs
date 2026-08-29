macro_rules! declare_helper {
    ($name:ident) => {
        pub fn $name() {}
    };
}

declare_helper!(generated);

pub fn written_literally() {}
