use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const GENERATED_FILE: &str = "rspigot.plugin.v1.rs";
const PROTO_FILE: &str = "rspigot/plugin/v1/plugin.proto";

fn main() -> ExitCode {
    let mut check = false;
    for argument in env::args().skip(1) {
        match argument.as_str() {
            "--check" => check = true,
            other => {
                eprintln!("unknown argument `{other}`, the only option is `--check`");
                return ExitCode::FAILURE;
            }
        }
    }
    match run(check) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            eprintln!(
                "{GENERATED_FILE} is out of date, run `cargo run --manifest-path tools/plugin-protocol-gen/Cargo.toml`"
            );
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("generation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(check: bool) -> Result<bool, Box<dyn Error>> {
    let crate_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/rspigot-plugin-protocol");
    let committed = crate_dir.join("src/generated").join(GENERATED_FILE);

    let scratch = env::temp_dir().join(format!("plugin-protocol-gen-{}", std::process::id()));
    fs::create_dir_all(&scratch)?;
    let fresh = generate(&crate_dir, &scratch);
    let _ = fs::remove_dir_all(&scratch);
    let fresh = fresh?;

    if check {
        Ok(fs::read(&committed).is_ok_and(|current| current == fresh))
    } else {
        if let Some(parent) = committed.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&committed, fresh)?;
        Ok(true)
    }
}

fn generate(crate_dir: &Path, scratch: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let proto_root: PathBuf = crate_dir.join("proto");
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .build_transport(false)
        .emit_rerun_if_changed(false)
        .out_dir(scratch)
        .compile_protos(&[proto_root.join(PROTO_FILE)], &[proto_root])?;
    Ok(fs::read(scratch.join(GENERATED_FILE))?)
}
