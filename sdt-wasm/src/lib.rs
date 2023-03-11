mod utils;
use sdt::service::SdtInput;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub fn dispatch(input: &str) -> String {
    let input = SdtInput::from_str(input);
    input.unwrap().execute().unwrap();
    todo!()
}
