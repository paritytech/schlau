use criterion::{criterion_group, criterion_main, Criterion};
use drink_wasm::Weight;

use alloy_dyn_abi::{DynSolValue, JsonAbiExt};
use alloy_json_abi::JsonAbi;
use alloy_primitives::I256;
use parity_scale_codec::Encode;
use schlau::{
    drink::runtime::{AccountIdFor, MinimalRuntime},
    drink_api::{CallArgs, CreateArgs, DrinkApi},
    evm::{
        CallArgs as EvmCallArgs, CreateArgs as EvmCreateArgs, EvmRuntime, EvmSandbox,
        DEFAULT_ACCOUNT,
    },
    solang,
};
use sp_core::{H160, U256};
use subxt_signer::sr25519::dev;

pub struct EvmContract {
    address: H160,
    abi: JsonAbi,
    sandbox: EvmSandbox<EvmRuntime>,
}

impl EvmContract {
    fn init(contract: &str) -> Self {
        let result =
            schlau::solc::build_contract(&format!("contracts/solidity/{}.sol", contract)).unwrap();
        let mut sandbox = EvmSandbox::<EvmRuntime>::new();

        let create_args = EvmCreateArgs {
            source: DEFAULT_ACCOUNT,
            init: result.code,
            gas_limit: 1_000_000_000,
            max_fee_per_gas: U256::from(1_000_000_000),
            ..Default::default()
        };
        let address = sandbox.create(create_args).unwrap();
        EvmContract {
            address,
            abi: result.abi,
            sandbox,
        }
    }

    pub fn call_args(&self, func: &str, args: &[DynSolValue]) -> EvmCallArgs {
        let func = &self.abi.function(func).unwrap()[0];
        let data = func.abi_encode_input(args).unwrap();

        EvmCallArgs {
            source: DEFAULT_ACCOUNT,
            target: self.address.clone(),
            input: data,
            gas_limit: 1_000_000_000,
            max_fee_per_gas: U256::from(1_000_000_000),
            ..Default::default()
        }
    }
}

pub struct SolangContract {
    drink_api: DrinkApi<MinimalRuntime>,
    contract_account: AccountIdFor<MinimalRuntime>,
    build_result: solang::BuildResult,
}

impl SolangContract {
    pub fn init(name: &str) -> Self {
        let build_result =
            solang::build_and_load_contract(&format!("contracts/solidity/{}.sol", name)).unwrap();

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

fn triangle_number(c: &mut Criterion) {
    let n = 100_000i64;

    let mut group = c.benchmark_group(format!("triangle_number_{}", n));

    let mut solang_contract = SolangContract::init("Computation");
    let mut evm_contract = EvmContract::init("Computation");

    group.bench_function("solang", |b| {
        let triangle_number_args = solang_contract.call_args("triangle_number", n);

        b.iter(|| {
            solang_contract
                .drink_api
                .call(triangle_number_args.clone())
                .unwrap();
        })
    });

    group.bench_function("evm", |b| {
        let input = [DynSolValue::Int(I256::try_from(n).unwrap(), 64)];
        let triangle_number_args = evm_contract.call_args("triangle_number", &input);

        b.iter(|| {
            evm_contract
                .sandbox
                .call(triangle_number_args.clone())
                .unwrap();
        })
    });

    group.finish()
}

fn odd_product(c: &mut Criterion) {
    let n = 100_000i32;

    let mut solang_contract = SolangContract::init("Computation");
    let mut evm_contract = EvmContract::init("Computation");

    let mut group = c.benchmark_group(format!("odd_product_{}", n));
    group.sample_size(30);

    group.bench_function("solang", |b| {
        let odd_product_args = solang_contract.call_args("odd_product", n);

        b.iter(|| {
            solang_contract
                .drink_api
                .call(odd_product_args.clone())
                .unwrap();
        })
    });

    group.bench_function("evm", |b| {
        let input = [DynSolValue::Int(I256::try_from(n).unwrap(), 32)];
        let odd_product_args = evm_contract.call_args("odd_product", &input);

        b.iter(|| {
            evm_contract.sandbox.call(odd_product_args.clone()).unwrap();
        })
    });

    group.finish()
}

criterion_group!(benches, odd_product, triangle_number);
criterion_main!(benches);
