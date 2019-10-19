extern crate rand;

use pairing::bn256::Bn256;

use circom2_compiler::algebra::{Value, FS, LC, QEQ, SignalId};
use circom2_compiler::storage::Constraints;
use circom2_compiler::storage::Ram;
use circom2_compiler::storage::RamConstraints;
use circom2_compiler::storage::StorageFactory;

use bellman::LinearCombination;

use std::io::{Read, Write};

use bellman::groth16::{Parameters, Proof};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ff_ce::PrimeField;
use pairing::Engine;

use error::{Error, Result};
use serde_cbor::{from_slice, to_vec};
use serde_json;

use super::error;

type G1JsonStruct = [String; 2];
type G2JsonStruct = [[String; 2]; 2];

fn str_to_fq(s: &str) -> Result<pairing::bn256::Fq> {
    let fsstr = FS::parse(&s)?.to_string();
    Ok(pairing::bn256::Fq::from_str(&fsstr).unwrap())
}

fn g1_jstruct_to_bellman(
    g1: &G1JsonStruct,
) -> Result<<Bn256 as bellman::pairing::Engine>::G1Affine> {
    let (x, y) = (str_to_fq(&g1[0])?, str_to_fq(&g1[1])?);
    let p = <Bn256 as bellman::pairing::Engine>::G1Affine::try_from_coordinates(x, y);
    Ok(p.ok_or_else(|| Error::BadFormat(format!("bad coordinates ({},{})", x, y)))?)
}

fn g1_bellman_to_jstruct(
    g1: &<Bn256 as bellman::pairing::Engine>::G1Affine,
) -> Result<G1JsonStruct> {
    let invalid_point_error = || Error::BadFormat("invalid point".to_string());
    let (x, y) = g1.try_to_coordinates().ok_or_else(invalid_point_error)?;
    Ok([x.into_repr().to_string(), y.into_repr().to_string()])
}

fn g2_jstruct_to_bellman(
    g2: &G2JsonStruct,
) -> Result<<Bn256 as bellman::pairing::Engine>::G2Affine> {
    let x = pairing::bn256::Fq2 {
        c0: str_to_fq(&g2[0][0])?,
        c1: str_to_fq(&g2[0][1])?,
    };
    let y = pairing::bn256::Fq2 {
        c0: str_to_fq(&g2[1][0])?,
        c1: str_to_fq(&g2[1][1])?,
    };
    let p = <Bn256 as bellman::pairing::Engine>::G2Affine::try_from_coordinates(x, y);
    Ok(p.ok_or_else(|| Error::BadFormat(format!("bad coordinates ({},{})", x, y)))?)
}

fn g2_bellman_to_jstruct(
    g2: &<Bn256 as bellman::pairing::Engine>::G2Affine,
) -> Result<G2JsonStruct> {
    let invalid_point_error = || Error::BadFormat("invalid point".to_string());
    let (x, y) = g2.try_to_coordinates().ok_or_else(invalid_point_error)?;
    Ok([
        [x.c0.into_repr().to_string(), x.c1.into_repr().to_string()],
        [y.c0.into_repr().to_string(), y.c1.into_repr().to_string()],
    ])
}

#[derive(Serialize, Deserialize)]
pub struct JsonProofAndInput(G1JsonStruct, G2JsonStruct, G1JsonStruct, Vec<String>);

impl JsonProofAndInput {
    pub fn from_bellman(proof: Proof<Bn256>, public_input: Vec<(String, FS)>) -> Result<Self> {
        Ok(JsonProofAndInput(
            g1_bellman_to_jstruct(&proof.a)?,
            g2_bellman_to_jstruct(&proof.b)?,
            g1_bellman_to_jstruct(&proof.c)?,
            public_input
                .into_iter()
                .map(|(_, v)| v.to_string())
                .collect::<Vec<_>>(),
        ))
    }

    pub fn to_bellman(json: &str) -> Result<(Proof<Bn256>, Vec<pairing::bn256::Fr>)> {
        let JsonProofAndInput(a, b, c, inputs) = serde_json::from_str(json)?;
        let proof = Proof {
            a: g1_jstruct_to_bellman(&a)?,
            b: g2_jstruct_to_bellman(&b)?,
            c: g1_jstruct_to_bellman(&c)?,
        };

        let err_bad_format = || Error::BadFormat("bad format".to_string());
        let parsed_inputs = inputs
            .iter()
            .map(|s| pairing::bn256::Fr::from_str(s).ok_or_else(err_bad_format))
            .collect::<Result<Vec<_>>>()?;

        Ok((proof, parsed_inputs))
    }

