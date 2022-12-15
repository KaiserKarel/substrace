macro_rules! docs {
    ($($lint_name: expr,)*) => {
        pub fn explain(lint: &str) {
            println!("{}", match lint {
                $(
                    $lint_name => include_lint!(concat!("docs/", concat!($lint_name, ".txt"))),
                )*
                _ => "unknown linttt",
            })
        }
    }
}

docs! {}
