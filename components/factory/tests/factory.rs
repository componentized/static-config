//! Exercises the factory component built by make into `target/components/factory/factory.wasm`,
//! running the components it builds.

use anyhow::{bail, Context, Result};
use test_harness::built_component;
use wasmtime::{
    component::{Component, Instance, Linker, Val},
    Engine, Store,
};

/// The factory interface, at the version of the wit package, which shares the crates' version.
const FACTORY_INTERFACE: &str = concat!(
    "componentized:static-config/factory@",
    env!("CARGO_PKG_VERSION")
);
const STORE_INTERFACE: &str = "wasi:config/store@0.2.0-rc.1";

/// Calls the function `name` exported by `interface`.
fn call(
    store: &mut Store<()>,
    instance: &Instance,
    interface: &str,
    name: &str,
    params: &[Val],
) -> Result<Val> {
    let parent = instance
        .get_export_index(&mut *store, None, interface)
        .with_context(|| format!("missing export {interface}"))?;
    let index = instance
        .get_export_index(&mut *store, Some(&parent), name)
        .with_context(|| format!("missing export {interface}#{name}"))?;
    let func = instance
        .get_func(&mut *store, index)
        .with_context(|| format!("{interface}#{name} is not a function"))?;
    let mut results = [Val::Bool(false)];
    func.call(&mut *store, params, &mut results)?;
    Ok(results[0].clone())
}

/// Instantiates a component that imports nothing.
fn instantiate(engine: &Engine, bytes: &[u8]) -> Result<(Store<()>, Instance)> {
    let component = Component::from_binary(engine, bytes)?;
    let mut store = Store::new(engine, ());
    let instance = Linker::new(engine).instantiate(&mut store, &component)?;
    Ok((store, instance))
}

/// Builds a component from `values` with the factory component.
fn build_component(engine: &Engine, values: &[(&str, &str)]) -> Result<Vec<u8>> {
    let path = built_component("factory")?;
    let bytes =
        std::fs::read(&path).with_context(|| format!("unable to read {}", path.display()))?;
    let (mut store, instance) = instantiate(engine, &bytes)?;

    let values = Val::List(
        values
            .iter()
            .map(|(k, v)| Val::Tuple(vec![Val::String(k.to_string()), Val::String(v.to_string())]))
            .collect(),
    );
    match call(
        &mut store,
        &instance,
        FACTORY_INTERFACE,
        "create",
        &[values],
    )? {
        Val::Result(Ok(Some(bytes))) => match *bytes {
            Val::List(bytes) => bytes
                .into_iter()
                .map(|byte| match byte {
                    Val::U8(byte) => Ok(byte),
                    other => bail!("unexpected byte {other:?}"),
                })
                .collect(),
            other => bail!("unexpected bytes {other:?}"),
        },
        other => bail!("expected ok, got {other:?}"),
    }
}

/// Calls `wasi:config/store#get` on the component, unwrapping its `result<option<string>>`.
fn get(engine: &Engine, bytes: &[u8], key: &str) -> Result<Option<String>> {
    let (mut store, instance) = instantiate(engine, bytes)?;
    match call(
        &mut store,
        &instance,
        STORE_INTERFACE,
        "get",
        &[Val::String(key.into())],
    )? {
        Val::Result(Ok(Some(value))) => match *value {
            Val::Option(None) => Ok(None),
            Val::Option(Some(value)) => match *value {
                Val::String(value) => Ok(Some(value)),
                other => bail!("unexpected value {other:?}"),
            },
            other => bail!("unexpected option {other:?}"),
        },
        other => bail!("expected ok, got {other:?}"),
    }
}

#[test]
fn it_builds_a_component_serving_the_values() -> Result<()> {
    let engine = Engine::default();
    let component = build_component(&engine, &[("greeting", "hello"), ("name", "componentized")])?;
    assert_eq!(get(&engine, &component, "greeting")?, Some("hello".into()));
    assert_eq!(
        get(&engine, &component, "name")?,
        Some("componentized".into())
    );
    assert_eq!(get(&engine, &component, "missing")?, None);
    Ok(())
}

#[test]
fn it_builds_an_empty_component() -> Result<()> {
    let engine = Engine::default();
    let component = build_component(&engine, &[])?;
    assert_eq!(get(&engine, &component, "greeting")?, None);
    Ok(())
}
