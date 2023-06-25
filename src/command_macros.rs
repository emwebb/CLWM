#[macro_use]
pub mod command_macros {
    #[macro_export]
    macro_rules! arg_input {
        ($op:expr, $query:expr) => {{
            let mut arg = String::new();
            if $op.is_some() {
                arg = $op.clone().unwrap().to_string();
            } else {
                println!($query);
                std::io::stdin()
                    .read_line(&mut arg)
                    .expect("failed to readline");
                if let Some('\n') = arg.chars().next_back() {
                    arg.pop();
                }
                if let Some('\r') = arg.chars().next_back() {
                    arg.pop();
                }
            }
            arg
        }};
    }
}
