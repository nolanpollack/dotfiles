use std::env;
use std::fs;

use serde_json::{json, Value};

use crate::agents;
use crate::fs as atomic_fs;
use crate::store;

pub(crate) fn install_hooks(dry_run: bool, uninstall: bool) -> Result<(), String> {
    let executable = env::current_exe().map_err(|e| e.to_string())?;
    for agent in agents::AGENTS {
        let path = agent.config_path()?;
        let mut root = if path.exists() {
            serde_json::from_str::<Value>(&fs::read_to_string(&path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("{}: {e}", path.display()))?
        } else {
            json!({})
        };
        if uninstall {
            agent.uninstall(&mut root)?;
        } else {
            agent.install(&mut root, &executable)?;
        }
        let rendered = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())? + "\n";
        if dry_run {
            println!("# {}\n{}", path.display(), rendered);
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        if path.exists() {
            fs::copy(&path, path.with_extension(format!("json.bak.{}", store::now_ms())))
                .map_err(|e| e.to_string())?;
        }
        atomic_fs::atomic_write(&path, rendered.as_bytes())?;
        println!(
            "{} {}",
            if uninstall { "updated" } else { "installed" },
            path.display()
        );
    }
    Ok(())
}

pub(crate) fn doctor() -> Result<(), String> {
    println!(
        "binary: {}",
        env::current_exe().map_err(|e| e.to_string())?.display()
    );
    println!("state: {}", store::state_root()?.display());
    for agent in agents::AGENTS {
        let path = agent.config_path()?;
        let installed = agent.is_installed(&path);
        println!(
            "{} hooks: {} ({})",
            agent.label(),
            if installed { "installed" } else { "missing" },
            path.display()
        );
    }
    Ok(())
}
