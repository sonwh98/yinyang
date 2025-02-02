mod clojure;

use log::{debug, error, info, trace, warn};

use yinyang::clojure::repl;

fn main() {
    env_logger::init();
    repl();

    // let rs = clojure::read_string("( \"[1]\" )");
    // println!("rs={:?}", rs);
}
