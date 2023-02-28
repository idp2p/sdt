pub mod error;
pub mod utils;
pub mod value;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use utils::*;
use value::SdtValue;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct Sdt {
    id: String,
    previous: Option<String>,
    root: SdtNode,
}

impl Sdt {
    pub fn new(id: &str, node: SdtNode) -> Self {
        Self {
            id: id.to_owned(),
            previous: None,
            root: node,
        }
    }

    pub fn new_mutation(id: &str, prev: &str, node: SdtNode) -> Self {
        Self {
            id: id.to_owned(),
            previous: Some(prev.to_owned()),
            root: node,
        }
    }

    pub fn proof(&self) -> String {
        digest(&serde_json::to_string(&self).unwrap())
    }

    pub fn disclose_by_query(&self, query: &str) -> SdtNode {
        let query_keys = parse_query(query);
        let mut result = self.root.clone();
        let mut queue: Vec<(String, &mut SdtNode)> = vec![("".to_owned(), &mut result)];
        while let Some((path, cn)) = queue.pop() {
            let path_key = format!("{}{}/", path, cn.key);
            if !query_keys.contains(&path_key) {
                let matched = query_keys.iter().any(|x| x.starts_with(&path_key));
                if matched {
                    match &mut cn.value {
                        SdtNodeKind::Branch { children } => {
                            for n in children {
                                queue.push((path_key.clone(), n));
                            }
                        }
                        _ => {}
                    }
                }else{
                    cn.mask()
                }
            }
        }
        result
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNode {
    key: String,
    proof: String,
    value: SdtNodeKind,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MutationKind {
    Create { value: SdtValue },
    Update { value: SdtValue },
    Revoke,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SdtNodeKind {
    Masked,
    Claim { salt: String, change: MutationKind },
    Branch { children: Vec<SdtNode> },
}

impl SdtNode {
    pub fn new_branch(key: &str, children: Vec<Self>) -> Self {
        let mut payload = json!({});
        for child in &children {
            payload[child.key.clone()] = serde_json::Value::String(child.proof.clone())
        }

        Self {
            key: key.to_string(),
            proof: digest(&payload.to_string()),
            value: SdtNodeKind::Branch { children },
        }
    }

    pub fn new_claim(key: &str, m: MutationKind) -> Self {
        let salt = hex::encode(create_random::<16>());
        let claim = SdtNodeKind::Claim {
            salt: salt.to_string(),
            change: m,
        };
        let proof = digest(&serde_json::to_string(&claim).unwrap());
        Self {
            key: key.to_string(),
            proof: proof,
            value: claim,
        }
    }

    pub fn mask(&mut self){
        self.value = SdtNodeKind::Masked;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdt_test() {
        let personal = SdtNode::new_branch(
            "personal",
            vec![SdtNode::new_claim(
                "name",
                MutationKind::Create {
                    value: SdtValue::String("Adem".to_owned()),
                },
            )],
        );
        let assertion_key1 = SdtNode::new_claim(
            "key1",
            MutationKind::Create {
                value: SdtValue::Number(0u32.into()),
            },
        );
        let assertion_keys = SdtNode::new_branch("assertion_keys", vec![assertion_key1]);
        let root = SdtNode::new_branch("", vec![personal, assertion_keys]);
        let sdt = Sdt::new("id", root);
        let filter = sdt.disclose_by_query("query");
        eprintln!("{}", serde_json::to_string(&filter).unwrap());

        /*let query_root = SdtQuery::new_with_children(
            "",
            vec![SdtQuery::new_with_children(
                "keys",
                vec![SdtQuery::new_with_children(
                    "agreements",
                    vec![SdtQuery::new("key-1")],
                )],
            )],
        );

        //let mut result: SdtNode = SdtNode::new_branch(key, children)
        let mut queue: Vec<(String, SdtNode)> = vec![("".to_owned(), root)];
        while !queue.is_empty() {
            let (path, mut cn) = queue.pop().unwrap();
            let path_key = format!("{}{}/", path, cn.key);
            cn.value = SdtNodeKind::None;
            match cn.value {
                SdtNodeKind::Claim { salt, change } => println!("{}{}", salt, path_key),
                SdtNodeKind::Branch { children } => {
                    for n in children {
                        queue.push((path_key.clone(), n));
                    }
                }
                _ => {}
            }
        }*/
        //eprintln!("{:?}", trie);
    }
}
