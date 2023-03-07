use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    error::SdtError,
    utils::{create_random, digest, parse_query},
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
pub enum SdtNode {
    Value(SdtValue),
    Branch(BTreeMap<String, SdtNode>),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtSalt {
    Value(String),
    Branch(BTreeMap<String, SdtSalt> ),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtProof {
    Value(String),
    Branch(BTreeMap<String, SdtProof>),
}

/*pub fn disclose(result: &mut SdtNode, query: &str) -> Result<(), SdtError> {
    let query_keys = parse_query(query);
    let mut queue: Vec<(String, &mut SdtNode)> = vec![("/".to_owned(), result)];
    while let Some((path, cn)) = queue.pop() {
        if !query_keys.contains(&path) {
            let matched = query_keys.iter().any(|x| x.starts_with(&path));
            if matched {
                if let Branch(v) = &mut cn {
                    if let  SdtNode::Branch (children)= v  {
                        for (key, n) in children {
                            queue.push((format!("{}{}/", path, key.to_owned()), n));
                        }
                    }
                }
            } else {
                cn.body = None;
            }
        }
    }
    Ok(())
}*/


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdt_test() {
        let mut personal_map: BTreeMap<String, SdtNode> = BTreeMap::new();
        personal_map.insert("name".to_owned(), SdtNode::Value(SdtValue::Bool(true)));
        let personal = SdtNode::Branch(personal_map);
        let mut root_map: BTreeMap<String, SdtNode> = BTreeMap::new();
        root_map.insert("personal".to_owned(), personal);
        let root = SdtNode::Branch(root_map);

        let mut personal_salt_map: BTreeMap<String, SdtSalt> = BTreeMap::new();
        personal_salt_map.insert("name".to_owned(), SdtSalt::Value("SALT".to_owned()));
        let personal_salt = SdtSalt::Branch(personal_salt_map);
        let mut root_salt_map: BTreeMap<String, SdtSalt> = BTreeMap::new();
        root_salt_map.insert("personal".to_owned(), personal_salt);

        let salt = SdtSalt::Branch(root_salt_map);
        /*let mut personal: BTreeMap<String, SdtNode> = BTreeMap::new();
        let personal_name = SdtNode{
            proof: None,
            salt: Some("a".to_owned()),
            value: Some()
        };
        personal.insert("name".to_owned(), personal_name);
        let personal =  SdtNode{
            proof: None,
            value: Some(SdtNodeKind::Branch(personal))
        };
        let root = SdtNode{
            proof: None,
            value: Some(SdtNodeKind::Branch(personal))
        };*/
        eprintln!("{}", serde_json::to_string_pretty(&root).unwrap());
        eprintln!("{}", serde_json::to_string_pretty(&salt).unwrap());
        //let node = SdtNode()
    }
}
