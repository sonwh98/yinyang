mod clojure;

//use log::{debug, error, info, trace, warn};

fn main() {
    env_logger::init();
    clojure::repl();

    // let rs = clojure::read_string("( \"[1]\" )");
    // println!("rs={:?}", rs);
}
