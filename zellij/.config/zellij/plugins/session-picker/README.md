# Session picker

The Zellij session picker includes a native lifecycle bridge for Codex and Claude Code. Hooks write
agent state while the picker is closed; the WASM plugin queries the bridge when it is visible.

## Build and install

Builds run from `~/stripe` because binaries executed from the dotfiles tree can be blocked locally.

```sh
make test
make check-wasm
make install-binaries
make hooks-dry-run
make install-hooks
```

The hook installer preserves existing hook groups, creates timestamped backups, and owns only
commands marked `SESSION_PICKER_AGENT_HOOK=1`. After installation, open `/hooks` in Codex and trust
the new definitions. Restart existing Codex and Claude Code sessions so they load the new hooks.

Use `make doctor` to verify the binary and hook configuration. Agent snapshots live under
`~/.local/state/session-picker/agents` and are removed as soon as their owning process exits.

Set `agent_bridge_path` on the `session-manager` plugin alias if the bridge is installed somewhere
other than a directory on Zellij's `PATH`.
