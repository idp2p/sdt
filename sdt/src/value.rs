use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    error::SdtError,
    node::{SdtBranch, SdtNode},
    utils::{create_random, digest},
};
use serde_json::Number;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtValueKind {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtValue {
    pub salt: String,
    pub value: SdtValueKind,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtProofPayload(BTreeMap<String, SdtValueKind>);

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtClaim {
    Value(SdtValueKind),
    Branch(HashMap<String, SdtClaim>),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtResult {
    Values(Vec<SdtValueKind>),
    Branch(HashMap<String, SdtResult>),
}

impl SdtValue {
    pub fn new(value: SdtValueKind) -> Self {
        let raw = hex::encode(create_random::<16>()).to_owned();
        let salt = format!("0x{raw}");
        Self { salt, value }
    }

    pub fn gen_proof(&self) -> Result<String, SdtError> {
        let proof_map = SdtProofPayload::new()
            .insert_str("salt", &self.salt)
            .insert("value", self.value.clone())
            .build();
        digest(&proof_map)
    }
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

impl SdtProofPayload {
    pub fn new() -> Self {
        let body: BTreeMap<String, SdtValueKind> = BTreeMap::new();
        Self(body)
    }

    pub fn insert(&mut self, key: &str, value: SdtValueKind) -> &mut Self {
        self.0.insert(key.to_owned(), value);
        self
    }

    pub fn insert_str(&mut self, key: &str, s: &str) -> &mut Self {
        self.insert(key, SdtValueKind::String(s.to_owned()))
    }

    pub fn insert_i64(&mut self, key: &str, v: i64) -> &mut Self {
        self.insert(key, SdtValueKind::new_i64(v))
    }

    pub fn build(&mut self) -> Self {
        self.to_owned()
    }
}


impl SdtValueKind {
    pub fn new_i64(number: i64) -> Self {
        SdtValueKind::Number(Number::from(number))
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
