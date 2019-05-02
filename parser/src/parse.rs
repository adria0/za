use super::ast;
use super::ast::Meta;
use super::lang;
use super::error::*;

fn preprocess(expr: &str) -> Result<String> {
    let mut pp = String::new();
    let mut state = 0;
    let mut loc = 0;
    let mut block_comment_start = 0;

    let mut it = expr.chars();
    while let Some(c0) = it.next() {
        loc += 1;
        match (state, c0) {
            (0, '/') => { loc += 1; match it.next() {
                Some('/') => {
                    state = 1;
                    pp.push(' '); pp.push(' ');
                }
                Some('*') => {
                    block_comment_start = loc;
                    state = 2;
                    pp.push(' '); pp.push(' ');
                }
                Some(c1) => {
                    pp.push(c0); pp.push(c1);
                }
                None => {
                    pp.push(c0);
                    break;
                }
            }},
            (0, _) => pp.push(c0),
            (1, '\n') => {
                pp.push(c0);
                state = 0;
            }
            (2, '*') => { loc += 1; match it.next() {
                Some('/') => {
                    pp.push(' '); pp.push(' ');
                    state = 0;
                }
                Some(_) => { 
                    pp.push(' '); pp.push(' ');
                }
                None =>
                    return Err(
                        Error::ParseError("unterminated /* */".to_string(),
                        Meta::new(block_comment_start,block_comment_start,None))
                    ),
            }},
            _ => { 
                pp.push(' ');
            }
        }
    }
    Ok(pp)
}

/// parse circom lang
pub fn parse(expr: &str) -> Result<Vec<ast::BodyElementP>> {
    use lalrpop_util::ParseError::*;
    lang::BodyParser::new()
        .parse(&preprocess(expr)?)
        .map_err(|err| match err { 
            InvalidToken{location}
                => Error::ParseError(format!("{:?}", err), Meta::new(location,location,None)),
            UnrecognizedToken{token:Some((left,_,right)),..}
                => Error::ParseError(format!("{:?}", err), Meta::new(left,right,None)),
            ExtraToken{token:(left,_,right)}
                => Error::ParseError(format!("{:?}", err), Meta::new(left,right,None)),
            _ 
                => Error::ParseError(format!("{:?}", err), Meta::new(0,0,None))
         })   
}

#[cfg(test)]
mod test {
    use std::fs::{read_dir, File};
    use std::io::prelude::*;

    fn test_preprocess(expr: &str, expected: &str) {
        let pp = super::preprocess(expr).unwrap();
        assert_eq!(&format!("{}", pp), expected);
    }

    #[test]
    fn preprocessor_comments() {
        test_preprocess(
            "helo // jalo",
            "helo        "
        );
        test_preprocess(
            "helo // jalo\nfoo",
            "helo        \nfoo"
        );
        test_preprocess(
            "helo /* jalo */\nfoo",
            "helo           \nfoo"
        );
        test_preprocess(
            "helo /* jalo \n*/foo",
            "helo            foo"
        );
        test_preprocess(
            "helo /* // */foo",
            "helo         foo"
        );
    }

    #[test]
    fn parse_circomlib() {
        let paths = read_dir("./test").unwrap();
        for path in paths {
            let path = path.unwrap().path();
            if path.is_file() {
                println!("+++ testing {} +++", path.display());
                let mut file = File::open(path).expect("Unable to open the file");
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Unable to read the file");
                if let Err(err) = super::parse(&contents) {
                    panic!("{:?}", err);
                }
            }
        }
    }
}