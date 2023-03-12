use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    error::SdtError,
    proof::SdtProof,
    utils::parse_query,
    value::{SdtValue, SdtValueKind},
};
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtMap(HashMap<String, SdtMapValue>);

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtMapValue {
    Proof(String),
    Value(SdtValue),
    Map(SdtMap),
}

impl SdtMapValue {
    pub fn gen_proof(&self) -> Result<String, SdtError> {
        match &self {
            SdtMapValue::Proof(p) => Ok(p.to_owned()),
            SdtMapValue::Value(value) => value.gen_proof(),
            SdtMapValue::Map(children) => children.gen_proof(),
        }
    }
}

impl SdtMap {
    pub fn new() -> Self {
        let map: HashMap<String, SdtMapValue> = HashMap::new();
        Self(map)
    }

    pub fn add_node(&mut self, key: &str, map: Self) -> &mut Self {
        self.0.insert(key.to_owned(), SdtMapValue::Map(map));
        self
    }

    pub fn add_str_value(&mut self, key: &str, val: &str) -> &mut Self {
        self.0.insert(
            key.to_owned(),
            SdtMapValue::Value(SdtValue::new(SdtValueKind::String(val.to_owned()))),
        );
        self
    }

    pub fn add_proof(&mut self, key: &str, proof: &str) -> &mut Self {
        self.0.insert(key.to_owned(), SdtMapValue::Proof(proof.to_owned()));
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
        let mut queue: Vec<(String, &mut SdtMap)> = vec![("/".to_owned(), self)];
        while let Some((path, map)) = queue.pop() {
            let mut path_keys: HashMap<String, String> = HashMap::new();
            for (key, val) in map.0.to_owned() {
                let path_key = format!("{}{}/", path, key.to_owned());
                if !query_keys.contains(&path_key) {
                    let matched = query_keys.iter().any(|x| x.starts_with(&path_key));
                    if !matched {
                        map.add_proof(&key, &val.gen_proof()?);
                    } else {
                        path_keys.insert(key, path_key);
                    }
                }
            }

            for (key, val) in &mut map.0 {
                if let SdtMapValue::Map(m) = val {
                    if let Some(path_key) = path_keys.get(key) {
                        queue.push((path_key.to_owned(), m));
                    }
                }
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

        let r: SdtMap = serde_json::from_str(result_str)?;
        eprintln!("{:?}", r);
        Ok(())
    }

    #[test]
    fn select_test() -> Result<(), SdtError> {
        let personal = SdtMap::new()
            .add_str_value("name", "Adem")
            .add_str_value("surname", "Çağlın")
            .build();
        let assertions = SdtMap::new().add_str_value("key_1", "0x12").build();
        let keys = SdtMap::new().add_node("assertions", assertions).build();
        let mut root = SdtMap::new()
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
        eprintln!("{}", root.gen_proof()?);
        root.select(query)?;
        eprintln!("{}", serde_json::to_string(&root)?);
        eprintln!("{}", root.gen_proof()?);
        Ok(())
    }
}
