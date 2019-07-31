extern crate rand;

use pairing::bn256::Bn256;
use regex::Regex;

use circom2_compiler::algebra::{Value, FS, LC, QEQ};
use circom2_compiler::storage::Constraints;
use circom2_compiler::storage::Ram;
use circom2_compiler::storage::RamConstraints;
use circom2_compiler::storage::StorageFactory;

use bellman::LinearCombination;

use std::io::{Read, Write};

use bellman::groth16::{Parameters, Proof};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ff::PrimeField;
use pairing::Engine;

use error::{Error, Result};
use serde_cbor::{from_slice, to_vec};
use serde_json;

use super::error;

#[derive(Serialize, Deserialize)]
struct JsonInputAndProof([String; 2],[[String; 2]; 2],[String; 2],Vec<String>);

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

pub fn parse_g1(e: &<Bn256 as bellman::pairing::Engine>::G1Affine) -> (String, String) {
    let raw_e = e.to_string();

    let captures = G1_REGEX.captures(&raw_e).unwrap();

    (
        captures.name(&"x").unwrap().as_str().to_string(),
        captures.name(&"y").unwrap().as_str().to_string(),
    )
}

pub fn parse_g2(e: &<Bn256 as bellman::pairing::Engine>::G2Affine) -> (String, String, String, String) {
    let raw_e = e.to_string();

    let captures = G2_REGEX.captures(&raw_e).unwrap();

    (
        captures.name(&"x1").unwrap().as_str().to_string(),
        captures.name(&"x0").unwrap().as_str().to_string(),
        captures.name(&"y1").unwrap().as_str().to_string(),
        captures.name(&"y0").unwrap().as_str().to_string(),
    )
}

pub fn parse_g1_hex(e: &<Bn256 as bellman::pairing::Engine>::G1Affine) -> String {
    let parsed = parse_g1(e);

    format!("{}, {}", parsed.0, parsed.1)
}

pub fn parse_g2_hex(e: &<Bn256 as bellman::pairing::Engine>::G2Affine) -> String {
    let parsed = parse_g2(e);

    format!("[{}, {}], [{}, {}]", parsed.0, parsed.1, parsed.2, parsed.3,)
}

pub fn value_to_bellman_fr<E: Engine>(value: &Value) -> E::Fr {
    match value {
        Value::FieldScalar(fs) => fe_to_bellman_fr::<E>(fs),
        _ => panic!("Invalid signal value"),
    }
}

pub fn fe_to_bellman_fr<E: Engine>(fe: &FS) -> E::Fr {
    E::Fr::from_str(&fe.to_string()).unwrap()
}

pub fn lc_to_bellman<E: Engine>(
    mut base: LinearCombination<E>,
    signals: &[bellman::Variable],
    lc: &LC,
) -> LinearCombination<E> {
    use std::ops::Add;
    for (s, v) in &lc.0 {
        base = base.add((fe_to_bellman_fr::<E>(&v), signals[*s]));
    }
    base
}

pub fn write_input_and_proof<W: Write>(
    public_input: Vec<(String, FS)>,
    proof: Proof<Bn256>,
    out: &mut W,
) -> Result<()> {
    let a = parse_g1(&proof.a);
    let b = parse_g2(&proof.b);
    let c = parse_g1(&proof.c);

    let json = serde_json::to_string(&JsonInputAndProof(
        [a.0, a.1],
        [[b.0, b.1], [b.2, b.3]],
        [c.0, c.1],
        public_input
            .into_iter()
            .map(|(_, v)| v.to_string())
            .collect::<Vec<_>>(),
    ))?;

    out.write(json.as_bytes())?;
    Ok(())
}

pub fn write_pk<W: Write, C: Constraints>(
    mut pk: W,
    constraints: &C,
    params: &Parameters<Bn256>,
) -> Result<()> {
    // write constratins & proving key
    pk.write_u32::<BigEndian>(constraints.len()? as u32)?;
    for i in 0..constraints.len()? {
        let qeq = to_vec(&constraints.get(i)?)?;
        pk.write_u32::<BigEndian>(qeq.len() as u32)?;
        pk.write(&qeq)?;
    }

    params.write(pk)?;
    Ok(())
}

