use std::io;
use std::mem;

use std::fmt;

use crate::epxr::{Expr, LayoutId, Type};

pub enum Value {
    Struct { fields: Vec<(String, Value)> },
    I8(i64),
    I16(i64),
    I32(i64),
    I64(i64),
    F32(f64),
    F64(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::I8(v) => write!(f, "{}", v),
            Value::I16(v) => write!(f, "{}", v),
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{:.6}", v), // Limits float precision for readability
            Value::F64(v) => write!(f, "{:.6}", v),
            Value::Struct { fields } => {
                writeln!(f, "{{")?;
                for (name, value) in fields {
                    let indented_value = format!("{}", value)
                        .lines()
                        .map(|line| format!("  {}", line)) // Indent each line for nested structs
                        .collect::<Vec<_>>()
                        .join("\n");
                    writeln!(f, "  {}: {}", name, indented_value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Value {
    pub fn size(&self) -> usize {
        match self {
            Value::I8(_) => mem::size_of::<i8>(),
            Value::I16(_) => mem::size_of::<i16>(),
            Value::I32(_) => mem::size_of::<i32>(),
            Value::I64(_) => mem::size_of::<i64>(),
            Value::F32(_) => mem::size_of::<f32>(),
            Value::F64(_) => mem::size_of::<f64>(),
            Value::Struct { fields } => {
                // Ensure a consistent order by sorting the field names.
                fields.iter().map(|(_, val)| val.size()).sum()
            }
        }
    }

    /// Write the byte representation of this value into the provided buffer.
    /// Returns the number of bytes written.
    fn write_into(&self, buf: &mut [u8]) -> usize {
        let mut offset = 0;
        match self {
            Value::I8(val) => {
                let v = *val as i8;
                let bytes = v.to_le_bytes();
                buf[..bytes.len()].copy_from_slice(&bytes);
                offset += bytes.len();
            }
            Value::I16(val) => {
                let v = *val as i16;
                let bytes = v.to_le_bytes();
                buf[..bytes.len()].copy_from_slice(&bytes);
                offset += bytes.len();
            }
            Value::I32(val) => {
                let v = *val as i32;
                let bytes = v.to_le_bytes();
                buf[..bytes.len()].copy_from_slice(&bytes);
                offset += bytes.len();
            }
            Value::I64(val) => {
                let bytes = val.to_le_bytes();
                buf[..bytes.len()].copy_from_slice(&bytes);
                offset += bytes.len();
            }
            Value::F32(val) => {
                let v = *val as f32;
                let bytes = v.to_le_bytes();
                buf[..bytes.len()].copy_from_slice(&bytes);
                offset += bytes.len();
            }
            Value::F64(val) => {
                let bytes = val.to_le_bytes();
                buf[..bytes.len()].copy_from_slice(&bytes);
                offset += bytes.len();
            }
            Value::Struct { fields } => {
                for (_, value) in fields {
                    let written = value.write_into(&mut buf[offset..]);
                    offset += written;
                }
            }
        }
        offset
    }

    /// Format the value into a Vec<u8> with minimal allocations.
    pub fn encode_value(&self) -> Vec<u8> {
        let total_size = self.size();
        let mut buf = vec![0u8; total_size];
        self.write_into(&mut buf);
        buf
    }

    pub fn prompt_for_value(expr: &Expr, id: LayoutId) -> Option<Value> {
        Self::prompt_for_value_helper(expr, id, "")
    }

    fn prompt_for_value_helper(expr: &Expr, id: LayoutId, prefix: &str) -> Option<Value> {
        let layout = expr.layouts.get(&id)?;
        let mut values = Vec::new();
        let mut input = String::new();

        for (field_name, field_type) in &layout.fields {
            let full_field_name = format!("{}{}", prefix, field_name);
            let val = match field_type {
                Type::I8 | Type::I16 | Type::I32 | Type::I64 => {
                    println!("Enter integer value for {}:", full_field_name);
                    input.clear();
                    io::stdin().read_line(&mut input).ok()?;
                    if let Ok(value) = input.trim().parse::<i64>() {
                        match field_type {
                            Type::I8 => Value::I8(value),
                            Type::I16 => Value::I16(value),
                            Type::I32 => Value::I32(value),
                            Type::I64 => Value::I64(value),
                            _ => unreachable!(),
                        }
                    } else {
                        println!("Invalid input. Expected an integer.");
                        return None;
                    }
                }
                Type::F32 | Type::F64 => {
                    println!("Enter floating-point value for {}:", full_field_name);
                    input.clear();
                    io::stdin().read_line(&mut input).ok()?;
                    if let Ok(value) = input.trim().parse::<f64>() {
                        match field_type {
                            Type::F32 => Value::F32(value),
                            Type::F64 => Value::F64(value),
                            _ => unreachable!(),
                        }
                    } else {
                        println!("Invalid input. Expected a floating-point number.");
                        return None;
                    }
                }
                Type::Struct(inner_id) => Self::prompt_for_value_helper(
                    expr,
                    *inner_id,
                    &format!("{}.", full_field_name),
                )?,
            };
            values.push((field_name.to_string(), val));
        }
        Some(Value::Struct { fields: values })
    }
}
