pub mod error;
pub mod node;
pub mod utils;
pub mod value;
use error::SdtError;
use node::SdtNode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utils::*;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct Sdt {
    id: String,
    previous: Option<String>,
    node: SdtNode,
}

impl Sdt {
    pub fn new(id: &str, node: SdtNode) -> Self {
        Self {
            id: id.to_owned(),
            previous: None,
            node: node,
        }
    }

    pub fn new_mutation(id: &str, prev: &str, node: SdtNode) -> Self {
        Self {
            id: id.to_owned(),
            previous: Some(prev.to_owned()),
            node: node,
        }
    }

    pub fn gen_proof(&self) -> Result<String, SdtError> {
        digest(&self)
    }

    pub fn disclose_by_query(&mut self, query: &str) -> Result<(), SdtError> {
        self.node.disclose(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdt_test() {
        
    }
}
