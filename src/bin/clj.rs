use yinyang::repl::{create_env, repl};

fn main() {
    let mut env = create_env();
    repl(&mut env);
}
