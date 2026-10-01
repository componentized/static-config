use anyhow::{bail, Context, Result};
use componentized_constants::{
    value::{Type, Value, WasmValue},
    Overrides,
};
use std::collections::BTreeMap;
use wasm_encoder::{ComponentBuilder, ComponentExportKind};
use wasm_metadata::Producers;
use wit_component::DecodedWasm;
use wit_parser::{Resolve, WorldId, WorldItem};

/// Exports `wasi:config/store`, serving the values it imports from `config-values`.
const CONFIG: &[u8] = include_bytes!("../lib/config.wasm");

pub fn create_component(values: Vec<(String, String)>) -> Result<Vec<u8>> {
    // later values replace earlier values with the same key, served sorted by key
    let values: BTreeMap<String, String> = values.into_iter().collect();

    let DecodedWasm::Component(mut resolve, config_world) = wit_component::decode(CONFIG)? else {
        bail!("expected a component, found a WIT package");
    };
    let (config_values, _) = single(resolve.worlds[config_world].imports.iter(), "import")?;
    let (store, _) = single(resolve.worlds[config_world].exports.iter(), "export")?;
    let config_values = resolve.name_world_key(config_values);
    let store = resolve.name_world_key(store);

    let values_world = values_world(&mut resolve, config_world);
    let values = create_values_component(&resolve, values_world, values)?;

    // instantiate config with the values, exporting its store
    let mut builder = ComponentBuilder::default();
    let values = builder.component_raw(Some("values"), &values);
    let config = builder.component_raw(Some("config"), CONFIG);
    let values = builder.instantiate(None, values, Vec::<(&str, _, _)>::new());
    let values = builder.alias_export(values, &config_values, ComponentExportKind::Instance);
    let config = builder.instantiate(
        None,
        config,
        [(&config_values, ComponentExportKind::Instance, values)],
    );
    let store_export = builder.alias_export(config, &store, ComponentExportKind::Instance);
    builder.export(&store, ComponentExportKind::Instance, store_export, None);
    let component = builder.finish();
    wasmparser::Validator::new().validate_all(&component)?;

    let mut producers = Producers::default();
    producers.add(
        "processed-by",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
    producers.add_to_wasm(&component)
}

/// Adds a world to `resolve` that exports what the config world imports, the `config-values`
/// interface, to be implemented by `componentized-constants`.
fn values_world(resolve: &mut Resolve, config_world: WorldId) -> WorldId {
    let mut world = resolve.worlds[config_world].clone();
    world.name = "values".into();
    world.exports = std::mem::take(&mut world.imports);
    resolve.worlds.alloc(world)
}

/// Creates a component exporting `config-values`, returning `values`.
///
/// The interface's single function returns the values, written as an override of the form
/// `{config-values: {values: [(key, value), ...]}}`.
fn create_values_component(
    resolve: &Resolve,
    world: WorldId,
    values: BTreeMap<String, String>,
) -> Result<Vec<u8>> {
    let (_, item) = single(resolve.worlds[world].exports.iter(), "export")?;
    let WorldItem::Interface { id, .. } = item else {
        bail!("expected config values to be an interface");
    };
    let interface = &resolve.interfaces[*id];
    let interface_name = interface
        .name
        .as_deref()
        .context("expected config values to be a named interface")?;
    let (func_name, _) = single(interface.functions.iter(), "function")?;

    let pair_type = Type::tuple([Type::STRING, Type::STRING]).context("tuple type")?;
    let list_type = Type::list(pair_type.clone());
    let interface_type =
        Type::record([(func_name.as_str(), list_type.clone())]).context("record type")?;
    let overrides_type =
        Type::record([(interface_name, interface_type.clone())]).context("record type")?;

    let pairs = values
        .into_iter()
        .map(|(key, value)| {
            Value::make_tuple(
                &pair_type,
                [
                    Value::make_string(key.into()),
                    Value::make_string(value.into()),
                ],
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let overrides = Value::make_record(
        &overrides_type,
        [(
            interface_name,
            Value::make_record(
                &interface_type,
                [(func_name.as_str(), Value::make_list(&list_type, pairs)?)],
            )?,
        )],
    )?;

    componentized_constants::create_component(resolve, world, Some(Overrides::Value(&overrides)))
}

/// The only entry in `items`, failing if there are none or several.
fn single<I: ExactSizeIterator>(mut items: I, kind: &str) -> Result<I::Item> {
    match (items.len(), items.next()) {
        (1, Some(item)) => Ok(item),
        (n, _) => bail!("expected the config component to have a single {kind}, found {n}"),
    }
}
