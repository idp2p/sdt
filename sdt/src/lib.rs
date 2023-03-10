pub mod error;
pub mod node;
pub mod utils;
pub mod value;
use std::collections::HashMap;

use error::SdtError;
use node::SdtNode;
use serde::{Deserialize, Serialize};
use utils::digest;
use value::{SdtClaim, SdtProofPayload, SdtResult};
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
        Ok(Sdt {
            hash_alg: HASH_ALG,
            subject: sub.to_owned(),
            payload: SdtPayload {
                proof: inception_proof(sub, &node.proof)?,
                node,
                next: None,
            },
        })
    }

    pub fn mutate(&mut self, claim: SdtClaim) -> Result<&mut Self, SdtError> {
        let current = self.payload.find_current();
        let node = claim.to_node()?;
        current.next = Some(Box::new(SdtPayload {
            proof: mutation_proof(&current.proof, &node.proof)?,
            node: node,
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

    pub fn verify(&self, proofs: Vec<String>) -> Result<SdtResult, SdtError> {
        let mut proofs = proofs.clone();
        if let Some(proof) = proofs.pop() {
            let mut result = SdtResult::Branch(HashMap::new());
            let node_proof = self.payload.node.verify()?;
            let inception_proof = inception_proof(&self.subject, &node_proof)?;
            if proof != inception_proof {
                eprintln!("Inception is not match: {} {} {}", node_proof, proof, inception_proof);
                return Err(SdtError::Other);
            }
            self.payload.node.disclose("", &mut result)?;
            if let Some(next) = &self.payload.next {
                next.verify(&self.payload.proof, &mut proofs, &mut result)?;
            }
                
            return Ok(result);
        }
        Err(SdtError::Other)
    }
}

fn inception_proof(sub: &str, claim_proof: &str) -> Result<String, SdtError> {
    let proof_map = SdtProofPayload::new()
        .insert_i64("hash_alg", HASH_ALG as i64)
        .insert_str("subject", sub)
        .insert_str("claim_proof", claim_proof)
        .build();

    let digest = digest(&proof_map);
    eprintln!("{:?}", digest);
    digest
}

fn mutation_proof(previous: &str, claim_proof: &str) -> Result<String, SdtError> {
    let proof_map = SdtProofPayload::new()
        .insert_str("claim_proof", claim_proof)
        .insert_str("previous", previous)
        .build();
    digest(&proof_map)
}

impl SdtPayload {
    pub fn select(&mut self, query: &str) -> Result<&mut Self, SdtError> {
        self.node.select(query)?;
        if self.next.is_none() {
            return Ok(self);
        }
        self.next.as_mut().unwrap().select(query)
    }

    pub fn verify(
        &self,
        prev: &str,
        proofs: &mut Vec<String>,
        res: &mut SdtResult,
    ) -> Result<(), SdtError> {
        if let Some(proof) = proofs.pop() {
            let node_proof = self.node.verify()?;
            let pay_proof = mutation_proof(prev, &node_proof)?;
            if proof != pay_proof {
                //eprintln!("Proof is not match: {} {}", proof, pay_proof);
                return Err(SdtError::Other);
            }
            self.node.disclose("", res)?;

            if let Some(next) = &self.next {
                return next.verify(&self.proof, proofs, res);
            }else{
                return Ok(());
            }
        }
        Err(SdtError::Other)
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
        let inception_proof = sdt.payload.proof.clone();
        let mutation_pay = sdt.payload.next.clone().unwrap();
        let mutation_proof = mutation_pay.proof.clone();
        let mutation2_proof = mutation_pay.next.unwrap().proof.clone();

        let result = sdt.verify(vec![mutation2_proof,  mutation_proof, inception_proof])?;
        sdt.select(query)?;
        eprintln!("{}", serde_json::to_string(&result)?);
        //eprintln!("{}", serde_json::to_string(&sdt)?);
        Ok(())
    }
}
