use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    error::SdtError,
    utils::{create_random, digest},
};
use serde_json::Number;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtValue {
    Bool(bool),
    Number(Number),
    String(String),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum EventKind {
    #[serde(rename = "C")]
    Create { value: SdtValue },
    #[serde(rename = "U")]
    Update { value: SdtValue },
    #[serde(rename = "R")]
    Revoke,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNodeValue {
    #[serde(flatten)]
    event: EventKind,
    salt: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtNodeKind {
    Value(SdtNodeValue),
    Branch(Vec<SdtNode>),
}

#[skip_serializing_none]
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNode {
    pub key: String,
    pub proof: Option<String>,
    pub inner: Option<SdtNodeKind>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtPayload {
    Body(BTreeMap<String, String>),
}

impl SdtNode {
    pub fn new() -> Self {
        Self {
            key: "".to_owned(),
            proof: None,
            inner: Some(SdtNodeKind::Branch(vec![])),
        }
    }

    pub fn create_branch(&mut self, key: &str) -> &mut SdtNode {
        let node = Self {
            key: key.to_string(),
            proof: None,
            inner: Some(SdtNodeKind::Branch(vec![])),
        };
        if let SdtNodeKind::Branch(children) = self.inner.as_mut().unwrap() {
            children.push(node);
            children.last_mut().unwrap()
        } else {
            panic!("Can't add branch");
        }
    }

    pub fn create_value(&mut self, key: &str, event: EventKind) -> &mut SdtNode {
        let salt = hex::encode(create_random::<16>()).to_owned();
        let value = SdtNodeValue{
            event: event,
            salt: salt
        };
        let node = Self {
            key: key.to_string(),
            proof: None,
            inner: Some(SdtNodeKind::Value(value)) 
        };
        if let SdtNodeKind::Branch(children) = self.inner.as_mut().unwrap() {
            children.push(node);
        } else {
            panic!("Can't add claim");
        }
        self
    }

    pub fn gen_proof(&mut self) -> Result<String, SdtError> {
        let digest = match self.inner.as_mut().unwrap() {
            SdtNodeKind::Branch(children) => {
                let mut body: BTreeMap<String, String> = BTreeMap::new();
                for child in children {
                    body.insert(child.key.to_owned(), child.gen_proof()?);
                }
                let payload = SdtPayload::Body(body);
                digest(&payload)?
            }
            val => digest(&val)?,
        };
        self.proof = Some(digest.clone());
        Ok(digest)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn sdt_test() {
        let a_value = EventKind::Create {
            value: SdtValue::String("Adem".to_owned()),
        };
        let mut root = SdtNode::new();
        let personal = root.create_branch("personal");

        personal
            .create_value("surname", a_value.clone())
            .create_value("name", a_value.clone());
        let addresses = personal.create_branch("addresses");
        addresses.create_value("work", a_value.clone());
        let keys = root.create_branch("keys");
        let assertions = keys.create_branch("assertions");
        assertions.create_value("key-1", a_value);
        eprintln!("{}", root.gen_proof().unwrap());
        eprintln!("{}", root.gen_proof().unwrap());
        eprintln!("{}", serde_json::to_string(&root).unwrap());
        eprintln!("--------------------------");
    }
}
