use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    error::SdtError,
    utils::{create_random, digest},
};
use serde_json::{Number, Value};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtNodeKind {
    Value(SdtValue),
    Branch(BTreeMap<String, SdtNode>),
}

#[skip_serializing_none]
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNode {
    pub proof: Option<String>,
    pub value: Option<SdtNodeKind>,
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdt_test() {
        let mut personal: BTreeMap<String, SdtNode> = BTreeMap::new();
        let personal_name = SdtNode{
            proof: None,
            value: Some(SdtNodeKind::Value(SdtValue::String("Adem".to_owned())))
        };
        personal.insert("name".to_owned(), personal_name);
        let personal =  SdtNode{
            proof: None,
            value: Some(SdtNodeKind::Branch(personal_name))
        };
        let root = SdtNode{
            proof: None,
            value: Some(SdtNodeKind::Branch(personal))
        };
        eprintln!("{}", serde_json::to_string_pretty(&root).unwrap());
        //let node = SdtNode()
    }
}
