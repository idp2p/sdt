use crate::{dto::SdtClaim, error::SdtError, Sdt};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "payload")]
pub enum SdtInput {
    Inception { subject: String, claim: SdtClaim },
    Mutation { sdt: Sdt, claim: SdtClaim },
    Selection { sdt: Sdt, query: String },
    Proof(Sdt),
    Verification { sdt: Sdt, proof: String },
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SdtResult {
    Inception(Sdt),
    Mutation(Sdt),
    Selection(Sdt),
    Proof(String),
    Verification(bool),
    Error { error_kind: String, message: String },
}

impl SdtInput {
    pub fn from_str(input: &str) -> Result<Self, SdtError> {
        let s: Self = serde_json::from_str(input)?;
        Ok(s)
    }

    pub fn execute(self) -> Result<SdtResult, SdtError> {
        let result = match self {
            SdtInput::Inception { subject, claim } => {
                SdtResult::Inception(Sdt::new(&subject, claim))
            }
            SdtInput::Mutation { sdt, claim } => {
                let mut sdt_clone = sdt.clone();
                SdtResult::Mutation(sdt_clone.mutate(claim).build())
            }
            SdtInput::Selection { sdt, query } => {
                let mut sdt_clone = sdt.clone();
                sdt_clone.select(&query)?;
                SdtResult::Selection(sdt_clone.build())
            }
            SdtInput::Proof(sdt) => SdtResult::Proof(sdt.gen_proof()?),
            SdtInput::Verification { sdt, proof } => SdtResult::Verification(sdt.verify(&proof)?),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let r = "bb".as_bytes() > "aa".as_bytes();
        eprintln!("{r}");
    }
}
