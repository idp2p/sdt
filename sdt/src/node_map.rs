use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;

use crate::{
    error::SdtError,
    utils::{create_random, digest, parse_query},
    value::SdtValue,
};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MutationKind {
    Create { value: SdtValue },
    Update { value: SdtValue },
    Revoke,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtClaim {
    salt: String,
    change: MutationKind,
}

#[skip_serializing_none]
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Default)]
pub struct SdtNode {
    pub proof: Option<String>,
    pub value: Option<SdtClaim>,
    pub branch: Option<BTreeMap<String, SdtNode>>,
}

impl SdtNode {
    pub fn new() -> Self {
        Self {
            proof: None,
            value: None,
            branch: None,
        }
    }

    fn create(&mut self, key: &str, value: Option<SdtClaim>) -> &mut SdtNode {
        let child = Self {
            proof: None,
            value: value,
            branch: None,
        };
        if self.branch.is_none() {
            self.branch = Some(BTreeMap::new());
        }
        self.branch.as_mut().unwrap().insert(key.to_owned(), child);
        self.branch.as_mut().unwrap().get_mut(key).unwrap()
    }

    pub fn create_branch(&mut self, key: &str) -> &mut SdtNode {
        self.create(key, None)
    }

    pub fn push_claim(&mut self, key: &str, m: MutationKind) -> &mut SdtNode {
        let salt = hex::encode(create_random::<16>());
        let claim = SdtClaim {
            salt: salt.to_string(),
            change: m,
        };
        self.create(key, Some(claim));
        self
    }

    pub fn gen_proof(&mut self) -> Result<String, SdtError> {
        let mut payload = json!({});
        if let Some(value) = &mut self.value {
            payload["value"] = serde_json::to_value(&value)?;
        }
        if let Some(branch) = &mut self.branch {
            let mut branch_pay = json!({});
            for (k, v) in branch {
                branch_pay[k] = serde_json::Value::String(v.gen_proof()?)
            }
            payload["branch"] = branch_pay;
        }

        let digest = digest(&serde_json::to_string(&payload)?);
        self.proof = Some(digest.clone());
        Ok(digest)
    }
}

pub fn disclose(result: &mut SdtNode, query: &str) -> Result<(), SdtError> {
    let query_keys = parse_query(query);
    let mut queue: Vec<(String, &mut SdtNode)> = vec![("".to_owned(), result)];
    while let Some((path, cn)) = queue.pop() {
        if let Some(bm) = cn.branch.as_mut() {
           for (k, v) in bm{
              let path_key = format!("{}{}/", path, k);
              if !query_keys.contains(&path_key) {
                let matched = query_keys.iter().any(|x| x.starts_with(&path_key));
                if matched {
                    
                } else {
                    
                }
            }
           }
           cn.branch = None;
        }
        
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::value::SdtValue;

    use super::*;
    #[test]
    fn sdt_test() {
        let a_value = MutationKind::Create {
            value: SdtValue::String("Adem".to_owned()),
        };
        let mut root = SdtNode::new();
        let personal = root.create_branch("personal");

        personal
            .push_claim("surname", a_value.clone())
            .push_claim("name", a_value.clone());
        let addresses = personal.create_branch("addresses");
        addresses.push_claim("work", a_value.clone());
        let keys = root.create_branch("keys");
        let assertions = keys.create_branch("assertions");
        assertions.push_claim("key-1", a_value);
        eprintln!("{}", root.gen_proof().unwrap());
        eprintln!("{}", root.gen_proof().unwrap());
        eprintln!("{}", serde_json::to_string(&root).unwrap());
    }
}
