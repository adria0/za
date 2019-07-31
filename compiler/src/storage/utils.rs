use circom2_parser::ast::SignalType;
use super::types::*;
use super::error::Result;

pub fn is_public_input(signal: &Signal) -> bool {
    let component_len = signal.full_name.0.chars().filter(|ch| *ch == '.').count();
    component_len == 1
        && (signal.xtype == SignalType::Output || signal.xtype == SignalType::PublicInput)
}

pub fn public_inputs<S:Signals>(signals:&S) -> Result<Vec<String>> {
    let mut inputs = Vec::new();
    for i in 1..signals.len()? {
        let signal = signals.get_by_id(i)?.unwrap();
        if is_public_input(&signal) {
            inputs.push(signal.full_name.to_string());
        }
    }
    Ok(inputs)
}
