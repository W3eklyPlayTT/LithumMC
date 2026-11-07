# Lithum Server

Lithum is an experimental Minecraft server foundation written in Rust. It is not a Paper replacement yet. The current build implements the Minecraft Java server-list handshake, status response and ping/pong, plus a Rust-hosted Rhai script API for server lifecycle, one-second tick and console-command callbacks.

## Current limits

This prototype does **not** implement player login, play packets, chunks, world generation, entities, persistence, or Paper/Bukkit plugin compatibility. A client can see the server in the multiplayer list, but joining is rejected with an explanatory message. The advertised protocol number is a placeholder, so server-list version compatibility may be shown as unknown. The project does not produce a `.jar`: a Rust server ships as a native executable (or a platform archive containing it).

Horizon is a useful reference and already has a native `horizon-server`; Lithum is a separate project with a different plugin API goal. See [Horizon](https://github.com/horizon-rust/horizon).

## Build and run

Requires Rust stable and Cargo.

```powershell
cargo build --release
Copy-Item lithum.example.toml lithum.toml
.
target\release\lithum-server.exe
```

On first run, Lithum creates `lithum.toml` with defaults. Set `server.address`, `server.port`, `server.motd`, `server.max_players`, `server.max_connections`, `server.worker_threads`, and `metrics.refresh_interval_ms` there. `worker_threads = 0` uses the machine's available CPU count. The connection cap bounds concurrent TCP clients.

Console commands: `status`, `tps`, `help`, `stop`. The metrics display reports measured tick rate, process memory and current connections. The memory bar is scaled in 128 MiB increments; it is a visual guide, not a configured memory limit.

## Lithum script plugins

Drop `.rhai` files into `plugins/`. Functions are optional:

```rhai
fn on_enable() { print("plugin enabled"); }
fn on_tick(tick) { }
fn on_console_command(command) { }
```

`on_tick` fires once per second. Scripts run in Rhai's embedded engine, without filesystem or process APIs registered by this server. A plugin callback runs synchronously, so a slow or infinite script can delay the server loop; use trusted plugins only. This API is experimental and currently exposes lifecycle/console signals, not game entities or world access.

## Roadmap

1. Select and document a Minecraft protocol target; implement login/configuration/play state and online authentication.
2. Add chunk storage, world persistence, chunk generation and scheduled world/entity ticking.
3. Expand the plugin API with permissioned player, world, command and event interfaces, then stabilize its versioning.
4. Profile under realistic player and chunk loads before making performance claims.
