use std::collections::HashMap;

use peg::str::LineCol;
use thiserror::Error;

use crate::epxr::{Expr, LayoutId, Struct, Type};

peg::parser! {
    pub grammar struct_parser() for str {
        rule identifier() -> String
            = s:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) { s.to_string() }

        rule type_name() -> String
            = identifier()

        rule field() -> (String, String)
            = name:identifier() _ ":" _ type_name:type_name() _ ","? _ { (name, type_name) }

        rule fields() -> Vec<(String, String)>
            = field_list:(field()*) { field_list }

        rule struct_def() -> (String, Vec<(String, String)>)
            = _ "struct" _ name:identifier() _ "{" _ fields:fields() _ "}" { (name, fields) }


        pub rule structs() -> Vec<(String, Vec<(String, String)>)>
            = struct_defs:(struct_def()*) _ { struct_defs }


        rule _() = [' ' | '\n' | '\t']*
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    Parse(#[from] peg::error::ParseError<LineCol>),
    #[error("Unknown type: {0}")]
    UnknownType(String),
}

impl Expr {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let mut layout_ids = HashMap::new();
        let mut layouts = HashMap::new();
        let parsed_structs = struct_parser::structs(input)?;
        for (id_counter, (name, _)) in parsed_structs.iter().enumerate() {
            let id = LayoutId::new(id_counter as u32);
            layout_ids.insert(name.clone(), id);
        }
        for (name, fields) in parsed_structs {
            let id = *layout_ids.get(&name).unwrap();
            let mut parsed_fields = Vec::new();
            for (field_name, field_type) in fields {
                let field_type = match field_type.as_str() {
                    "i8" => Type::I8,
                    "i16" => Type::I16,
                    "i32" => Type::I32,
                    "i64" => Type::I64,
                    "f32" => Type::F32,
                    "f64" => Type::F64,
                    other => {
                        if let Some(layout_id) = layout_ids.get(other) {
                            Type::Struct(*layout_id)
                        } else {
                            return Err(ParseError::UnknownType(other.to_string()));
                        }
                    }
                };
                parsed_fields.push((field_name, field_type));
            }
            layouts.insert(
                id,
                Struct {
                    fields: parsed_fields,
                },
            );
        }
        Ok(Self {
            layouts,
            layout_ids,
        })
    }
}
