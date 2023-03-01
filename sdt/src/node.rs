use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{SdtNodeKind, error::SdtError, MutationKind, utils::{create_random, digest}, SdtResult};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNode {
    key: String,
    value: SdtNodeKind<Self>,
}

impl SdtNode {
    pub fn new() -> Self {
        Self {
            key: "".to_owned(),
            value: SdtNodeKind::Branch { children: vec![] },
        }
    }

    pub fn from_json(node: &str) -> Result<Self, SdtError> {
        /*let v: Value = serde_json::from_str(events)?;
        let mut node = SdtNode::new_branch("", vec![]);
        let mut queue: Vec<(Value, &mut SdtNode)> = vec![(v, &mut node)];
        while let Some((cv, cn)) = queue.pop() {
            if let Some(map) = cv.as_object() {
                if map.contains_key("kind") {
                } else {
                }
            }
        }*/
        todo!()
    }

    pub fn create_branch(&mut self, key: &str) -> &mut SdtNode {
        let child = Self {
            key: key.to_string(),
            value: SdtNodeKind::Branch { children: vec![] },
        };
        if let SdtNodeKind::Branch { children } = &mut self.value {
            children.push(child);
            children.last_mut().unwrap()
        } else {
            panic!("Can't add branch");
        }
    }

    pub fn push_claim(&mut self, key: &str, m: MutationKind) -> &mut SdtNode {
        let salt = hex::encode(create_random::<16>());
        let claim = Self {
            key: key.to_string(),
            value: SdtNodeKind::Claim {
                salt: salt.to_string(),
                change: m,
            },
        };
        if let SdtNodeKind::Branch { children } = &mut self.value {
            children.push(claim);
        } else {
            panic!("Can't add claim");
        }
        self
    }

    pub fn gen_proof(&self) -> Result<String, SdtError> {
        let mut payload = json!({"key": self.key});
        match &self.value {
            SdtNodeKind::Claim { salt: _, change: _ } => {
                payload["kind"] = serde_json::Value::String("Claim".to_owned());
                payload["value"] = serde_json::to_value(&self.value)?;
            }
            SdtNodeKind::Branch { children } => {
                payload["kind"] = serde_json::Value::String("Branch".to_owned());
                let mut value = json!({});
                let mut sorted = children.clone();
                sorted.sort_by_key(|x| x.key.clone());
                for child in sorted {
                    value[child.key.clone()] = serde_json::Value::String(child.gen_proof()?)
                }
                payload["value"] = value;
            }
        }
        let digest = digest(&serde_json::to_string(&payload)?);
        Ok(digest)
    }
}

impl TryFrom<&SdtNode> for SdtResult{
    type Error = SdtError;

    fn try_from(value: &SdtNode) -> Result<Self, Self::Error> {
        todo!()
    }
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
        let addresses = personal.create_branch("addresses");
        addresses.push_claim("work", a_value.clone());
        personal
            .push_claim("name", a_value.clone())
            .push_claim("surname", a_value.clone());

        let keys = root.create_branch("keys");
        let assertions = keys.create_branch("assertions");
        assertions.push_claim("key-1", a_value);
        eprintln!("{}", root.gen_proof().unwrap());
        eprintln!("{}", root.gen_proof().unwrap());
        eprintln!("{}", serde_json::to_string(&root).unwrap());
    }
}
