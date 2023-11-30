mod drink_api;
mod ink_build;

use contract_build::Target;
use drink::runtime::MinimalRuntime;

fn main() -> anyhow::Result<()> {
    let contract = "contracts/ink/crypto/Cargo.toml";
    let build_result = ink_build::build_contract(contract, Target::RiscV)?;
    let code = std::fs::read(build_result)?;

    let mut drink_api = drink_api::DrinkApi::<ink::env::DefaultEnvironment, MinimalRuntime>::new();

    let value = 0;
    let salt = Vec::new();
    let storage_deposit_limit = None;

    use crypto::crypto::{Crypto, CryptoRef};

    let mut constructor = CryptoRef::new();

    let contract = drink_api.ink_instantiate_with_code::<Crypto, _, _>(
        code,
        value,
        &mut constructor,
        salt,
        &subxt_signer::sr25519::dev::alice(),
        storage_deposit_limit,
    )?;

    let call = contract.sha3(1000);

    drink_api.ink_call(&subxt_signer::sr25519::dev::alice(), &call, 0, None)?;

    Ok(())
}
