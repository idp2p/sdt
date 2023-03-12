use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    dto::{SdtValueResult, SdtClaim},
    error::SdtError,
    proof::SdtProof,
    utils::parse_query,
    value::{SdtValue, SdtValueKind},
};
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNode(HashMap<String, SdtNodeKind>);

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtNodeKind {
    Proof(String),
    Value(SdtValue),
    Node(SdtNode),
}

impl SdtNodeKind {
    pub fn gen_proof(&self) -> Result<String, SdtError> {
        match &self {
            Self::Proof(p) => Ok(p.to_owned()),
            Self::Value(value) => value.gen_proof(),
            Self::Node(children) => children.gen_proof(),
        }
    }
}

impl SdtNode {
    pub fn new() -> Self {
        let map: HashMap<String, SdtNodeKind> = HashMap::new();
        Self(map)
    }

    pub fn add_node(&mut self, key: &str, map: Self) -> &mut Self {
        self.0.insert(key.to_owned(), SdtNodeKind::Node(map));
        self
    }

    pub fn add_str_value(&mut self, key: &str, val: &str) -> &mut Self {
        self.0.insert(
            key.to_owned(),
            SdtNodeKind::Value(SdtValue::new(SdtValueKind::String(val.to_owned()))),
        );
        self
    }

    pub fn add_proof(&mut self, key: &str, proof: &str) -> &mut Self {
        self.0
            .insert(key.to_owned(), SdtNodeKind::Proof(proof.to_owned()));
        self
    }

    pub fn build(&self) -> Self {
        self.to_owned()
    }

    pub fn gen_proof(&self) -> Result<String, SdtError> {
        let mut proof = SdtProof::new();
        for (k, v) in &self.0 {
            proof.insert_str(&k, &v.gen_proof()?);
        }
        proof.digest()
    }

    pub fn select(&mut self, query: &str) -> Result<(), SdtError> {
        let query_keys = parse_query(query);
        let mut queue: Vec<(String, &mut SdtNode)> = vec![("/".to_owned(), self)];
        while let Some((path, node)) = queue.pop() {
            let mut path_keys: HashMap<String, String> = HashMap::new();
            for (key, val) in node.0.to_owned() {
                let path_key = format!("{}{}/", path, key.to_owned());
                if !query_keys.contains(&path_key) {
                    let matched = query_keys.iter().any(|x| x.starts_with(&path_key));
                    if !matched {
                        node.add_proof(&key, &val.gen_proof()?);
                    } else {
                        path_keys.insert(key, path_key);
                    }
                }
            }

            for (key, val) in &mut node.0 {
                if let SdtNodeKind::Node(inner_node) = val {
                    if let Some(path_key) = path_keys.get(key) {
                        queue.push((path_key.to_owned(), inner_node));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn to_claim(&self, claim: &mut SdtClaim ) -> Result<(), SdtError> {
        for (k, v) in &self.0 {
            match v {
                SdtNodeKind::Value(val) => {
                    
                }
                SdtNodeKind::Node(inner) => {
                    //inner.to_claim()?;
                }
                _ => {}
            }
        }

        Ok(())
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
                    "name": {
                        "salt": "salt",
                        "value": "Adem"
                    },
                    "surname": {
                        "salt": "b",
                        "value": 5,
                        "bb": false
                    }
                }
            }"#;

        let r: SdtNode = serde_json::from_str(result_str)?;
        eprintln!("{:?}", r);
        Ok(())
    }

    #[test]
    fn select_test() -> Result<(), SdtError> {
        let personal = SdtNode::new()
            .add_str_value("name", "Adem")
            .add_str_value("surname", "Çağlın")
            .build();
        let assertions = SdtNode::new().add_str_value("key_1", "0x12").build();
        let keys = SdtNode::new().add_node("assertions", assertions).build();
        let mut root = SdtNode::new()
            .add_node("personal", personal)
            .add_node("keys", keys)
            .build();
        let query = "
        {
          personal{
             name
             surname
          }
        }";

        root.select(query)?;
        eprintln!("{}", serde_json::to_string(&root)?);
        eprintln!("{}", root.gen_proof()?);
        let mut result = SdtValueResult::Branch(HashMap::new());
        root.disclose("", &mut result)?;
        eprintln!("{}", serde_json::to_string(&result)?);
        Ok(())
    }
}
