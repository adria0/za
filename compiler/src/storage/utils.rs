use super::error::Result;
use super::types::*;
use crate::algebra::SignalId;
use circom2_parser::ast::SignalType;

pub fn is_public_input(signal: &Signal) -> bool {
    let component_len = signal.full_name.0.chars().filter(|ch| *ch == '.').count();
    component_len == 1
        && (signal.xtype == SignalType::Output || signal.xtype == SignalType::PublicInput)
}

pub fn is_main_component_input(signal: &Signal) -> bool {
    let component_len = signal.full_name.0.chars().filter(|ch| *ch == '.').count();
    component_len == 1
        && (signal.xtype == SignalType::Output
            || signal.xtype == SignalType::PublicInput
            || signal.xtype == SignalType::PrivateInput)
}

pub fn public_inputs<S: Signals>(signals: &S) -> Result<Vec<String>> {
    let mut inputs = Vec::new();
    for i in 1..signals.len()? {
        let signal = signals.get_by_id(i)?.unwrap();
        if is_public_input(&signal) {
            inputs.push(signal.full_name.to_string());
        }
    }
    Ok(inputs)
}

// TODO join with precious one
pub fn public_inputs_ids<S: Signals>(signals: &S) -> Result<Vec<SignalId>> {
    let mut inputs = Vec::new();
    for i in 1..signals.len()? {
        let signal = signals.get_by_id(i)?.unwrap();
        if is_public_input(&signal) {
            inputs.push(i);
        }
    }
    Ok(inputs)
}

pub fn main_component_inputs_ids<S: Signals>(signals: &S) -> Result<Vec<SignalId>> {
    let mut inputs = Vec::new();
    for i in 1..signals.len()? {
        let signal = signals.get_by_id(i)?.unwrap();
        if is_main_component_input(&signal) {
            inputs.push(i);
        }
    }
    Ok(inputs)
}
