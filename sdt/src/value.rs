use crate::error::SdtError;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

#[derive(PartialEq, Debug, Clone)]
pub enum SdtValue {
    Bool(bool),
    Number(Number),
    String(String),
}

impl From<SdtValue> for Value {
    fn from(value: SdtValue) -> Self {
        match value {
            SdtValue::Bool(b) => Value::Bool(b),
            SdtValue::Number(n) => Value::Number(n),
            SdtValue::String(s) => Value::String(s),
        }
    }
}

impl TryFrom<Value> for SdtValue {
    type Error = SdtError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(SdtValue::Bool(b)),
            Value::Number(n) => Ok(SdtValue::Number(n)),
            Value::String(s) => Ok(SdtValue::String(s)),
            _ => Err(SdtError::Other),
        }
    }
}

impl Serialize for SdtValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Value::from(self.to_owned()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SdtValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;
        v.try_into().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
    struct Custom {
       name: SdtValue
    }
    #[test]
    fn value_test() {
        let c = Custom{
            name: SdtValue::Number(Number::from(5))
        };
        let s = serde_json::to_string(&c).unwrap();
        let rec: Custom = serde_json::from_str(&s).unwrap();
        assert_eq!(c, rec);
    }
}
