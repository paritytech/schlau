use super::drink::{
    pallet_balances, pallet_contracts,
    runtime::{AccountIdFor, Runtime as RuntimeT},
    BalanceOf,
};
use super::drink_api::{CallArgs, ContractsBalanceOf, DrinkApi};
use crate::drink_api::CreateArgs;
use contract_build::{
    BuildArtifacts, BuildMode, ExecuteArgs, Features, ManifestPath, Network, OptimizationPasses,
    OutputType, Target, UnstableFlags, Verbosity,
};
use ink::{
    codegen::ContractCallBuilder,
    env::{
        call::{
            utils::{ReturnType, Set, Unset},
            Call, CreateBuilder, ExecutionInput, FromAccountId,
        },
        ContractReference, Environment,
    },
};
use parity_scale_codec::{Decode, Encode};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use subxt_signer::sr25519::{dev, Keypair};

pub struct InkDrink<E: Environment, Runtime: RuntimeT> {
    pub drink: DrinkApi<Runtime>,
    _phantom: PhantomData<E>,
}

impl<E, Runtime> InkDrink<E, Runtime>
where
    E: Environment,
    E::AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
    Runtime: RuntimeT + pallet_balances::Config + pallet_contracts::Config,
    AccountIdFor<Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
    BalanceOf<Runtime>: From<u128>,
    ContractsBalanceOf<Runtime>: From<u128>,
{
    pub fn new() -> Self {
        let drink = DrinkApi::new();
        Self {
            drink,
            _phantom: PhantomData,
        }
    }

    pub fn build_and_instantiate<P, Contract, Args, R>(
        &mut self,
        contract: P,
        constructor: &mut CreateBuilderPartial<E, <Contract as ContractReference>::Type, Args, R>,
    ) -> <Contract as ContractCallBuilder>::Type
    where
        P: AsRef<Path> + Copy,
        Contract: ContractReference + ContractCallBuilder,
        <Contract as ContractReference>::Type: Clone,
        <Contract as ContractCallBuilder>::Type: FromAccountId<E>,
        Args: Encode + Clone,
    {
        let target = crate::target();
        let build_result = build_contract(contract, target).expect("Error building contract");
        let code = std::fs::read(build_result).expect("Error loading contract");

        let caller = dev::alice();
        let data = constructor_exec_input(constructor.clone());
        let create_args = CreateArgs::new(code, caller).with_data(data);

        let account_id = self
            .drink
            .instantiate_with_code(create_args)
            .expect("Error instantiating contract");
        <<Contract as ContractCallBuilder>::Type as FromAccountId<E>>::from_account_id(
            E::AccountId::from(*account_id.as_ref()),
        )
    }
}

impl<Runtime: RuntimeT + pallet_contracts::Config> CallArgs<Runtime> {
    pub fn from_call_builder<E: Environment, Args: Encode + Clone, RetType: Decode>(
        caller: Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
    ) -> Self
    where
        E::AccountId: AsRef<[u8; 32]>,
        CallBuilderFinal<E, Args, RetType>: Clone,
        AccountIdFor<Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
    {
        let account_id = message.clone().params().callee().clone();
        let account_id = (*account_id.as_ref()).into();
        let exec_input = Encode::encode(message.clone().params().exec_input());

        Self::new(account_id, caller, exec_input)
    }
}

/// The type returned from `ContractRef` constructors, partially initialized with the
/// execution input arguments.
pub type CreateBuilderPartial<E, ContractRef, Args, R> = CreateBuilder<
    E,
    ContractRef,
    Unset<<E as Environment>::Hash>,
    Unset<u64>,
    Unset<<E as Environment>::Balance>,
    Set<ExecutionInput<Args>>,
    Unset<ink::env::call::state::Salt>,
    Set<ReturnType<R>>,
>;

/// Get the encoded constructor arguments from the partially initialized `CreateBuilder`
pub fn constructor_exec_input<E, ContractRef, Args, R>(
    builder: CreateBuilderPartial<E, ContractRef, Args, R>,
) -> Vec<u8>
where
    E: Environment,
    Args: Encode,
{
    // set all the other properties to default values, we only require the `exec_input`.
    builder
        .endowment(0u32.into())
        .code_hash(ink::primitives::Clear::CLEAR_HASH)
        .salt_bytes(Vec::new())
        .params()
        .exec_input()
        .encode()
}

/// Represents an initialized contract message builder.
pub type CallBuilderFinal<E, Args, RetType> = ink::env::call::CallBuilder<
    E,
    Set<Call<E>>,
    Set<ExecutionInput<Args>>,
    Set<ReturnType<RetType>>,
>;

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
