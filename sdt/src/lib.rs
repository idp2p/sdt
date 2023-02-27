use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Digest;

fn create_random<const N: usize>() -> [u8; N] {
    let mut key_data = [0u8; N];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    key_data
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TrieNode {
    key: String,
    proof: String,
    value: TrieNodeKind,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MutationKind {
    Create { value: Value },
    Update { value: Value },
    Revoke,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TrieNodeKind {
    Masked,
    Claim { salt: String, change: MutationKind },
    Branch { children: Vec<TrieNode> },
}

fn digest(payload: &str) -> String {
    hex::encode(sha2::Sha256::digest(payload.as_bytes()))
}

impl TrieNode {
    pub fn new_branch(key: &str, children: Vec<Self>) -> Self {
        let mut payload = json!({});
        for child in &children {
            payload[child.key.clone()] = serde_json::Value::String(child.proof.clone())
        }

        TrieNode {
            key: key.to_string(),
            proof: digest(&payload.to_string()),
            value: TrieNodeKind::Branch { children },
        }
    }

    pub fn create_claim(key: &str, value: Value) -> Self {
        let salt = hex::encode(create_random::<16>());
        let claim = TrieNodeKind::Claim {
            salt: salt.to_string(),
            change: MutationKind::Create { value },
        };
        let proof = digest(&serde_json::to_string(&claim).unwrap());
        TrieNode {
            key: key.to_string(),
            proof: proof,
            value: claim,
        }
    }

    pub fn update_claim(key: &str, value: Value) -> Self {
        let salt = hex::encode(create_random::<16>());
        let claim = TrieNodeKind::Claim {
            salt: salt.to_string(),
            change: MutationKind::Update { value },
        };
        let proof = digest(&serde_json::to_string(&claim).unwrap());
        TrieNode {
            key: key.to_string(),
            proof: proof,
            value: claim,
        }
    }

    pub fn revoke_claim(key: &str) -> Self {
        let salt = hex::encode(create_random::<16>());
        let claim = TrieNodeKind::Claim {
            salt: salt.to_string(),
            change: MutationKind::Revoke,
        };
        let proof = digest(&serde_json::to_string(&claim).unwrap());
        TrieNode {
            key: key.to_string(),
            proof: proof,
            value: claim,
        }
    }

    pub fn revealByQuery(&self, query: &str) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trie_test() {
        /*
         {
           "subject": "idp2p://xxxx",
           "previous": {
              "subject": "idp2p://xxxx",
              "body": {
               }
           },
           "body": {
            }
         }
        */
        let personal = TrieNode::new_branch(
            "personal",
            vec![TrieNode::create_claim(
                "name",
                Value::String("Adem".to_owned()),
            )],
        );
        let assertion_key1 = TrieNode::create_claim("key1", Value::Number(0u32.into()));
        let assertion_keys = TrieNode::new_branch("assertion_keys", vec![assertion_key1]);
        let root = TrieNode::new_branch("/", vec![personal, assertion_keys]);
        eprintln!("{}", serde_json::to_string(&root).unwrap());
        //eprintln!("{:?}", trie);
    }
}
