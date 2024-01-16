use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    drink::runtime::{AccountIdFor, MinimalRuntime},
    drink::Weight,
    drink_api::{CallArgs, CreateArgs, DrinkApi},
};
use contract_build::Target;
use parity_scale_codec::Encode;
use subxt_signer::sr25519::dev;

pub struct SolangContract {
    pub drink_api: DrinkApi<MinimalRuntime>,
    contract_account: AccountIdFor<MinimalRuntime>,
    build_result: BuildResult,
}

impl SolangContract {
    pub fn init(name: &str) -> Self {
        let build_result =
            build_and_load_contract(&format!("contracts/solidity/{}.sol", name)).unwrap();

        let mut drink_api = DrinkApi::<MinimalRuntime>::new();

        let constructor_selector = build_result.constructor_selector("new").unwrap();
        let create_args =
            CreateArgs::<MinimalRuntime>::new(build_result.code.clone(), dev::alice())
                .with_data(constructor_selector);

        let contract_account = drink_api.instantiate_with_code(create_args).unwrap();

        Self {
            drink_api,
            contract_account,
            build_result,
        }
    }

    pub fn call_args<Args: Encode>(&self, message: &str, args: Args) -> CallArgs<MinimalRuntime> {
        let mut call_data = self.build_result.message_selector(message).unwrap();
        call_data.append(&mut args.encode());

        CallArgs::<MinimalRuntime>::new(self.contract_account.clone(), dev::alice(), call_data)
            .with_gas_limit(Weight::MAX)
    }
}

/// Builds the Solidity source in `path_to_source_sol`.
/// Returns the path to the build output directory.
///
/// For each each contract found in the source code, solang creates two files:
/// - `contract_name.wasm`: The code blob.
/// - `contract_name.contract`: The full contract artifact including metadata.
/// Where `contract_name` is equal to the name of that contract
///
/// Note: For RiscV the produced contract blob does still have the `.wasm` file extension.
pub fn build_contract<P>(path_to_source_sol: P, target: Target) -> anyhow::Result<PathBuf>
where
    P: AsRef<Path> + Copy,
{
    let target = match target {
        Target::RiscV => "polkadot-riscv",
        Target::Wasm => "polkadot",
    };

    let bin_path = PathBuf::from("bin").join("solang");

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

pub fn build_and_load_contract<P>(path_to_source_sol: P) -> anyhow::Result<BuildResult>
where
    P: AsRef<Path> + Copy,
{
    let target = crate::target();
    let out_dir = build_contract(path_to_source_sol, target)?;
    let contract_name = path_to_source_sol
        .as_ref()
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let contract_path = out_dir.join(format!("{}.contract", contract_name));
    let contract = contract_metadata::ContractMetadata::load(contract_path)?;
    let source = contract.source.wasm.ok_or(anyhow::anyhow!(
        "Contract {} does not contain a wasm blob",
        contract_name
    ))?;

    Ok(BuildResult {
        code: source.0,
        abi: contract.abi,
    })
}

pub struct BuildResult {
    pub code: Vec<u8>,
    pub abi: serde_json::Map<String, serde_json::Value>,
}

impl BuildResult {
    pub fn constructor_selector(&self, constructor: &str) -> anyhow::Result<Vec<u8>> {
        self.selector("constructors", constructor)
    }

    pub fn message_selector(&self, message: &str) -> anyhow::Result<Vec<u8>> {
        self.selector("messages", message)
    }

    fn selector(&self, constructors_or_messages: &str, label: &str) -> anyhow::Result<Vec<u8>> {
        let spec = self
            .abi
            .get("spec")
            .ok_or(anyhow::anyhow!("Contract does not contain a spec field"))?;
        let messages = spec
            .get(constructors_or_messages)
            .ok_or(anyhow::anyhow!(
                "Contract does not contain a '{}' field",
                spec
            ))?
            .as_array()
            .ok_or(anyhow::anyhow!("'{}' should be an array", spec))?;
        let message = messages
            .iter()
            .find(|m| m["label"] == label)
            .ok_or(anyhow::anyhow!("{} not found", label))?;
        let selector = message
            .get("selector")
            .ok_or(anyhow::anyhow!("{} has no selector", message))?
            .as_str()
            .ok_or(anyhow::anyhow!("Selector should be a string"))?;
        Ok(hex::decode(selector.trim_start_matches("0x"))?)
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
