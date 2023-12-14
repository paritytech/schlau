use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

/// Builds the Solidity source in `path_to_source_sol`.
/// Returns the bytes of the compiled contract.
pub fn build_contract<P>(path_to_source_sol: P) -> anyhow::Result<Vec<u8>>
    where
        P: AsRef<Path> + Copy,
{
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

    let bin_path = PathBuf::from(manifest_dir).join("bin").join("solc");

    let output = Command::new(&bin_path)
        .arg("--bin")
        .arg(path_to_source_sol.as_ref())
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        let mut prev_line = None;
        for line in stdout.lines() {
            if prev_line == Some("Binary:") {
                return Ok(hex::decode(line.trim_start())?);
            }
            prev_line = Some(line);
        }
        Err(anyhow::anyhow!("Failed to find binary in output"))
    } else {
        Err(anyhow::anyhow!("Failed to compile contract:\n {output:?}"))
    }
}