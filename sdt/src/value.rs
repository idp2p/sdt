use serde::{Deserialize, Serialize};

use crate::{error::SdtError, proof::SdtProof, utils::create_random};
use serde_json::Number;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtValueKind {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtValue {
    pub salt: String,
    pub value: SdtValueKind,
}

impl SdtValue {
    pub fn new(value: SdtValueKind) -> Self {
        let raw = hex::encode(create_random::<16>()).to_owned();
        let salt = format!("0x{raw}");
        Self { salt, value }
    }

    pub fn gen_proof(&self) -> Result<String, SdtError> {
        SdtProof::new()
            .insert_str("salt", &self.salt)
            .insert("value", self.value.clone())
            .digest()
    }
}

impl SdtValueKind {
    pub fn new_i64(number: i64) -> Self {
        SdtValueKind::Number(Number::from(number))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_test() -> Result<(), SdtError> {
        Ok(())
    }
}
