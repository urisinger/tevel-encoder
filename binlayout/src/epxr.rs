use std::{collections::HashMap, hash::Hash, io};

use crate::value::Value;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct LayoutId(u32);

impl LayoutId {
    pub fn new(id: u32) -> Self {
        LayoutId(id)
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub layouts: HashMap<LayoutId, Struct>,
    pub layout_ids: HashMap<String, LayoutId>,
}

impl Expr {
    pub fn size_of(&self, id: LayoutId) -> Option<usize> {
        self.layouts
            .get(&id)?
            .fields
            .iter()
            .map(|(_, ty)| match ty {
                Type::Struct(layout_id) => self.size_of(*layout_id),
                Type::I8 => Some(1),
                Type::I16 => Some(2),
                Type::I32 => Some(4),
                Type::I64 => Some(8),
                Type::F32 => Some(4),
                Type::F64 => Some(8),
            })
            .sum()
    }

    pub fn get_id(&self, name: &str) -> Option<LayoutId> {
        self.layout_ids.get(name).copied()
    }

    pub fn get_type(&self, id: LayoutId) -> Option<&Struct> {
        self.layouts.get(&id)
    }

    pub fn get(&self, name: &str) -> Option<&Struct> {
        self.get_id(name).and_then(|id| self.get_type(id))
    }

    pub fn read_value(&self, buf: &[u8], layout_id: LayoutId) -> Option<Value> {
        self.read_value_helper(buf, layout_id).map(|v| v.0)
    }
    /// Reads a `Value` from a byte buffer.
    fn read_value_helper(&self, buf: &[u8], layout_id: LayoutId) -> Option<(Value, usize)> {
        let mut offset = 0;
        let layout = self.get_type(layout_id)?;

        let mut fields = Vec::with_capacity(layout.fields.len());
        for (name, ty) in &layout.fields {
            let val = match ty {
                Type::I8 => {
                    if buf.len() < offset + 1 {
                        return None;
                    }
                    let val = i8::from_le_bytes([buf[offset]]);
                    offset += 1;
                    Value::I8(val as i64)
                }
                Type::I16 => {
                    if buf.len() < offset + 2 {
                        return None;
                    }
                    let val = i16::from_le_bytes(buf[offset..offset + 2].try_into().unwrap());
                    offset += 2;
                    Value::I16(val as i64)
                }
                Type::I32 => {
                    if buf.len() < offset + 4 {
                        return None;
                    }
                    let val = i32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap());
                    offset += 4;
                    Value::I32(val as i64)
                }
                Type::I64 => {
                    if buf.len() < offset + 8 {
                        return None;
                    }
                    let val = i64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
                    offset += 8;
                    Value::I64(val)
                }
                Type::F32 => {
                    if buf.len() < offset + 4 {
                        return None;
                    }
                    let val = f32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap());
                    offset += 4;
                    Value::F32(val as f64)
                }
                Type::F64 => {
                    if buf.len() < offset + 8 {
                        return None;
                    }
                    let val = f64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
                    offset += 8;
                    Value::F64(val)
                }
                Type::Struct(inner_id) => {
                    let (val, added_offset) = self.read_value_helper(&buf[offset..], *inner_id)?;
                    offset += added_offset;
                    val
                }
            };
            fields.push((name.to_string(), val));
        }

        Some((Value::Struct { fields }, offset))
    }
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Struct(LayoutId),
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}
