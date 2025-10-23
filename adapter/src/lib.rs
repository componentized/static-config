#![no_main]

mod config;

pub(crate) struct Adapter;

wit_bindgen::generate!({
    path: "../wit",
    world: "adapter",
    generate_all
});

export!(Adapter);
