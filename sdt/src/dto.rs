use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{value::SdtValueKind, node::{SdtNode, SdtBranch}, error::SdtError};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtDiscloseResult {
    Values(Vec<SdtValueKind>),
    Branch(HashMap<String, SdtDiscloseResult>),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtClaim {
    Value(SdtValueKind),
    Branch(HashMap<String, SdtClaim>),
}

impl SdtClaim {
    pub fn to_node(&self) -> Result<SdtNode, SdtError> {
        let mut branch = SdtBranch::new();
        if let SdtClaim::Branch(map) = &self {
            for (k, v) in map {
                match v {
                    SdtClaim::Value(val) => {
                        branch.add_value(k, val.to_owned())?;
                    }
                    SdtClaim::Branch(_) => {
                        branch.add_node(k, v.to_node()?);
                    }
                }
            }
        }
        return branch.build();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_test() -> Result<(), SdtError> {
        let result_str = r#"
            {
                "personal": {
                    "name": ["Adem", "Adem2"],
                    "surname": ["Çağlın", null]
                }
            }"#;

        serde_json::from_str(result_str)?;
        Ok(())
    }

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
        let node = claim.to_node()?;
        assert!(!node.proof.is_empty());
        Ok(())
    }
}