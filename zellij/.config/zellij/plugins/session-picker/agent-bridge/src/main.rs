mod agents;
mod aliases;
mod commands;
mod fs;
mod hook;
mod installer;
mod process;
mod store;

use std::env;

fn main() {
    if let Err(error) = run() {
        eprintln!("session-picker-agent-bridge: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("hook") => {
            if args.next().as_deref() != Some("--agent") {
                return Err("usage: hook --agent codex|claude".into());
            }
            let agent = commands::parse_agent(args.next().as_deref())?;
            commands::ingest_hook(agent)
        }
        Some("list") => commands::list_agents(),
        Some("mark-seen") => commands::mark_seen(args.next().ok_or("missing agent id")?),
        Some("rename-session") => {
            let old = args.next().ok_or("missing old session name")?;
            let new = args.next().ok_or("missing new session name")?;
            commands::rename_session(&old, &new)
        }
        Some("install-hooks") => installer::install_hooks(args.any(|a| a == "--dry-run"), false),
        Some("uninstall-hooks") => installer::install_hooks(args.any(|a| a == "--dry-run"), true),
        Some("doctor") => installer::doctor(),
        _ => Err("usage: session-picker-agent-bridge <hook|list|mark-seen|rename-session|install-hooks|uninstall-hooks|doctor>".into()),
    }
}
