use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};
use alloy_json_abi::JsonAbi;

/// Builds the Solidity source in `path_to_source_sol`.
/// Returns the bytes of the compiled contract.
pub fn build_contract<P>(path_to_source_sol: P) -> anyhow::Result<BuildResult>
where
    P: AsRef<Path> + Copy,
{
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

    let bin_path = PathBuf::from(manifest_dir).join("bin").join("solc");

    let abi_str = run_and_extract_output(path_to_source_sol, &bin_path, "--abi", "Contract JSON ABI")?;
    let abi: JsonAbi = serde_json::from_str(&abi_str)?;

    let code_hex = run_and_extract_output(path_to_source_sol, &bin_path, "--bin", "Binary:")?;
    let code = hex::decode(code_hex)?;

    Ok(BuildResult {
        abi,
        code,
    })
}

fn run_and_extract_output<P>(path_to_source_sol: P, bin_path: &PathBuf, arg: &str, prefix: &str) -> anyhow::Result<String>
where
    P: AsRef<Path> + Copy,
{
    let output = Command::new(&bin_path)
        .arg(arg)
        .arg(path_to_source_sol.as_ref())
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        let mut prev_line = None;
        for line in stdout.lines() {
            if prev_line == Some(prefix) {
                return Ok(line.trim_start().to_owned());
            }
            prev_line = Some(line);
        }
        Err(anyhow::anyhow!("Failed to find '{prefix}' in output"))
    } else {
        Err(anyhow::anyhow!("Failed to execute command fod contract:\n {output:?}"))
    }
}

#[derive(Debug)]
pub struct BuildResult {
    pub abi: JsonAbi,
    pub code: Vec<u8>,
}
