# Lithum Server

Lithum is a Minecraft Java server built in Rust. This first playable build packages the real game server, world handling, chunk loading, and a sandboxed WebAssembly plugin runtime. The server engine is based on [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin); the Lithum name and package do not mean the engine was rewritten from scratch.

## Run the JAR on Windows

1. Install Java 21 or newer.
2. Put `LithumServer.jar` in a new, empty server folder.
3. Run `java -jar LithumServer.jar` from that folder.
4. The first start creates `lithum.toml` and the world files. Review the configuration and restart the server.
5. Connect with a Java Edition client for Minecraft 26.3 at `localhost:25565`.

The JAR is a Java launcher that extracts and starts the bundled Windows x64 Rust server. It is not a Java reimplementation and this first JAR does not run on Linux or macOS. The source can also be built as a native server executable.

## Pelican and Pterodactyl hosting

Pelican and Pterodactyl run game servers in Linux Docker containers, so use the Linux executable rather than the Windows JAR. The `Build Lithum Linux server` workflow creates a Linux x86_64 executable artifact after changes are pushed to `main` (or when started manually from GitHub Actions).

1. Download the `LithumServer-linux-x86_64` artifact from a successful workflow run and extract it.
2. Import [`deploy/egg-lithum.json`](deploy/egg-lithum.json) through the panel administrator's custom Eggs page. Choose the Debian Docker image for that panel.
3. Create a Lithum server with a Java Edition allocation, then upload `lithum-server-linux-x86_64` into the server's root folder.
4. Start the server. The Egg makes the executable runnable, and the first launch creates `lithum.toml` and the world files. The Egg updates the Java bind port to the allocation chosen in the panel.

Importing an Egg requires panel administrator access. If the host does not allow custom Eggs, ask its administrator to add this Egg or use a host that allows a custom Linux executable.

## Plugins

Lithum plugins are sandboxed WebAssembly components. Paper, Bukkit, Spigot, and Fabric plugin JARs are not compatible. The `lithum-plugin-api` Rust crate is a Lithum-named compatibility re-export of Pumpkin's permissively licensed plugin API. The initial API/runtime is early and may change.

The example plugin in `examples/hello-lithum-plugin` shows the lifecycle entry point. Build it with a Rust toolchain that has the `wasm32-wasip2` target installed, then put the generated `.wasm` component in the server's `plugins` directory.

## Configuration and performance

Lithum creates `lithum.toml` on first launch. It includes networking, world and chunk handling, lighting, plugin permissions, and other server settings. Chunk generation and lighting use worker threads; the engine also has chunk scheduling and entity-island options. Defaults aim to preserve vanilla behavior, so tune view distance and simulation distance for the host machine and player count.

This build does not include a graphical admin panel or a live RAM/TPS dashboard. Server health and performance telemetry are a follow-up feature.

## Build the Windows JAR

Requirements: Rust 1.97 or newer, the MSVC C++ build tools, and a Java 21+ JDK.

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-windows-jar.ps1
```

The script writes `dist/LithumServer.jar` and `dist/lithum-server.exe`.

## Source and license

Lithum includes code from [Pumpkin-MC/Pumpkin](https://github.com/Pumpkin-MC/Pumpkin), used under GPL-3.0. The upstream source commit is recorded in [NOTICE.md](NOTICE.md). The server and this derivative distribution are GPL-3.0; the upstream plugin API and Lithum API compatibility crate are MIT OR Apache-2.0. See the license files in `crates/pumpkin-plugin-api` for those plugin API terms.

## Contributions

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md)

## Docs

Pumpkin's documentation can be found at <https://pumpkinmc.org/>

## Communication

Consider joining [our Discord server](https://discord.gg/wT8XjrjKkf) to stay up-to-date on events, updates, and connect with other members.

## Funding

If you want to fund me and help the project, check out the [Donation Page](https://pumpkinmc.org/donate/).

## License & Attribution

* **Pumpkin Server**: Licensed under the [GNU General Public License v3.0 (GPLv3)](LICENSE).
* **Plugin API (`pumpkin-plugin-api` & `pumpkin-plugin-wit`)**: Dual-licensed under [MIT](crates/pumpkin-plugin-api/LICENSE-MIT) OR [Apache-2.0](crates/pumpkin-plugin-api/LICENSE-APACHE) for maximum flexibility when writing plugins.
* **Third-Party Assets & Data**: Bedrock mappings, protocol conversion data, and Minecraft assets are subject to their respective licenses and attribution terms. See [assets/NOTICE.md](assets/NOTICE.md) for full details.
