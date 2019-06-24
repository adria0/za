extern crate rand;

use lazy_static::lazy_static;
use pairing::bn256::{Bn256, Fr};
use regex::Regex;

/*
Taken from Thibaut Schaeffer's ZoKrates
https://github.com/Zokrates/ZoKrates/commit/20790b72fff3b48a518dd7b910f7e005612faf95
*/

lazy_static! {
    static ref G2_REGEX: Regex = Regex::new(r"G2\(x=Fq2\(Fq\((?P<x0>0[xX][0-9a-fA-F]{64})\) \+ Fq\((?P<x1>0[xX][0-9a-fA-F]{64})\) \* u\), y=Fq2\(Fq\((?P<y0>0[xX][0-9a-fA-F]{64})\) \+ Fq\((?P<y1>0[xX][0-9a-fA-F]{64})\) \* u\)\)").unwrap();
}

lazy_static! {
    static ref G1_REGEX: Regex = Regex::new(
        r"G1\(x=Fq\((?P<x>0[xX][0-9a-fA-F]{64})\), y=Fq\((?P<y>0[xX][0-9a-fA-F]{64})\)\)"
    )
    .unwrap();
}

lazy_static! {
    static ref FR_REGEX: Regex = Regex::new(r"Fr\((?P<x>0[xX][0-9a-fA-F]{64})\)").unwrap();
}

fn parse_g1(e: &<Bn256 as bellman::pairing::Engine>::G1Affine) -> (String, String) {
    let raw_e = e.to_string();

    let captures = G1_REGEX.captures(&raw_e).unwrap();

    (
        captures.name(&"x").unwrap().as_str().to_string(),
        captures.name(&"y").unwrap().as_str().to_string(),
    )
}

fn parse_g2(e: &<Bn256 as bellman::pairing::Engine>::G2Affine) -> (String, String, String, String) {
    let raw_e = e.to_string();

    let captures = G2_REGEX.captures(&raw_e).unwrap();

    (
        captures.name(&"x1").unwrap().as_str().to_string(),
        captures.name(&"x0").unwrap().as_str().to_string(),
        captures.name(&"y1").unwrap().as_str().to_string(),
        captures.name(&"y0").unwrap().as_str().to_string(),
    )
}

fn parse_fr(e: &Fr) -> String {
    let raw_e = e.to_string();

    let captures = FR_REGEX.captures(&raw_e).unwrap();

    captures.name(&"x").unwrap().as_str().to_string()
}

pub fn parse_g1_json(e: &<Bn256 as bellman::pairing::Engine>::G1Affine) -> String {
    let parsed = parse_g1(e);

    format!("[\"{}\", \"{}\"]", parsed.0, parsed.1)
}

pub fn parse_g2_json(e: &<Bn256 as bellman::pairing::Engine>::G2Affine) -> String {
    let parsed = parse_g2(e);

    format!(
        "[[\"{}\", \"{}\"], [\"{}\", \"{}\"]]",
        parsed.0, parsed.1, parsed.2, parsed.3,
    )
}

pub fn parse_fr_json(e: &Fr) -> String {
    let parsed = parse_fr(e);

    format!("\"{}\"", parsed)
}

pub fn parse_g1_hex(e: &<Bn256 as bellman::pairing::Engine>::G1Affine) -> String {
    let parsed = parse_g1(e);

    format!("{}, {}", parsed.0, parsed.1)
}

pub fn parse_g2_hex(e: &<Bn256 as bellman::pairing::Engine>::G2Affine) -> String {
    let parsed = parse_g2(e);

    format!("[{}, {}], [{}, {}]", parsed.0, parsed.1, parsed.2, parsed.3,)
}
