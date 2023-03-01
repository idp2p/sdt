pub mod error;
pub mod utils;
pub mod value;
pub mod node;
pub mod disclose;
use error::SdtError;
use node::SdtNode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utils::*;
use value::SdtValue;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MutationKind {
    Create { value: SdtValue },
    Update { value: SdtValue },
    Revoke,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SdtNodeKind<T> {
    Claim { salt: String, change: MutationKind },
    Branch { children: Vec<T> },
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtResult {
    key: String,
    proof: String,
    value: Option<SdtNodeKind<Self>>,
}

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

    pub fn gen_proof(&self) -> Result<String, SdtError> {
        Ok(digest(&serde_json::to_string(&self)?))
    }

    pub fn disclose_by_query(&self, query: &str) -> Result<SdtResult, SdtError> {
        todo!()
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdt_test() {
        /*let personal = SdtNode::new_branch(
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
        eprintln!("{}", root.proof().unwrap());*/
        //let sdt = Sdt::new("id", root);
        //let filter = sdt.disclose_by_query("query");
        //eprintln!("{}", serde_json::to_string(&filter).unwrap());

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
