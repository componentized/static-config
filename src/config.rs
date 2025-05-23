use anyhow::{bail, Context, Result};
use serde::Deserialize;
use walrus::{ir::Value, ConstExpr, DataKind, ExportItem, GlobalKind, Module};

use crate::walrus_ops::{bump_stack_global, get_active_data_segment};

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Set specific runtime overrides
    #[serde(default)]
    pub overrides: Vec<(String, String)>,
}

pub(crate) fn create_config<'a>(module: &'a mut Module, config: &Config) -> Result<()> {
    let config_ptr_addr = {
        let config_ptr_export = module
            .exports
            .iter()
            .find(|expt| expt.name.as_str() == "CONFIG")
            .context("Adapter 'CONFIG' is not exported")?;
        let ExportItem::Global(config_ptr_global) = config_ptr_export.item else {
            bail!("Adapter 'config' not a global");
        };
        let GlobalKind::Local(ConstExpr::Value(Value::I32(config_ptr_addr))) =
            &module.globals.get(config_ptr_global).kind
        else {
            bail!("Adapter 'config' not a local I32 global value");
        };
        *config_ptr_addr as u32
    };

    let memory = module.get_memory_id()?;

    // prepare the field data list vector for writing
    // strings must be sorted as binary searches are used against this data
    let mut field_data_vec: Vec<&str> = Vec::new();
    let mut sorted_overrides = config.overrides.clone();
    sorted_overrides.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (key, value) in &sorted_overrides {
        field_data_vec.push(key.as_ref());
        field_data_vec.push(value.as_ref());
    }

    let mut field_data_bytes = Vec::new();
    for str in field_data_vec {
        assert!(field_data_bytes.len() % 4 == 0);
        // write the length at the aligned offset
        field_data_bytes.extend_from_slice(&(str.len() as u32).to_le_bytes());
        let str_bytes = str.as_bytes();
        field_data_bytes.extend_from_slice(str_bytes);
        let rem = str_bytes.len() % 4;
        // add padding for alignment if necessary
        if rem > 0 {
            field_data_bytes.extend((0..4 - rem).map(|_| 0));
        }
    }

    if field_data_bytes.len() % 8 != 0 {
        field_data_bytes.resize(field_data_bytes.len() + 4, 0);
    }

    let field_data_addr = if field_data_bytes.len() > 0 {
        // Offset the stack global by the static field data length
        let field_data_addr = bump_stack_global(module, field_data_bytes.len() as i32)?;

        // Add a new data segment for this new range created at the top of the stack
        module.data.add(
            DataKind::Active {
                memory,
                offset: ConstExpr::Value(Value::I32(field_data_addr as i32)),
            },
            field_data_bytes,
        );
        Some(field_data_addr)
    } else {
        None
    };

    // In the existing static data segment, update the static data options.
    //
    // From virtual-adapter/src/config.rs:
    //
    // #[repr(C)]
    // pub struct Config {
    //     /// [byte 0]
    //     unused: bool,
    //     /// How many host fields are defined in the data pointer
    //     /// [byte 4]
    //     host_field_cnt: u32,
    //     /// Byte data of u32 byte len followed by string bytes
    //     /// up to the lengths previously provided.
    //     /// [byte 8]
    //     host_field_data: *const u8,
    // }
    let (data, data_offset) = get_active_data_segment(module, memory, config_ptr_addr)?;
    let bytes = data.value.as_mut_slice();

    let host_field_cnt = config.overrides.len() as u32;
    bytes[data_offset + 4..data_offset + 8].copy_from_slice(&host_field_cnt.to_le_bytes());
    if let Some(field_data_addr) = field_data_addr {
        bytes[data_offset + 8..data_offset + 12].copy_from_slice(&field_data_addr.to_le_bytes());
    }

    Ok(())
}
