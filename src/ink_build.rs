use contract_build::{
    BuildArtifacts, BuildMode, ExecuteArgs, Features, ManifestPath, Network, OptimizationPasses,
    OutputType, Target, UnstableFlags, Verbosity,
};
use std::path::{Path, PathBuf};

/// Builds the contract at `manifest_path`, returns the path to the contract
/// Wasm build artifact.
pub fn build_contract<P>(path_to_cargo_toml: P, target: Target) -> anyhow::Result<PathBuf>
where
    P: AsRef<Path> + Copy,
{
    let manifest_path = ManifestPath::new(path_to_cargo_toml).unwrap_or_else(|err| {
        panic!(
            "Invalid manifest path {}: {err}",
            path_to_cargo_toml.as_ref().display()
        )
    });
    let args = ExecuteArgs {
        manifest_path,
        verbosity: Verbosity::Default,
        build_mode: BuildMode::Release,
        features: Features::default(),
        network: Network::Online,
        build_artifact: BuildArtifacts::CodeOnly,
        unstable_flags: UnstableFlags::default(),
        optimization_passes: Some(OptimizationPasses::default()),
        keep_debug_symbols: false,
        output_type: OutputType::HumanReadable,
        skip_wasm_validation: false,
        target,
        ..Default::default()
    };

    let build_result = contract_build::execute(args)?;

    let code_artifact_path = build_result
        .dest_wasm
        .expect("Wasm code artifact not generated")
        .canonicalize()?;

    Ok(code_artifact_path)
}
