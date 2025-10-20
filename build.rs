//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use probe_rs::flashing::Format::Bin;
use probe_rs::flashing::{BinOptions, DownloadOptions, FlashLoader};
use probe_rs::probe::list::Lister;
use probe_rs::{MemoryInterface, Permissions};
use std::fs::{File, read_to_string};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    println!("cargo:rerun-if-changed=flash_files");
    download_files().unwrap();
}

const STATIC_START: u64 = 0x10300000;
fn download_files() -> Result<bool, anyhow::Error> {
    let out_dir = env::var("OUT_DIR")?;

    let binding = read_to_string("flash_files")?;
    let files = binding.lines().collect::<Vec<_>>();

    let lister = Lister::new();
    let probes = lister.list_all();
    let probe = probes[0].open()?;
    let mut session = probe.attach("RP235x", Permissions::default())?;

    let mut core = session.core(0)?;

    let mut hasher = DefaultHasher::new();
    files.hash(&mut hasher);
    for file in files.iter() {
        fs::metadata(file)?.modified()?.hash(&mut hasher);
    }
    let hash = hasher.finish();

    let expected = [STATIC_START, hash];
    let mut header = [0u64; 2];
    core.read_64(STATIC_START, &mut header)?;
    drop(core);

    let missing: Vec<&&str> = files.iter()
        .filter(|f| !Path::new(&out_dir).join(f).exists())
        .collect();

    if header == expected && missing.is_empty() {
        return Ok(false);
    }

    if header != expected {
        p!(
            "header mismatch, expected {:?}, got {:?}. Reflashing...",
            expected,
            header
        );
    } else {
        p!("Missing file(s): {:?}", missing);
    }


    let mut i = STATIC_START;
    let mut loader = FlashLoader::new(
        session.target().memory_map.to_vec(),
        session.target().source().clone(),
    );
    loader.add_data(i, &expected[0].to_le_bytes())?;
    i += 8;
    loader.add_data(i, &expected[1].to_le_bytes())?;
    i += 8;

    for file in files.iter() {
        println!("cargo:rerun-if-changed={}", file);

        let size = fs::metadata(file)?.len();
        let options = BinOptions {
            base_address: Some(i),
            skip: 0,
        };
        loader.load_image(&mut session, &mut File::open(file)?, Bin(options), None)?;

        let path = Path::new(&out_dir).join(file);
        let content = format!("core::slice::from_raw_parts({} as *const u8, {})", i, size);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, content)?;

        i += size;
    }

    let mut options = DownloadOptions::default();
    options.do_chip_erase = true;
    options.keep_unwritten_bytes = true;
    options.verify = true;

    loader.commit(&mut session, options)?;

    Ok(true)
}
