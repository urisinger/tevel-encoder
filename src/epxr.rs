use std::{collections::HashMap, hash::Hash, io};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct LayoutId(u32);

impl LayoutId {
    pub fn new(id: u32) -> Self {
        LayoutId(id)
    }
}

#[derive(Debug)]
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
}

#[derive(Debug)]
pub struct Struct {
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug)]
pub enum Type {
    Struct(LayoutId),
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}
