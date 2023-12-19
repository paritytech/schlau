use criterion::{criterion_group, criterion_main, Criterion};
use drink_wasm::Weight;

use alloy_dyn_abi::{DynSolValue, JsonAbiExt};
use alloy_json_abi::JsonAbi;
use alloy_primitives::I256;
use schlau::evm::{
    CallArgs as EvmCallArgs, CreateArgs as EvmCreateArgs, EvmRuntime, EvmSandbox, DEFAULT_ACCOUNT,
};
use sp_core::{H160, U256};

pub struct EvmContract {
    address: H160,
    abi: JsonAbi,
    sandbox: EvmSandbox<EvmRuntime>,
}

impl EvmContract {
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

fn init_evm_contract(contract: &str) -> EvmContract {
    let result =
        schlau::solc::build_contract(&format!("../contracts/solidity/{}.sol", contract)).unwrap();
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

fn computation_evm(c: &mut Criterion) {
    let mut evm_contract = init_evm_contract("Computation");

    let mut group = c.benchmark_group("computation_evm");
    group.sample_size(30);

    let n = 100_000;
    let bench_name = format!("odd_product_{}", n);

    let input = [DynSolValue::Int(I256::try_from(n).unwrap(), 32)];
    let odd_product_args = evm_contract.call_args("odd_product", &input);

    group.bench_function(bench_name, |b| {
        b.iter(|| {
            evm_contract.sandbox.call(odd_product_args.clone()).unwrap();
        })
    });

    let n = 100_000;
    let bench_name = format!("triangle_number_{}", n);

    let input = [DynSolValue::Int(I256::try_from(n).unwrap(), 64)];
    let triangle_number_args = evm_contract.call_args("triangle_number", &input);

    group.bench_function(bench_name, |b| {
        b.iter(|| {
            evm_contract
                .sandbox
                .call(triangle_number_args.clone())
                .unwrap();
        })
    });

    group.finish()
}

fn computation_pallet_contracts(c: &mut Criterion) {
    use parity_scale_codec::Encode;
    use schlau::{
        drink::runtime::MinimalRuntime,
        drink_api::{CallArgs, CreateArgs, DrinkApi},
        solang,
    };
    use subxt_signer::sr25519::dev;

    let contract = solang::build_and_load_contract("contracts/solidity/Computation.sol").unwrap();

    let mut drink_api = DrinkApi::<MinimalRuntime>::new();

    let constructor_selector = contract.constructor_selector("new").unwrap();
    let create_args = CreateArgs::<MinimalRuntime>::new(contract.code.clone(), dev::alice())
        .with_data(constructor_selector);

    let contract_account = drink_api.instantiate_with_code(create_args).unwrap();
    println!("contract_account: {:?}", contract_account);

    let mut group = c.benchmark_group("computation_pallet_contracts");
    group.sample_size(30);

    let n = 100_000i32;
    let bench_name = format!("odd_product_{}", n);

    let mut call_data = contract.message_selector("odd_product").unwrap();
    call_data.append(&mut n.encode());

    let odd_product_args =
        CallArgs::<MinimalRuntime>::new(contract_account, dev::alice(), call_data)
            .with_gas_limit(Weight::MAX);

    group.bench_function(bench_name, |b| {
        b.iter(|| {
            drink_api.call(odd_product_args.clone()).unwrap();
        })
    });
}

criterion_group!(benches, computation_pallet_contracts);
// criterion_group!(benches, computation_evm, computation_pallet_contracts);
criterion_main!(benches);
