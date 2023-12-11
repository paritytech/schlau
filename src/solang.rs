use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use contract_build::Target;

/// Builds the Solidity source in `path_to_source_sol`.
/// Returns the path to the build output directory.
///
/// Note: For RiscV the produced contract blob will still have the `.wasm` file extension.
pub fn build_contract<P>(path_to_source_sol: P, target: Target) -> anyhow::Result<PathBuf>
where
    P: AsRef<Path> + Copy,
{
    let target = match target {
        Target::RiscV => "polkadot-riscv",
        Target::Wasm => "polkadot",
    };

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let bin_path = PathBuf::from(manifest_dir).join("bin").join("solang");

    let out_dir = path_to_source_sol
        .as_ref()
        .parent()
        .expect("source file is not in root dir");

    match Command::new(&bin_path)
        .arg("compile")
        .arg("--target")
        .arg(target)
        .arg("-O")
        .arg("aggressive")
        .arg("--release")
        .arg("--wasm-opt")
        .arg("z")
        .arg("-o")
        .arg(out_dir)
        .arg(path_to_source_sol.as_ref())
        .output()
    {
        Ok(output) if output.status.success() => Ok(out_dir.to_path_buf()),
        Ok(output) => Err(anyhow::anyhow!("Failed to compile contract:\n {output:?}")),
        Err(msg) => Err(anyhow::anyhow!("Failed to execute {bin_path:?}: {msg:?}")),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    fn can_compile(target: contract_build::Target) {
        let path = PathBuf::from("contracts/solidity/compile_test");

        super::build_contract(&path.with_extension("sol"), target).unwrap();

        let len = fs::read(path.with_extension("wasm"))
            .expect("compiler should produce a contract blob")
            .len();
        assert!(len > 0, "compiler should produce a non-empty contract blob");

        let _ = std::fs::remove_file(path.with_extension("wasm"));
        let _ = std::fs::remove_file(path.with_extension("contract"));
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn can_compile_wasm() {
        can_compile(contract_build::Target::Wasm)
    }

    #[cfg(feature = "riscv")]
    #[test]
    fn can_compile_riscv() {
        can_compile(contract_build::Target::RiscV)
    }
}
