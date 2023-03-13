mod utils;
use sdt::service::{SdtService};
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub fn dispatch(input: &str) -> String {
    let service = SdtService(input.to_owned());
    service.execute()
}
