use alloy_dyn_abi::{DynSolValue, JsonAbiExt};
use alloy_primitives::I256;
use criterion::{criterion_group, criterion_main, Criterion};
use schlau::evm::{CallArgs, CreateArgs, EvmRuntime, EvmSandbox, DEFAULT_ACCOUNT};
use sp_core::U256;

fn computation_evm(c: &mut Criterion) {
    let result = schlau::solc::build_contract("../contracts/solidity/Computation.sol").unwrap();
    let mut sandbox = EvmSandbox::<EvmRuntime>::new();

    let create_args = CreateArgs {
        source: DEFAULT_ACCOUNT,
        init: result.code,
        gas_limit: 1_000_000_000,
        max_fee_per_gas: U256::from(1_000_000_000),
        ..Default::default()
    };
    let address = sandbox.create(create_args).unwrap();

    let mut group = c.benchmark_group("computation_evm");
    group.sample_size(30);

    let n = 100_000;
    let bench_name = format!("odd_product_{}", n);

    let func = &result.abi.function("odd_product").unwrap()[0];
    let input = [DynSolValue::Int(I256::try_from(n).unwrap(), 32)];
    let data = func.abi_encode_input(&input).unwrap();

    let odd_product_args = CallArgs {
        source: DEFAULT_ACCOUNT,
        target: address.clone(),
        input: data,
        gas_limit: 1_000_000_000,
        max_fee_per_gas: U256::from(1_000_000_000),
        ..Default::default()
    };

    group.bench_function(bench_name, |b| {
        b.iter(|| {
            sandbox.call(odd_product_args.clone()).unwrap();
        })
    });

    let n = 100_000;
    let bench_name = format!("triangle_number_{}", n);

    let func = &result.abi.function("triangle_number").unwrap()[0];
    let input = [DynSolValue::Int(I256::try_from(n).unwrap(), 64)];
    let data = func.abi_encode_input(&input).unwrap();

    let triangle_num_args = CallArgs {
        source: DEFAULT_ACCOUNT,
        target: address,
        input: data,
        gas_limit: 1_000_000_000,
        max_fee_per_gas: U256::from(1_000_000_000),
        ..Default::default()
    };

    group.bench_function(bench_name, |b| {
        b.iter(|| {
            sandbox.call(triangle_num_args.clone()).unwrap();
        })
    });

    group.finish()
}

fn computation_pallet_contracts(c: &mut Criterion) {
    use schlau::{
        drink::runtime::MinimalRuntime,
        drink_api::{CallArgs, CreateArgs, DrinkApi},
        solang,
    };
    use subxt_signer::sr25519::dev;

    let contract = solang::build_and_load_contract("contracts/solidity/Computation.sol").unwrap();

    let mut drink_api = DrinkApi::<MinimalRuntime>::new();

    let create_args = CreateArgs::<MinimalRuntime>::new(contract.code, dev::alice());

    let contract_account = drink_api.instantiate_with_code(create_args).unwrap();

    let mut group = c.benchmark_group("computation_pallet_contracts");
    group.sample_size(30);

    let n = 100_000;
    let bench_name = format!("odd_product_{}", n);

    let message = contract.abi.spec().messages().iter().find(|m| m.label() == "odd_product").unwrap();

    // todo: construct call data...
    println!("odd_product selector {:?}", message.selector());

    // let func = &abi.function("odd_product").unwrap()[0];
    // let input = [DynSolValue::Int(I256::try_from(n).unwrap(), 32)];
    // let data = func.abi_encode_input(&input).unwrap();
    //
    // let odd_product_args = CallArgs {
    //     source: DEFAULT_ACCOUNT,
    //     target: address.clone(),
    //     input: data,
    //     gas_limit: 1_000_000_000,
    //     max_fee_per_gas: U256::from(1_000_000_000),
    //     ..Default::default()
    // };
    //
    // group.bench_function(bench_name, |b| {
    //     b.iter(|| {
    //         sandbox.call(odd_product_args.clone()).unwrap();
    //     })
    // });
}

criterion_group!(benches, computation_pallet_contracts);
// criterion_group!(benches, computation_evm, computation_pallet_contracts);
criterion_main!(benches);
