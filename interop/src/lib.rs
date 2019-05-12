#[cfg(test)]
mod test {
    use std::fs::{read_dir, File};
    use std::io::prelude::*;
    use circom2_parser::parse;
    use circom2_compiler::tester;

    #[test]
    fn circomlib_parse() {
        let paths = read_dir("./circomlib").unwrap();
        for path in paths {
            let path = path.unwrap().path();
            if path.is_file() {
                println!("+++ parsing testing {} +++", path.display());
                let mut file = File::open(path).expect("Unable to open the file");
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Unable to read the file");
                if let Err(err) = parse(&contents) {
                    panic!("{:?}", err);
                }
            }
        }
    }

    #[test]
    fn circomlib_tests() {

        if let Err((eval,err)) = tester::run_embeeded_test("./circomlib","all_tests.circom") {
            println!("{:?}",err);
            assert!(false);
        }
    }

}
