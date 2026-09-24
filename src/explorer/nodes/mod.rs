use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str;
use serde::{Deserialize, Serialize};
use serde_json::Result;

pub mod node;
pub use node::{Node,Type};

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
struct AuxType {
    r#type: Type,
}

#[derive(Debug)]
pub struct Nodes(pub Vec<Node>);

impl Nodes {
    pub fn new(data_raw: Vec<&str>, after_context: usize, before_context: usize) -> Self {
        let mut v: Nodes = Nodes(vec![]);
        let mut aux_vecs: Vec<(&str, Type)> = vec![];

        for d in data_raw {
            let t = match Self::parse_type(d) {
                Ok(t) => t,
                Err(_) => continue,
            };
            match t.r#type {
                Type::begin | Type::r#match | Type::summary | Type::context => {
                    aux_vecs.push((d, t.r#type))
                },
                Type::end => {
                    aux_vecs.push((d, t.r#type));
                    if let Some(n) = Node::new(aux_vecs, after_context, before_context) {
                        v.0.push(n);
                    }
                    aux_vecs = vec![];
                }
            }
        }
        v.0.sort_by_key(|a| a.file_name());
        v
    }

    fn parse_type(d: &str) -> Result<AuxType> {
        let n: AuxType = serde_json::from_str(d)?;
        Ok(n)
    }
}

impl Display for Nodes {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.iter().try_fold((), |_, node| writeln!(f, "{}", node))
    }
}

