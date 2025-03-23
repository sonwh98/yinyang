use yinyang::repl::{create_env, start_repl};

fn main() {
    let mut env = create_env();
    start_repl(&mut env);
}
