extern crate lalrpop;
extern crate num_bigint;
extern crate num_traits;
extern crate rustc_hex;

fn main() {
    lalrpop::process_root().unwrap();
}
