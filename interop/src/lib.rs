#[cfg(test)]
mod test {
    use std::fs::{read_dir, File};
    use std::io::prelude::*;
    use za_compiler::tester;
    use za_parser::parse;

    #[test]
    fn circomlib_parse() {
        let paths = read_dir("./circuits/circomlib/circuits").unwrap();
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
        match tester::run_embeeded_tests(
            "./circuits/circomlib",
            "all_tests.circom",
            false,
            false,
            false,
            "",
        ) {
            Ok(Some((_, err))) => {
                println!("{:?}", err);
                assert!(false);
            }
            Err(err) => {
                println!("{:?}", err);
                assert!(false);
            }
            _ => {}
        }
    }
}
