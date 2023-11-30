mod drink_api;
mod ink_build;

use contract_build::Target;
use drink::runtime::MinimalRuntime;

fn main() -> anyhow::Result<()> {
    let contract = "contracts/ink/crypto/Cargo.toml";
    let build_result = ink_build::build_contract(contract, Target::RiscV)?;
    let code = std::fs::read(build_result)?;

    let mut drink_api = drink_api::DrinkApi::<ink_env::DefaultEnvironment, MinimalRuntime>::new();

    let value = 0;
    let salt = Vec::new();
    let storage_deposit_limit = None;

    let mut constructor = crypto::crypto::CryptoRef::new();

    drink_api.deploy_contract(
        code,
        value,
        &mut constructor,
        salt,
        &subxt_signer::sr25519::dev::alice(),
        storage_deposit_limit,
    )?;

    Ok(())
}