    pub fn write<W: Write>(&self, out: &mut W) -> Result<()> {
        let json = serde_json::to_string(self)?;
        out.write(json.as_bytes())?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonVerifyingKey {
    pub(crate) alpha_g1: G1JsonStruct,
    pub(crate) beta_g1: G1JsonStruct,
    pub(crate) beta_g2: G2JsonStruct,
    pub(crate) delta_g1: G1JsonStruct,
    pub(crate) delta_g2: G2JsonStruct,
    pub(crate) gamma_g2: G2JsonStruct,
    pub(crate) ic: Vec<G1JsonStruct>,
    pub(crate) input_names: Vec<String>,
}

impl JsonVerifyingKey {
    pub fn from_bellman(vk: &bellman::groth16::VerifyingKey<Bn256>) -> Result<Self> {
        let ic = vk
            .ic
            .iter()
            .map(g1_bellman_to_jstruct)
            .collect::<Result<Vec<_>>>()?;

        Ok(JsonVerifyingKey {
            alpha_g1: g1_bellman_to_jstruct(&vk.alpha_g1)?,
            beta_g1: g1_bellman_to_jstruct(&vk.beta_g1)?,
            beta_g2: g2_bellman_to_jstruct(&vk.beta_g2)?,
            delta_g1: g1_bellman_to_jstruct(&vk.delta_g1)?,
            delta_g2: g2_bellman_to_jstruct(&vk.delta_g2)?,
            gamma_g2: g2_bellman_to_jstruct(&vk.gamma_g2)?,
            ic,
            input_names: Vec::new(),
        })
    }

    pub fn with_input_names(self, input_names: Vec<String>) -> JsonVerifyingKey {
        JsonVerifyingKey {
            input_names,
            ..self
        }
    }

    pub fn to_bellman(&self) -> Result<bellman::groth16::VerifyingKey<Bn256>> {
        let ic = self
            .ic
            .iter()
            .map(g1_jstruct_to_bellman)
            .collect::<Result<Vec<_>>>()?;

        Ok(bellman::groth16::VerifyingKey {
            alpha_g1: g1_jstruct_to_bellman(&self.alpha_g1)?,
            beta_g1: g1_jstruct_to_bellman(&self.beta_g1)?,
            beta_g2: g2_jstruct_to_bellman(&self.beta_g2)?,
            delta_g1: g1_jstruct_to_bellman(&self.delta_g1)?,
            delta_g2: g2_jstruct_to_bellman(&self.delta_g2)?,
            gamma_g2: g2_jstruct_to_bellman(&self.gamma_g2)?,
            ic: ic,
        })
    }

    pub fn from_json(json: &str) -> Result<JsonVerifyingKey> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }
}

pub fn value_to_bellman_fr<E: Engine>(value: &Value) -> E::Fr {
    match value {
        Value::FieldScalar(fs) => fs_to_bellman_fr::<E>(fs),
        _ => panic!("Invalid signal value"),
    }
}

pub fn fs_to_bellman_fr<E: Engine>(fe: &FS) -> E::Fr {
    E::Fr::from_str(&fe.to_string()).unwrap()
}

pub fn lc_to_bellman<E: Engine>(
    mut base: LinearCombination<E>,
    signals: &[Option<bellman::Variable>],
    lc: &LC,
) -> LinearCombination<E> {
    use std::ops::Add;
    for (s, v) in &lc.0 {
        let signal = signals[*s];
        if signal.is_none() {
            panic!("signal {} not defined",*s);
        }
        base = base.add((fs_to_bellman_fr::<E>(&v), signal.unwrap()));
    }
    base
}

pub fn write_pk<W: Write, C: Constraints>(
    mut pk: W,
    constraints: &C,
    ignore_signals: &[SignalId],
    params: &Parameters<Bn256>,
) -> Result<()> {
    // write constrains
    pk.write_u32::<BigEndian>(constraints.len()? as u32)?;
    for i in 0..constraints.len()? {
        let qeq = to_vec(&constraints.get(i)?)?;
        pk.write_u32::<BigEndian>(qeq.len() as u32)?;
        pk.write(&qeq)?;
    }

    // write signal aliases
    pk.write_u32::<BigEndian>(ignore_signals.len() as u32)?;
    for i in 0..ignore_signals.len() {
        pk.write_u32::<BigEndian>(ignore_signals[i] as u32)?;
    }

    // write signalid
    params.write(pk)?;
    Ok(())
}

pub fn read_pk<R: Read>(mut pk: R) -> Result<(RamConstraints, Vec<SignalId>, Parameters<Bn256>)> {
    let mut buffer = Vec::with_capacity(1024);
    let mut constraints = Ram::default().new_constraints()?;

    // read constraints
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

    // read signal aliases
    let count = pk.read_u32::<BigEndian>()?;
    let mut ignore_signals = Vec::with_capacity(count as usize);
    for _ in 0..count {
        ignore_signals.push(pk.read_u32::<BigEndian>()? as SignalId);
    }

    // read proving key
    let params: Parameters<Bn256> = Parameters::read(pk, true)?;

    Ok((constraints, ignore_signals, params))
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
                let value = value
                    .as_u64()
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
