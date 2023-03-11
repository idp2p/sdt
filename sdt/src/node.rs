use crate::{
    dto::SdtDiscloseResult, error::SdtError, proof::SdtProof, utils::parse_query, value::*,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtBranch {
    pub branch: HashMap<String, SdtNodeKind>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtPayloadKind {
    Leaf(SdtValue),
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
    pub payload: SdtPayloadKind,
}

impl SdtBranch {
    pub fn new() -> Self {
        let map: HashMap<String, SdtNodeKind> = HashMap::new();
        let branch = Self { branch: map };
        branch
    }

    pub fn add_node(&mut self, key: &str, node: SdtNode) -> &mut Self {
        self.branch.insert(key.to_owned(), SdtNodeKind::Node(node));
        self
    }

    pub fn add_str_value(&mut self, key: &str, val: &str) -> Result<&mut Self, SdtError> {
        self.branch
            .insert(key.to_owned(), SdtNodeKind::new_str_value(val)?);
        Ok(self)
    }

    pub fn add_bool_value(&mut self, key: &str, val: bool) -> Result<&mut Self, SdtError> {
        self.branch.insert(
            key.to_owned(),
            SdtNodeKind::new_value(SdtValueKind::Bool(val))?,
        );
        Ok(self)
    }

    pub fn add_i64_value(&mut self, key: &str, val: i64) -> Result<&mut Self, SdtError> {
        self.branch.insert(
            key.to_owned(),
            SdtNodeKind::new_value(SdtValueKind::new_i64(val))?,
        );
        Ok(self)
    }

    pub fn add_value(&mut self, key: &str, val: SdtValueKind) -> Result<&mut Self, SdtError> {
        self.branch
            .insert(key.to_owned(), SdtNodeKind::new_value(val)?);
        Ok(self)
    }

    pub fn build(&mut self) -> Result<SdtNode, SdtError> {
        let mut proof_map = SdtProof::new();
        for (k, v) in &self.branch {
            let key_proof = match v {
                SdtNodeKind::Proof(s) => s.to_owned(),
                SdtNodeKind::Node(n) => n.proof.to_owned(),
            };
            proof_map.insert_str(k, &key_proof);
        }
        let proof = proof_map.digest()?;
        Ok(SdtNode {
            proof,
            payload: SdtPayloadKind::Branch(self.to_owned()),
        })
    }
}

impl SdtNodeKind {
    pub fn new_value(v: SdtValueKind) -> Result<Self, SdtError> {
        let val = SdtValue::new(v);
        Ok(SdtNodeKind::Node(SdtNode {
            proof: val.gen_proof()?,
            payload: SdtPayloadKind::Leaf(val),
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
    pub fn select(&mut self, query: &str) -> Result<(), SdtError> {
        let query_keys = parse_query(query);
        let mut queue: Vec<(String, &mut SdtNode)> = vec![("/".to_owned(), self)];
        while let Some((path, cn)) = queue.pop() {
            if let SdtPayloadKind::Branch(payload) = &mut cn.payload {
                let hm = &mut payload.branch;
                let mut path_keys: HashMap<String, String> = HashMap::new();
                for (key, nk) in hm.to_owned() {
                    let path_key = format!("{}{}/", path, key.to_owned());
                    if let SdtNodeKind::Node(n) = nk {
                        if !query_keys.contains(&path_key) {
                            let matched = query_keys.iter().any(|x| x.starts_with(&path_key));
                            if !matched {
                                hm.insert(key.to_owned(), SdtNodeKind::Proof(n.proof.clone()));
                            } else {
                                path_keys.insert(key, path_key);
                            }
                        }
                    }
                }

                for (key, nk) in hm {
                    if let SdtNodeKind::Node(n) = nk {
                        if let Some(path_key) = path_keys.get(key) {
                            queue.push((path_key.to_owned(), n));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn disclose(&self, key: &str, result: &mut SdtDiscloseResult) -> Result<(), SdtError> {
        if let SdtDiscloseResult::Branch(map) = result {
            match &self.payload {
                SdtPayloadKind::Leaf(leaf) => {
                    let entry = map
                        .entry(key.to_owned())
                        .or_insert(SdtDiscloseResult::Values(vec![]));
                    if let SdtDiscloseResult::Values(values) = entry {
                        values.push(leaf.value.to_owned());
                    }
                }
                SdtPayloadKind::Branch(br) => {
                    let new_branch = map
                        .entry(key.to_owned())
                        .or_insert(SdtDiscloseResult::Branch(HashMap::new()));

                    for (k, v) in &br.branch {
                        match v {
                            SdtNodeKind::Node(node) => {
                                node.disclose(k, new_branch)?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn verify(&self) -> Result<String, SdtError> {
        let proof = match &self.payload {
            SdtPayloadKind::Leaf(leaf) => leaf.gen_proof(),
            SdtPayloadKind::Branch(br) => {
                let mut proof_map = SdtProof::new();
                for (k, v) in &br.branch {
                    let key_proof = match v {
                        SdtNodeKind::Proof(s) => s.to_owned(),
                        SdtNodeKind::Node(n) => n.verify()?,
                    };
                    proof_map.insert_str(k, &key_proof);
                }
                proof_map.digest()
            }
        }?;
        if self.proof != proof {
            return Err(SdtError::VerificationError {
                expected: self.proof.to_owned(),
                actual: proof,
            });
        }
        Ok(proof)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdt_test() -> Result<(), SdtError> {
        let personal = SdtBranch::new()
            .add_str_value("name", "Adem")?
            .add_str_value("surname", "Çağlın")?
            .add_i64_value("age", 40)?
            .build()?;
        let assertions = SdtBranch::new().add_str_value("key_1", "0x12")?.build()?;
        let keys = SdtBranch::new()
            .add_node("assertions", assertions)
            .build()?;
        let mut root = SdtBranch::new()
            .add_node("personal", personal)
            .add_node("keys", keys)
            .build()?;
        let query = "
        {
            personal{
                name
            }
        }
        ";
        root.select(query)?;
        eprintln!("{}", serde_json::to_string(&root)?);
        eprintln!("{}", root.verify()?);
        //let mut result = SdtResult::Branch(HashMap::new());
        //root.disclose("", &mut result)?;
        //eprintln!("{}", serde_json::to_string(&result)?);
        match root.payload {
            SdtPayloadKind::Branch(root_branch) => {
                if let SdtNodeKind::Node(_) = root_branch.branch.get("keys").unwrap() {
                    panic!("Keys should be proof")
                }
                match root_branch.branch.get("personal").unwrap() {
                    SdtNodeKind::Proof(_) => panic!("Personal should be node"),
                    SdtNodeKind::Node(personal_node) => match &personal_node.payload {
                        SdtPayloadKind::Leaf(_) => panic!("Personal should be branch"),
                        SdtPayloadKind::Branch(personal_br) => {
                            if let SdtNodeKind::Node(_) = personal_br.branch.get("surname").unwrap()
                            {
                                panic!("Surname should be proof")
                            }
                            if let SdtNodeKind::Node(name_node) =
                                personal_br.branch.get("name").unwrap()
                            {
                                if let SdtPayloadKind::Branch(_) = name_node.payload {
                                    panic!("Name should be leaf")
                                }
                            } else {
                                panic!("Name should exist")
                            }
                        }
                    },
                }
            }
            _ => panic!("Root should be branch"),
        }
        /*let root_val = serde_json::Value::from_str(&serde_json::to_string(&root)?)?;
        let val = root_val.get("branch")
           .and_then(|v| v.get("personal"))
           .and_then(|v| v.get("branch"))
           .and_then(|v| v.get("name"));
        eprintln!("{}", serde_json::to_string(&root)?);*/
        Ok(())
    }
}
