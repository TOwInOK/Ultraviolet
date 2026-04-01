use lazy_static::lazy_static;
use std::collections::HashMap;
use ultraviolet_core::types::{backend::ControlFlow, frontend::ast::UVValue};

lazy_static! {
    static ref BUILTIN_CONSTANTS: HashMap<&'static str, UVValue> = {
        let mut m = HashMap::new();
        m.insert("endl", UVValue::String("\n".to_string()));
        m
    };
}

/// Check if provided function name is built-in function
pub fn is_builtin_constant(name: &str) -> bool {
    BUILTIN_CONSTANTS.contains_key(name)
}

/// Execute builtin function by signature
pub fn get_builtin_constant(name: &str) -> ControlFlow {
    match BUILTIN_CONSTANTS.get(name) {
        Some(v) => ControlFlow::Simple(v.clone()),
        None => unreachable!(),
    }
}
