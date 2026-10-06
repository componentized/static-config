//! Helpers for tests that run the components built into `target/components/` by make.

use anyhow::{bail, Context, Result};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

/// Root directory of the workspace.
pub fn workspace_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace dir")
}

/// Directory containing the built components.
pub fn components_dir() -> PathBuf {
    workspace_dir().join("target/components")
}

/// Path to the named component in `target/components/`, e.g. `factory`, rebuilt with make unless
/// it was already built by this process.
pub fn built_component(name: &str) -> Result<PathBuf> {
    static BUILT: Mutex<Option<HashSet<String>>> = Mutex::new(None);

    // hold the lock while building so concurrent tests don't run make over each other
    let mut built = BUILT.lock().unwrap_or_else(|err| err.into_inner());
    let built = built.get_or_insert_with(HashSet::new);
    // the make target for a component, its files are named by absolute path
    let target = format!("components/{name}");
    if !built.contains(name) {
        let output = Command::new("make")
            .arg("-C")
            .arg(workspace_dir())
            .arg(&target)
            .output()
            .context("failed to run make")?;
        if !output.status.success() {
            bail!(
                "failed to build {target}:\n{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        built.insert(name.to_string());
    }
    Ok(components_dir().join(name).join(format!("{name}.wasm")))
}
