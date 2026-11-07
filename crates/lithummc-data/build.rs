use std::fs::{self, File};
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};

const MC_VERSION: &str = "26.3";
const FALLBACK_WORLD_PRESET_VERSION: &str = "26.2";
const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

const WORLD_PRESET_FILES: &[&str] = &[
    "data/minecraft/worldgen/world_preset/normal.json",
    "data/minecraft/worldgen/world_preset/amplified.json",
    "data/minecraft/worldgen/world_preset/large_biomes.json",
    "data/minecraft/worldgen/world_preset/flat.json",
    "data/minecraft/worldgen/world_preset/flat_all_dimensions.json",
    "data/minecraft/worldgen/world_preset/single_biome_surface.json",
    "data/minecraft/worldgen/world_preset/debug_all_block_states.json",
    "data/minecraft/worldgen/flat_level_generator_preset/bottomless_pit.json",
    "data/minecraft/worldgen/flat_level_generator_preset/classic_flat.json",
    "data/minecraft/worldgen/flat_level_generator_preset/desert.json",
    "data/minecraft/worldgen/flat_level_generator_preset/overworld.json",
    "data/minecraft/worldgen/flat_level_generator_preset/redstone_ready.json",
    "data/minecraft/worldgen/flat_level_generator_preset/snowy_kingdom.json",
    "data/minecraft/worldgen/flat_level_generator_preset/the_void.json",
    "data/minecraft/worldgen/flat_level_generator_preset/tunnelers_dream.json",
    "data/minecraft/worldgen/flat_level_generator_preset/water_world.json",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or("Failed to resolve repository root directory")?;
    let assets_dir = repo_root.join("assets");
    let datapack_dir = assets_dir.join("datapack");
    let mc_data_dir = datapack_dir.join("data").join("minecraft");

    println!("cargo:rerun-if-changed={}", mc_data_dir.display());
    println!(
        "cargo:rerun-if-changed={}",
        assets_dir.join("conventional_tags").display()
    );

    if mc_data_dir.is_dir() {
        ensure_world_presets(&assets_dir)?;
        copy_conventional_tags(&assets_dir, &datapack_dir);
        return Ok(());
    }

    println!(
        "cargo:warning=Vanilla datapack missing at {}. Downloading Minecraft {MC_VERSION} server jar...",
        mc_data_dir.display()
    );

    if let Err(e) = ensure_datapack(&assets_dir, MC_VERSION) {
        return Err(format!(
            "Failed to download and extract vanilla datapack: {e}\n\
             Please ensure you have an active internet connection or set LITHUMMC_MINECRAFT_SERVER_JAR\n\
             or manually place Minecraft {MC_VERSION} data in {}",
            mc_data_dir.display()
        )
        .into());
    }

    ensure_world_presets(&assets_dir)?;

    Ok(())
}

fn ensure_world_presets(assets_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let datapack_dir = assets_dir.join("datapack");
    let missing_files: Vec<_> = WORLD_PRESET_FILES
        .iter()
        .filter(|path| !datapack_dir.join(path).is_file())
        .copied()
        .collect();

    if missing_files.is_empty() {
        return Ok(());
    }

    println!(
        "cargo:warning=Fetching legacy built-in world presets from Minecraft {FALLBACK_WORLD_PRESET_VERSION}..."
    );
    let jar_bytes = download_server_jar(FALLBACK_WORLD_PRESET_VERSION)?;
    let inner_bytes = {
        let mut outer_zip = zip::ZipArchive::new(Cursor::new(jar_bytes))?;
        let inner_jar_name = format!(
            "META-INF/versions/{FALLBACK_WORLD_PRESET_VERSION}/server-{FALLBACK_WORLD_PRESET_VERSION}.jar"
        );
        let mut inner_file = outer_zip.by_name(&inner_jar_name)?;
        let mut inner_bytes = Vec::with_capacity(inner_file.size() as usize);
        inner_file.read_to_end(&mut inner_bytes)?;
        inner_bytes
    };

    let mut inner_zip = zip::ZipArchive::new(Cursor::new(inner_bytes))?;
    for name in missing_files {
        let outpath = datapack_dir.join(name);
        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut source = inner_zip.by_name(name)?;
        let mut output = File::create(&outpath)?;
        io::copy(&mut source, &mut output)?;
    }

    Ok(())
}

fn copy_conventional_tags(assets_dir: &Path, datapack_dir: &Path) {
    let src = assets_dir.join("conventional_tags").join("data").join("c");
    let dst = datapack_dir.join("data").join("c");
    if src.is_dir() && !dst.is_dir() {
        let _ = copy_dir_all(&src, &dst);
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn ensure_datapack(assets_dir: &Path, version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let datapack_dir = assets_dir.join("datapack");

    let jar_bytes = if let Ok(path) = std::env::var("LITHUMMC_MINECRAFT_SERVER_JAR") {
        let path = PathBuf::from(path);
        if path.is_file() {
            println!(
                "cargo:warning=Using local server jar from {}",
                path.display()
            );
            fs::read(&path)?
        } else {
            download_server_jar(version)?
        }
    } else {
        download_server_jar(version)?
    };

    println!("cargo:warning=Extracting datapack from server jar...");
    extract_datapack(&jar_bytes, version, &datapack_dir)?;
    copy_conventional_tags(assets_dir, &datapack_dir);
    println!("cargo:warning=Datapack extraction complete.");
    Ok(())
}

fn download_server_jar(version: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let client = reqwest::blocking::Client::builder()
        .user_agent("LithumMC-Build/0.1.0")
        .build()?;

    println!("cargo:warning=Fetching version manifest from {MANIFEST_URL}...");
    let manifest: serde_json::Value = client.get(MANIFEST_URL).send()?.json()?;

    let version_entry = manifest["versions"]
        .as_array()
        .and_then(|arr| arr.iter().find(|v| v["id"].as_str() == Some(version)))
        .ok_or_else(|| format!("Minecraft version {version} not found in version manifest"))?;

    let package_url = version_entry["url"]
        .as_str()
        .ok_or("Version entry missing package url")?;

    println!("cargo:warning=Fetching version package metadata from {package_url}...");
    let pkg: serde_json::Value = client.get(package_url).send()?.json()?;

    let server_url = pkg["downloads"]["server"]["url"]
        .as_str()
        .ok_or("Package metadata missing server download url")?;

    println!("cargo:warning=Downloading server jar from {server_url}...");
    let resp = client.get(server_url).send()?;
    if !resp.status().is_success() {
        return Err(format!("Server jar download failed with status {}", resp.status()).into());
    }

    let bytes = resp.bytes()?.to_vec();
    Ok(bytes)
}

fn extract_datapack(
    server_jar_bytes: &[u8],
    version: &str,
    datapack_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let inner_bytes = {
        let mut outer_zip = zip::ZipArchive::new(Cursor::new(server_jar_bytes))?;
        let inner_jar_name = format!("META-INF/versions/{version}/server-{version}.jar");
        let mut inner_file = outer_zip.by_name(&inner_jar_name)?;
        let mut inner_bytes = Vec::with_capacity(inner_file.size() as usize);
        inner_file.read_to_end(&mut inner_bytes)?;
        inner_bytes
    };

    let mut inner_zip = zip::ZipArchive::new(Cursor::new(inner_bytes))?;
    fs::create_dir_all(datapack_dir)?;

    for i in 0..inner_zip.len() {
        let mut file = inner_zip.by_index(i)?;
        let name = file.name()?;
        if !name.starts_with("data/") {
            continue;
        }
        let outpath = datapack_dir.join(&*name);
        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}
