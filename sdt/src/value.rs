use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    error::SdtError,
    utils::{create_random, digest}, node::{SdtBranch, SdtNodeKind, SdtNode},
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
pub enum SdtValues {
    Value(SdtValueKind),
    Branch(HashMap<String, SdtValues>),
}

impl SdtValue {
    pub fn new(value: SdtValueKind) -> Self {
        let salt = hex::encode(create_random::<16>()).to_owned();
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

impl SdtValues {
    pub fn parse_json(&self) -> Result<SdtNode, SdtError>{
        let mut branch = SdtBranch::new();
        match &self {
            SdtValues::Value(val) => {
                
            },
            SdtValues::Branch(map) => {
                for (k, v) in map{
                   match v {
                    SdtValues::Value(val) => {
                        branch.add(k, SdtNodeKind::new_value(val.to_owned())?);
                    },
                    SdtValues::Branch(_) => {
                        branch.add(k, SdtNodeKind::Node(v.parse_json()?));
                    },
                }
                
                }
            },
        }
        return branch.build()    
    } 
}
