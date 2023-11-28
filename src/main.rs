mod ink_build;

use contract_build::Target;
use drink::{
    pallet_contracts,
    runtime::MinimalRuntime,
    Sandbox,
    DEFAULT_GAS_LIMIT,
};

fn main() -> anyhow::Result<()> {
    let contract = "contracts/ink/crypto/Cargo.toml";
    let build_result = ink_build::build_contract(contract, Target::RiscV)?;

    let mut sandbox = Sandbox::<MinimalRuntime>::new().expect("Failed to initialize Drink! sandbox");

    let code = std::fs::read(build_result)?;
    let value = 0;
    let data = Vec::new(); // todo: use CreateBuilderPartial
    let salt = || Vec::new();
    let caller = [0u8; 32];
    let gas_limit = DEFAULT_GAS_LIMIT;
    let storage_deposit_limit = None;

    let result = sandbox.deploy_contract(
        code,
        value,
        data,
        salt(),
        caller.into(),
        gas_limit,
        storage_deposit_limit,
    );

    Ok(())
}

