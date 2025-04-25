use anyhow::{Error, Result};
use static_config::create_component;
use wasmtime::{
    component::{Component, Linker, ResourceTable},
    Engine, Store,
};

wasmtime::component::bindgen!({
    world: "adapter",
    path: "wit",
});

fn get_all(bytes: &[u8]) -> Result<Vec<(String, String)>> {
    let engine = Engine::default();
    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ResourceTable::new());
    let component = Component::from_binary(&engine, bytes)?;
    let instance = Adapter::instantiate(&mut store, &component, &linker)?;

    let result = instance.wasi_config_store().call_get_all(store)?;

    result.map_err(|message| Error::msg(message))
}

fn get(bytes: &[u8], key: &str) -> Result<Option<String>> {
    let engine = Engine::default();
    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ResourceTable::new());
    let component = Component::from_binary(&engine, bytes)?;
    let instance = Adapter::instantiate(&mut store, &component, &linker)?;

    let result = instance.wasi_config_store().call_get(store, key)?;

    result.map_err(|message| Error::msg(message))
}

#[test]
fn it_creates_an_empty_component() {
    let expected = vec![];
    let component = create_component(expected.clone()).expect("create_component should not error");
    assert_eq!(
        expected,
        get_all(&component).expect("get_config_values should not error"),
        "config values do not match"
    );
    assert_eq!(
        None,
        get(&component, "greeting").expect("get_config_value should not error"),
        "config value does not match"
    );
    assert_eq!(
        None,
        get(&component, "name").expect("get_config_value should not error"),
        "config value does not match"
    );
}

#[test]
fn it_creates_a_component_with_a_value() {
    let expected = vec![(String::from("greeting"), String::from("hello"))];
    let component = create_component(expected.clone()).expect("create_component should not error");
    assert_eq!(
        expected,
        get_all(&component).expect("get_config_values should not error"),
        "config values do not match"
    );
    assert_eq!(
        Some(String::from("hello")),
        get(&component, "greeting").expect("get_config_value should not error"),
        "config value does not match"
    );
    assert_eq!(
        None,
        get(&component, "name").expect("get_config_value should not error"),
        "config value does not match"
    );
}

#[test]
fn it_creates_a_component_with_multiple_values() {
    let expected = vec![
        (String::from("greeting"), String::from("hello")),
        (String::from("name"), String::from("componentized")),
    ];
    let component = create_component(expected.clone()).expect("create_component should not error");
    assert_eq!(
        expected,
        get_all(&component).expect("get_config_values should not error"),
        "config values do not match"
    );
    assert_eq!(
        Some(String::from("hello")),
        get(&component, "greeting").expect("get_config_value should not error"),
        "config value does not match"
    );
    assert_eq!(
        Some(String::from("componentized")),
        get(&component, "name").expect("get_config_value should not error"),
        "config value does not match"
    );
}
