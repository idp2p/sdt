pub mod error;
pub mod node;
pub mod utils;
pub mod value;
use error::SdtError;
use node::SdtNode;
use serde::{Deserialize, Serialize};
use utils::*;
use value::SdtClaim;


#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Sdt {
    nodes: Vec<SdtNode>,
}

impl Sdt {
    pub fn create(sub: &str, claim: SdtClaim) -> Result<Self, SdtError> {
        todo!()
    }

    pub fn mutate(&self, claim: SdtClaim) -> Result<Self, SdtError> {
        todo!()
    }

    pub fn disclose(&self, query: &str) -> Result<Self, SdtError> {
        todo!()
    }

    pub fn validate(&self, query: &str) -> Result<Self, SdtError>{
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdt_test() {
        
    }
}
