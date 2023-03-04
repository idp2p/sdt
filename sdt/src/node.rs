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
    Bool(bool),
    Number(Number),
    String(String),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum EventKind {
    #[serde(rename = "C")]
    Create { value: SdtValue },
    #[serde(rename = "U")]
    Update { value: SdtValue },
    #[serde(rename = "R")]
    Revoke,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNodeValue {
    #[serde(flatten)]
    event: EventKind,
    salt: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtNodeKind {
    Value(SdtNodeValue),
    Branch(Vec<SdtNode>),
}

#[skip_serializing_none]
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNode {
    pub key: String,
    pub proof: Option<String>,
    pub inner: Option<SdtNodeKind>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct  SdtPayload (BTreeMap<String, String>);

impl SdtNode {
    pub fn new() -> Self {
        Self {
            key: "".to_owned(),
            proof: None,
            inner: Some(SdtNodeKind::Branch(vec![])),
        }
    }

    pub fn from_json(claims: &str) -> Result<Self, SdtError> {
        let v: Value = serde_json::from_str(claims)?;
         parse_json("", v)
    }

    pub fn create_branch(&mut self, key: &str) -> &mut Self {
        let node = Self {
            key: key.to_string(),
            proof: None,
            inner: Some(SdtNodeKind::Branch(vec![])),
        };
        if let SdtNodeKind::Branch(children) = self.inner.as_mut().unwrap() {
            children.push(node);
            children.last_mut().unwrap()
        } else {
            panic!("Can't add branch");
        }
    }

    pub fn create_value(&mut self, key: &str, event: EventKind) -> &mut Self {
        let salt = hex::encode(create_random::<16>()).to_owned();
        let value = SdtNodeValue {
            event: event,
            salt: salt,
        };
        let node = Self {
            key: key.to_string(),
            proof: None,
            inner: Some(SdtNodeKind::Value(value)),
        };
        if let SdtNodeKind::Branch(children) = self.inner.as_mut().unwrap() {
            children.push(node);
        } else {
            panic!("Can't add claim");
        }
        self
    }

    pub fn gen_proof(&mut self) -> Result<String, SdtError> {
        let digest = match self.inner.as_mut().unwrap() {
            SdtNodeKind::Branch(children) => {
                let mut body: BTreeMap<String, String> = BTreeMap::new();
                for child in children {
                    body.insert(child.key.to_owned(), child.gen_proof()?);
                }
                let payload = SdtPayload(body);
                digest(&payload)?
            }
            val => digest(&val)?,
        };
        self.proof = Some(digest.clone());
        Ok(digest)
    }
}

fn parse_json(key: &str, val: Value) -> Result<SdtNode, SdtError>{
    let mut node = SdtNode::new();
    node.key = key.to_owned();
    match val {
        /*Value::Bool(b => {
            node.inner = Some(SdtNodeKind::Value(SdtNodeValue { event: EventKind::Create {
                value: SdtValue::Bool(b),
            }, salt: "".to_owned() }));
            
        }
        Value::Number(n) => {
            cn.create_value(
                &key,
                EventKind::Create {
                    value: SdtValue::Number(n),
                },
            );
        }*/
        Value::String(s) => {
            let event = EventKind::Create {
                value: SdtValue::String(s),
            };
            let x = SdtNodeKind::Value(SdtNodeValue { event, salt: "aaa".to_owned() });
            node.inner = Some(x);
        }
        Value::Object(kv) => {
            let mut list: Vec<SdtNode> = vec![]; 
            for (k, v) in kv { 
                let node = parse_json(&k, v)?;
                list.push(node);
            }
            node.inner = Some(SdtNodeKind::Branch(list));
        }
        _ => return Err(SdtError::Other),
    }
    return Ok(node)

} 
#[cfg(test)]
mod tests {

    use serde_json::json;

    use super::*;
    #[test]
    fn sdt_test() {
        let a_value = EventKind::Create {
            value: SdtValue::String("Adem".to_owned()),
        };
        let mut root = SdtNode::new();
        let personal = root.create_branch("personal");

        personal
            .create_value("surname", a_value.clone())
            .create_value("name", a_value.clone());
        let addresses = personal.create_branch("addresses");
        addresses.create_value("work", a_value.clone());
        let keys = root.create_branch("keys");
        let assertions = keys.create_branch("assertions");
        assertions.create_value("key-1", a_value);
        eprintln!("{}", root.gen_proof().unwrap());
        eprintln!("{}", root.gen_proof().unwrap());
        eprintln!("{}", serde_json::to_string(&root).unwrap());
        eprintln!("--------------------------");

        let _claims = json!({
            "personal": {
              "name": "Adem",
              "surname": "Çağlın",
              "gender": "Male",
              "birthday": "1.1.1984"
            },
            "phones": {
              "dial": "+90dial",
              "cell": "+90cell"
            },
            "addresses": {
              "home": {
                "zipcode": "2020",
                "city": "homecity"
              },
              "work": {
                "zipcode": "2030",
                "city": "workcity"
              }
            }
          });
          eprintln!("{}", serde_json::to_string_pretty(&parse_json("", _claims).unwrap()).unwrap());
    }
}