pub fn read_pk<R: Read>(mut pk: R) -> Result<(RamConstraints, Parameters<Bn256>)> {
    let mut buffer = Vec::with_capacity(1024);
    let mut constraints = Ram::default().new_constraints()?;
    let count = pk.read_u32::<BigEndian>()?;

    for _ in 0..count {
        let len = pk.read_u32::<BigEndian>()? as usize;
        if len > buffer.capacity() {
            buffer.reserve(len - buffer.capacity());
        }
        buffer.resize(len, 0u8);
        pk.read_exact(&mut buffer)?;
        let qeq = from_slice::<QEQ>(&buffer)?;
        constraints.push(qeq, None)?;
    }

    let params: Parameters<Bn256> = Parameters::read(pk, true)?;

    Ok((constraints, params))
}

pub fn flatten_json(prefix: &str, json: &str) -> Result<Vec<(String, FS)>> {

    fn flatten(prefix: &str, v: &serde_json::Value, result: &mut Vec<(String, FS)>) -> Result<()> {
        match v {
            serde_json::Value::Array(values) => {
                for (i, value) in values.iter().enumerate() {
                    flatten(&format!("{}[{}]", prefix, i), value, result)?;
                }
                Ok(())
            }
            serde_json::Value::Object(values) => {
                for (key, value) in values.iter() {
                    flatten(&format!("{}.{}", prefix, key), value, result)?;
                }
                Ok(())
            }
            serde_json::Value::String(value) => {
                let value = FS::parse(value)?;
                result.push((prefix.to_string(), value));
                Ok(())
            }
            serde_json::Value::Number(value) => {
                let value = value.as_u64()
                    .ok_or_else(|| Error::BadFormat(format!("bad value {:?}", value)))?;

                result.push((prefix.to_string(), FS::from(value)));
                Ok(())
            }
            _ => Err(Error::BadFormat(format!("Cannot decode value {:?}", v))),
        }
    }

    let json: serde_json::Value = serde_json::from_str(json)?;

    let mut result = Vec::new();
    flatten(prefix, &json, &mut result)?;
    Ok(result)
}

#[derive(Serialize, Deserialize)]
pub struct JsonVerifyingKey {
    pub(crate) alpha  : [String;2],
    pub(crate) beta   : [[String; 2]; 2],
    pub(crate) gamma  : [[String; 2]; 2],
    pub(crate) delta  : [[String; 2]; 2],
    pub(crate) ic     : Vec<[String;2]>,
    pub(crate) inputs : Vec<String>,
} 

impl JsonVerifyingKey {
    pub fn build(vk : &bellman::groth16::VerifyingKey<Bn256>) -> JsonVerifyingKey {
        let alpha_g1 = parse_g1(&vk.alpha_g1);
        let beta_g2 = parse_g2(&vk.beta_g2);
        let gamma_g2 = parse_g2(&vk.gamma_g2);
        let delta_g2 = parse_g2(&vk.delta_g2);
        let ic : Vec<[String;2]>= vk.ic.iter().map(|e| { 
            let (x,y) = parse_g1(e);
            [x,y]
        }).collect();
        
        JsonVerifyingKey {
            alpha : [alpha_g1.0,alpha_g1.1],
            beta  : [[beta_g2.1,beta_g2.3],[beta_g2.0,beta_g2.2]],
            gamma : [[gamma_g2.1,gamma_g2.3],[gamma_g2.0,gamma_g2.2]],
            delta : [[delta_g2.1,delta_g2.3],[delta_g2.0,delta_g2.2]],
            ic,
            inputs : Vec::new(),
        }
    }  
    pub fn with_inputs(self: JsonVerifyingKey, inputs: Vec<String>) -> JsonVerifyingKey {
        JsonVerifyingKey {
            inputs,
            ..self
        }
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }
}
