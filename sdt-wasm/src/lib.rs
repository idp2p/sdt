mod utils;

use sdt::{Sdt, value::SdtClaim};
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;


#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct CreateInput {
    pub subject: String,
    pub claim: SdtClaim  
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct MutateInput {
    pub sdt: Sdt,
    pub claim: SdtClaim  
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct DiscloseInput {
    pub sdt: Sdt,
    pub query: String  
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct ValidateInput {
    pub sdt: Sdt,
    pub query: String  
}

#[wasm_bindgen]
pub fn dispatch(cmd: &str, input: &str) -> String {
    match cmd {
        "CREATE" => {
            
        }
        _=> {}
    }
    todo!()
}