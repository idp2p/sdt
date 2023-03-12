pub mod dto;
pub mod error;
pub mod node;
pub mod proof;
pub mod service;
pub mod utils;
pub mod value;
pub mod node2;
use proof::SdtProof;
use std::collections::HashMap;

use dto::{SdtClaim, SdtDiscloseResult};
use error::SdtError;
use node::SdtNode;
use serde::{Deserialize, Serialize};
const HASH_ALG: u64 = 0x12;
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtPayload {
    pub proof: String,
    pub node: SdtNode,
    pub next: Option<Box<SdtPayload>>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Sdt {
    // JSON and HEX are default
    pub hash_alg: u64, // SHA256
    pub subject: String,
    pub payload: SdtPayload,
}

impl SdtPayload {
    pub fn find_current(&mut self) -> &mut Self {
        if self.next.is_none() {
            return self;
        }
        self.next.as_mut().unwrap().find_current()
    }
}

impl Sdt {
    pub fn new(sub: &str, claim: SdtClaim) -> Result<Self, SdtError> {
        let node = claim.to_node()?;
        let proof = inception_proof(sub, &node.proof)?;
        Ok(Sdt {
            hash_alg: HASH_ALG,
            subject: sub.to_owned(),
            payload: SdtPayload {
                proof,
                node,
                next: None,
            },
        })
    }

    pub fn mutate(&mut self, claim: SdtClaim) -> Result<&mut Self, SdtError> {
        let current = self.payload.find_current();
        let node = claim.to_node()?;
        let proof = mutation_proof(&current.proof, &node.proof)?;
        current.next = Some(Box::new(SdtPayload {
            proof,
            node,
            next: None,
        }));
        Ok(self)
    }

    pub fn build(&mut self) -> Result<Self, SdtError> {
        Ok(self.to_owned())
    }

    pub fn select(&mut self, query: &str) -> Result<(), SdtError> {
        self.payload.select(query)?;
        Ok(())
    }

    pub fn verify(&self, proof: &str) -> Result<SdtDiscloseResult, SdtError> {
        let mut result = SdtDiscloseResult::Branch(HashMap::new());
        let node_proof = self.payload.node.verify()?;
        let inception_proof = inception_proof(&self.subject, &node_proof)?;
        self.payload.node.disclose("", &mut result)?;
        if let Some(next) = &self.payload.next {
            let verified_proof = next.verify(&inception_proof, &mut result)?;
            if verified_proof != proof {
                return Err(SdtError::VerificationError {
                    expected: proof.to_owned(),
                    actual: verified_proof,
                });
            }
        }
        Ok(result)
    }
}
fn inception_proof(sub: &str, claim_proof: &str) -> Result<String, SdtError> {
    SdtProof::new()
        .insert_i64("hash_alg", HASH_ALG as i64)
        .insert_str("subject", sub)
        .insert_str("root", claim_proof)
        .digest()
}

fn mutation_proof(previous: &str, claim_proof: &str) -> Result<String, SdtError> {
    SdtProof::new()
        .insert_str("root", claim_proof)
        .insert_str("previous", previous)
        .digest()
}

impl SdtPayload {
    pub fn select(&mut self, query: &str) -> Result<&mut Self, SdtError> {
        self.node.select(query)?;
        if self.next.is_none() {
            return Ok(self);
        }
        self.next.as_mut().unwrap().select(query)
    }

    pub fn verify(&self, prev: &str, res: &mut SdtDiscloseResult) -> Result<String, SdtError> {
        let node_proof = self.node.verify()?;
        let pay_proof = mutation_proof(prev, &node_proof)?;
        self.node.disclose("", res)?;
        if let Some(next) = &self.next {
            return next.verify(&self.proof, res);
        } else {
            return Ok(pay_proof);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdt_test() -> Result<(), SdtError> {
        let new_claim_str = r#"{
            "personal": {
               "name": "Adem",
               "surname": "Çağlın",
               "age": 5
            },
            "keys": {
               "assertions": {
                  "key-1": "0x12...."
               }
            }
        }"#;
        let mutation_str = r#"{
            "personal": {
               "name": null,
               "surname": null
            }
        }"#;
        let mutation2_str = r#"{
            "keys": {
                "assertions": {
                   "key-1": "0x1234...."
                }
             }
        }"#;
        let query = "
        {
            personal {
                name
            }
        }
        ";
        let new_claim: SdtClaim = serde_json::from_str(new_claim_str)?;
        let mutation: SdtClaim = serde_json::from_str(mutation_str)?;
        let mutation2: SdtClaim = serde_json::from_str(mutation2_str)?;
        let mut sdt = Sdt::new("did:p2p:123456", new_claim)?
            .mutate(mutation)?
            .mutate(mutation2)?
            .build()?;
        eprintln!("{}", serde_json::to_string(&sdt)?);
        let result = sdt.verify(
            &sdt.payload
                .next
                .clone()
                .unwrap()
                .next
                .clone()
                .unwrap()
                .proof,
        )?;
        sdt.select(query)?;
        eprintln!("{}", serde_json::to_string(&result)?);
        //eprintln!("{}", serde_json::to_string(&sdt)?);
        Ok(())
    }
}
