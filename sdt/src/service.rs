use crate::{dto::{SdtClaim, SdtDiscloseResult}, error::SdtError, Sdt};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "payload")]
pub enum SdtInput {
    Inception { subject: String, claim: SdtClaim },
    Mutation { sdt: Sdt, claim: SdtClaim },
    Select { sdt: Sdt, query: String },
    Verification(Sdt),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SdtResult {
    Inception(Sdt),
    Mutation(Sdt),
    Select(Sdt),
    Verification(SdtDiscloseResult),
    Error{
        error_kind: String,
        message: String
    }
}

impl SdtInput {
    pub fn from_str(input: &str) -> Result<Self, SdtError>{
        let s: Self = serde_json::from_str(input)?;
        Ok(s)
    }

    pub fn execute(&self) -> Result<SdtResult, SdtError> {
        match &self {
            SdtInput::Inception { subject, claim } => todo!(),
            SdtInput::Mutation { sdt, claim } => todo!(),
            SdtInput::Select { sdt, query } => todo!(),
            SdtInput::Verification(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {}
}
