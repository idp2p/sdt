use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    error::SdtError,
    utils::{digest, parse_query},
    value::*,
};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtBranch {
    pub branch: HashMap<String, SdtNodeKind>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtBodyKind {
    Value(SdtValue),
    Branch(SdtBranch),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtNodeKind {
    Proof(String),
    Node(SdtNode),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNode {
    pub proof: String,
    #[serde(flatten)]
    pub body: SdtBodyKind,
}

impl SdtBranch {
    pub fn new() -> Self {
        let map: HashMap<String, SdtNodeKind> = HashMap::new();
        let branch = Self { branch: map };
        branch
    }

    pub fn add(&mut self, key: &str, node: SdtNodeKind) -> &mut Self {
        self.branch.insert(key.to_owned(), node);
        self
    }

    pub fn build(&mut self) -> Result<SdtNode, SdtError> {
        let mut body: BTreeMap<String, SdtValueKind> = BTreeMap::new();
        for (k, v) in &self.branch {
            let key_proof = match v {
                SdtNodeKind::Proof(s) => s.to_owned(),
                SdtNodeKind::Node(n) => n.proof.to_owned(),
            };
            body.insert(k.to_owned(), SdtValueKind::String(key_proof));
        }
        let payload = SdtPayload(body);
        let proof = digest(&payload)?;
        Ok(SdtNode {
            proof,
            body: SdtBodyKind::Branch(self.to_owned()),
        })
    }
}

impl SdtNodeKind {
    pub fn new_value(v: SdtValueKind) -> Result<Self, SdtError> {
        let val = SdtValue::new(v);
        Ok(SdtNodeKind::Node(SdtNode {
            proof: val.gen_proof()?,
            body: SdtBodyKind::Value(val),
        }))
    }

    pub fn new_str_value(s: &str) -> Result<Self, SdtError> {
        Self::new_value(SdtValueKind::String(s.to_owned()))
    }

    pub fn new_proof(p: &str) -> Self {
        SdtNodeKind::Proof(p.to_owned())
    }
}

impl SdtNode {
    pub fn from_json(s: &str) -> Result<Self, SdtError>{
        let values: SdtValues = serde_json::from_str(s)?;
        values.parse_json()
    }

    pub fn disclose(&mut self, query: &str) -> Result<(), SdtError> {
        let query_keys = parse_query(query);
        let mut queue: Vec<(String, &mut SdtNode)> = vec![("/".to_owned(), self)];
        while let Some((path, cn)) = queue.pop() {
            if let SdtBodyKind::Branch(body) = &mut cn.body {
                let hm = &mut body.branch;
                let mut keys: HashMap<String, String> = HashMap::new();
                for (key, nk) in hm.to_owned() {
                    let path_key = format!("{}{}/", path, key.to_owned());
                    if let SdtNodeKind::Node(n) = nk {
                        if !query_keys.contains(&path_key) {
                            let matched = query_keys.iter().any(|x| x.starts_with(&path_key));
                            if !matched {
                                hm.insert(key.to_owned(), SdtNodeKind::Proof(n.proof.clone()));
                            } else {
                                keys.insert(key, path_key);
                            }
                        }
                    }
                }
    
                for (key, nk) in hm {
                    if let SdtNodeKind::Node(n) = nk {
                        if let Some(path_key) = keys.get(key) {
                            queue.push((path_key.to_owned(), n));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
}


#[cfg(test)]
mod tests {
    use serde_json::{Number, json};

    use super::*;
    #[test]
    fn sdt_test() -> Result<(), SdtError> {
        let personal = SdtBranch::new()
            .add("name", SdtNodeKind::new_str_value("Adem")?)
            .add("surname", SdtNodeKind::new_str_value("Çağlın")?)
            .add(
                "age",
                SdtNodeKind::new_value(SdtValueKind::Number(Number::from(40)))?,
            )
            .build()?;
        let assertions = SdtBranch::new()
            .add("key_1", SdtNodeKind::new_str_value("key1....")?)
            .build()?;
        let keys = SdtBranch::new()
            .add("assertions", SdtNodeKind::Node(assertions))
            .build()?;
        let mut root = SdtBranch::new()
            .add("personal", SdtNodeKind::Node(personal))
            .add("keys", SdtNodeKind::Node(keys))
            .build()?;
        let query = "
        {
            personal{
                name
            }
        }
        ";
        root.disclose(query)?;
        eprintln!("{}", serde_json::to_string(&root)?);
        Ok(())
    }

    #[test]
    fn from_json_test() -> Result<(), SdtError> {
        let s = json!({
            "personal": {
               "name": "Adem",
               "age": 5
            },
            "keys": {
               "assertions": {
                  "key-1": "z...."
               }
            }
        });
        let node = SdtNode::from_json(&s.to_string())?;
        eprintln!("{}", serde_json::to_string(&node)?);
        Ok(())
    }
}
