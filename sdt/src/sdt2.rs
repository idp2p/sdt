use crate::proof::SdtProof;
use std::collections::HashMap;

use crate::dto::{SdtClaim, SdtValueResult};
use crate::error::SdtError;
use crate::node2::SdtNode;
use serde::{Deserialize, Serialize};

const VERSION: u64 = 0x1;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtItem {
    pub node: SdtNode,
    pub next: Option<Box<SdtItem>>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Sdt {
    pub version: u64, // JSON + HEX + SHA256
    pub subject: String,
    pub inception: SdtItem,
}

impl SdtItem {
    pub fn find_current(&mut self) -> &mut Self {
        if self.next.is_none() {
            return self;
        }
        self.next.as_mut().unwrap().find_current()
    }

    pub fn select(&mut self, query: &str) -> Result<&mut Self, SdtError> {
        self.node.select(query)?;
        if self.next.is_none() {
            return Ok(self);
        }
        self.next.as_mut().unwrap().select(query)
    }

    pub fn verify(&self, prev: &str, res: &mut SdtValueResult) -> Result<String, SdtError> {
        let node_proof = self.node.gen_proof()?;
        let item_proof = mutation_proof(prev, &node_proof)?;
        
        self.node.disclose("", res)?;
        if let Some(next) = &self.next {
            return next.verify(&item_proof, res);
        } else {
            return Ok(item_proof);
        }
    }
}

impl Sdt {
    /*pub fn new(sub: &str, claim: SdtClaim) -> Result<Self, SdtError> {
        let node = claim.to_node()?;
        Ok(Sdt {
            version: VERSION,
            subject: sub.to_owned(),
            inception: SdtItem {
                node,
                next: None,
            },
        })
    }

    pub fn mutate(&mut self, claim: SdtClaim) -> Result<&mut Self, SdtError> {
        let current = self.inception.find_current();
        let node = claim.to_node()?;
        current.next = Some(Box::new(SdtItem {
            node,
            next: None,
        }));
        Ok(self)
    }*/

    pub fn build(&mut self) -> Result<Self, SdtError> {
        Ok(self.to_owned())
    }

    pub fn select(&mut self, query: &str) -> Result<(), SdtError> {
        self.inception.select(query)?;
        Ok(())
    }

    pub fn verify(&self, proof: &str) -> Result<SdtValueResult, SdtError> {
        let mut result = SdtValueResult::Branch(HashMap::new());
        let inception_root = self.inception.node.gen_proof()?;
        let inception_proof = inception_proof(&self.subject, &inception_root)?;
        self.inception.node.disclose("", &mut result)?;
        if let Some(next) = &self.inception.next {
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
        .insert_i64("version", VERSION as i64)
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
        /*let mut sdt = Sdt::new("did:p2p:123456", new_claim)?
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
        eprintln!("{}", serde_json::to_string(&result)?);*/
        //eprintln!("{}", serde_json::to_string(&sdt)?);
        Ok(())
    }
}
