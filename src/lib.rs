use contract_build::Target;
use drink::runtime::{AccountIdFor, Runtime as RuntimeT};
use drink::{pallet_balances, pallet_contracts, BalanceOf};
use ink::codegen::ContractCallBuilder;
use ink::env::call::FromAccountId;
use ink::env::{ContractReference, Environment};
use parity_scale_codec::Encode;
use std::path::Path;

use self::drink_api::{ContractsBalanceOf, CreateBuilderPartial};
pub use drink_api::DrinkApi;

pub mod drink_api;
pub mod ink_build;

pub fn ink_build_and_instantiate<P, Runtime: RuntimeT, E: Environment, Contract, Args, R>(
    contract: P,
    target: Target,
    constructor: &mut CreateBuilderPartial<E, <Contract as ContractReference>::Type, Args, R>,
    drink_api: &mut DrinkApi<E, Runtime>,
) -> <Contract as ContractCallBuilder>::Type
where
    E: Environment,
    E::AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
    E::Hash: Copy + From<[u8; 32]>,
    Runtime: RuntimeT + pallet_balances::Config + pallet_contracts::Config,
    P: AsRef<Path> + Copy,
    Contract: ContractReference + ContractCallBuilder,
    <Contract as ContractReference>::Type: Clone,
    <Contract as ContractCallBuilder>::Type: FromAccountId<E>,
    Args: Encode + Clone,
    AccountIdFor<Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
    BalanceOf<Runtime>: From<u128>,
    ContractsBalanceOf<Runtime>: From<u128>,
{
    let build_result =
        ink_build::build_contract(contract, target).expect("Error building contract");
    let code = std::fs::read(build_result).expect("Error loading contract");

    let value = ContractsBalanceOf::<Runtime>::from(0u128);
    let salt = Vec::new();
    let storage_deposit_limit = None;

    drink_api
        .ink_instantiate_with_code::<Contract, Args, R>(
            code,
            value.into(),
            constructor,
            salt,
            &subxt_signer::sr25519::dev::alice(),
            storage_deposit_limit,
        )
        .expect("Error instantiating contract")
}
