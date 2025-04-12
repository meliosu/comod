use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Model {
    pub variables: HashMap<String, Variable>,
    pub operations: HashMap<String, Operation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variable {
    #[serde(rename = "type")]
    pub ty: Type,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Operation {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub module: Module,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Module {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Ptr,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = match self {
            Type::I8 => "i8",
            Type::U8 => "u8",
            Type::I16 => "i16",
            Type::U16 => "u16",
            Type::I32 => "i32",
            Type::U32 => "u32",
            Type::I64 => "i64",
            Type::U64 => "u64",
            Type::F32 => "f32",
            Type::F64 => "f64",
            Type::Ptr => "ptr",
        };

        write!(f, "{}", ty)
    }
}
