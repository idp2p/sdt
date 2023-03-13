use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{value::SdtValueKind, node::{SdtNode}};


#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtClaim {
    Value(SdtValueKind),
    Node(HashMap<String, SdtClaim>),
}


impl SdtClaim {
    pub fn to_node(&self) -> SdtNode {
        let mut node = SdtNode::new();
        if let SdtClaim::Node(map) = &self {
            for (k, v) in map {
                match v {
                    SdtClaim::Value(val) => {
                        node.add_value(k, val.to_owned());
                    }
                    SdtClaim::Node(_) => {
                        node.add_node(k, v.to_node());
                    }
                }
            }
        }
        return node.build();
    }
}

#[cfg(test)]
mod tests {
    use crate::error::SdtError;

    use super::*;

    #[test]
    fn from_json_test() -> Result<(), SdtError> {
        let s = r#"{
            "personal": {
               "name": "Adem",
               "age": 5
            },
            "keys": {
               "assertions": {
                  "key-1": "0x12...."
               }
            }
        }"#;
        let claim: SdtClaim = serde_json::from_str(s)?;
        let node = claim.to_node();
        assert!(!node.gen_proof()?.is_empty());
        Ok(())
    }
}