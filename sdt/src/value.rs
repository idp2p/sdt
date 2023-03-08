use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    error::SdtError,
    node::{SdtBranch, SdtNode, SdtNodeKind},
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
pub struct SdtPayload(pub BTreeMap<String, SdtValueKind>);

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
        let mut body: BTreeMap<String, SdtValueKind> = BTreeMap::new();
        body.insert("salt".to_owned(), SdtValueKind::String(self.salt.clone()));
        body.insert("value".to_owned(), self.value.clone());
        let payload = SdtPayload(body);
        digest(&payload)
    }
}

impl SdtClaim {
    pub fn parse_json(&self) -> Result<SdtNode, SdtError> {
        let mut branch = SdtBranch::new();
        if let SdtClaim::Branch(map) = &self {
            for (k, v) in map {
                match v {
                    SdtClaim::Value(val) => {
                        branch.add(k, SdtNodeKind::new_value(val.to_owned())?);
                    }
                    SdtClaim::Branch(_) => {
                        branch.add(k, SdtNodeKind::Node(v.parse_json()?));
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
    fn parse_test() -> Result<(), SdtError> {
        let query = r#"
            {
                "personal": {
                    "name": "Adem",
                    "surname": "Çağlın"
                }
            }"#;

        let result_str = r#"
            {
                "personal": {
                    "name": ["Adem", "Adem2"],
                    "surname": ["Çağlın", null]
                }
            }"#;

        let result: SdtResult = serde_json::from_str(result_str)?;
        eprintln!("{}", serde_json::to_string_pretty(&result)?);
        Ok(())
    }
}
