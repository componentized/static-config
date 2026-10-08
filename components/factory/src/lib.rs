#![cfg_attr(not(test), no_main)]

use crate::exports::componentized::static_config::factory::{ErrorCode, Guest, Wasm};
use componentized_static_config::create_component;

pub(crate) struct Factory;

impl Guest for Factory {
    fn create(values: Vec<(String, String)>) -> Result<Wasm, ErrorCode> {
        let output = create_component(values).map_err(|e| ErrorCode::Other(Some(e.to_string())))?;
        Ok(output)
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "factory",
    generate_all
});

export!(Factory);
