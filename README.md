# LithumMC

LithumMC is a Minecraft Java server project written in Rust. The hosting `.jar` is a Java launcher that starts the bundled native server, so Pelican and Pterodactyl can run it with their Java startup command.

This is an early derivative server build with world and chunk handling, configurable gameplay and networking, and a sandboxed WebAssembly plugin runtime. It is not a Paper/Bukkit reimplementation, and Paper, Spigot, Bukkit, and Fabric plugin JARs do not run on LithumMC. Upstream source and license attribution are recorded in [NOTICE.md](NOTICE.md).

## Run LithumMC

Use Java 21 or newer. Place `LithumMC.jar` in the server folder and start it with:

```sh
java -jar LithumMC.jar
```

The first launch creates `lithummc.toml` and the world data. The default Java listener is `0.0.0.0:25565`.

### Change the port

For a standalone server, set a port in `lithummc.toml`:

```toml
[networking.java]
port = 25575
```

When `SERVER_PORT` is present in the environment, LithumMC uses that allocated port ahead of the config value. Pelican and Pterodactyl provide this variable for the server's assigned allocation, so the same Egg works when the host changes the port.

## Pelican and Pterodactyl

1. Ask a panel administrator to import [`deploy/egg-lithummc.json`](deploy/egg-lithummc.json), then create a LithumMC server using a Java 21 image.
2. Upload `LithumMC.jar` to the server root.
3. Start it. The Egg runs `java -jar ./LithumMC.jar`, and LithumMC reads the port allocated by the panel.

If custom Eggs are disabled, use a Java 21 egg, set its server JAR filename to `LithumMC.jar`, and make sure its startup command is `java -jar <filename>`.

## Velocity

LithumMC supports Velocity **modern forwarding**. In `velocity.toml`, set `player-info-forwarding-mode = "modern"` and use the secret stored in Velocity's `forwarding.secret` file. In `lithummc.toml`, configure the backend like this:

```toml
[networking.java]
online_mode = false

[networking.proxy]
enabled = true

[networking.proxy.velocity]
enabled = true
secret = "paste-the-same-secret-from-velocity-forwarding.secret"
```

The secret must match on both sides. Keep the backend inaccessible to direct player connections; modern forwarding adds proxy authentication, while a firewall or host access rule still protects the backend port. See the [Velocity forwarding guide](https://docs.papermc.io/velocity/player-information-forwarding/) and [Velocity security guide](https://docs.papermc.io/velocity/security/).

## Plugins

LithumMC plugins run as sandboxed WebAssembly components. The [`lithummc-plugin-api` crate](crates/lithummc-plugin-api) exposes the server's current plugin API, and [`examples/hello-lithummc-plugin`](examples/hello-lithummc-plugin) contains a starter example. This API is early and may change.

## Build and downloads

The [LithumMC Linux build workflow](.github/workflows/lithummc-linux.yml) builds an optimized Linux x86_64 server and packages `LithumMC.jar` for Java 21 hosting. It runs on pushes to `main` that change server sources and can also be started manually from GitHub Actions. Download the `LithumMC-linux-x86_64` artifact from a successful run.

To build from source, use Rust 1.97 or newer:

```sh
cargo build --locked --release --package lithummc --bin lithummc-server
```

On Windows, the packaging script also needs the MSVC C++ build tools and a Java 21 JDK:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-windows-jar.ps1
```

## License

The server is distributed under GPL-3.0. The plugin API is MIT OR Apache-2.0. See [LICENSE](LICENSE), the API license files, and [NOTICE.md](NOTICE.md) for source and third-party attribution.
