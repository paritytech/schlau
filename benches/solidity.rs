use alloy_dyn_abi::{DynSolValue, JsonAbiExt};
use alloy_json_abi::JsonAbi;
use alloy_primitives::I256;
use criterion::{criterion_group, criterion_main, Criterion};
use schlau::evm::{CallArgs, CreateArgs, EvmRuntime, EvmSandbox, DEFAULT_ACCOUNT};
use sp_core::U256;

fn computation(c: &mut Criterion) {
    let contract = schlau::solc::build_contract("contracts/solidity/computation.sol").unwrap();
    let mut sandbox = EvmSandbox::<EvmRuntime>::new();

    let abi_path = "contracts/solidity/Computation.abi";
    let json = std::fs::read_to_string(abi_path).unwrap();
    let abi: JsonAbi = serde_json::from_str(&json).unwrap();

    let create_args = CreateArgs {
        source: DEFAULT_ACCOUNT,
        init: contract,
        gas_limit: 1_000_000_000,
        max_fee_per_gas: U256::from(1_000_000_000),
        ..Default::default()
    };
    let address = sandbox.create(create_args).unwrap();
    let n = 100_000;

    let func = &abi.function("odd_product").unwrap()[0];
    let input = [DynSolValue::Int(I256::try_from(100_000).unwrap(), 32)];
    let data = func.abi_encode_input(&input).unwrap();

    let call_args = CallArgs {
        source: DEFAULT_ACCOUNT,
        target: address,
        input: data,
        gas_limit: 1_000_000_000,
        max_fee_per_gas: U256::from(1_000_000_000),
        ..Default::default()
    };

    let mut group = c.benchmark_group("computation");
    group.sample_size(30);

    let bench_name = format!("odd_product_{}", n);

    group.bench_function(bench_name, |b| {
        b.iter(|| {
            sandbox.call(call_args.clone()).unwrap();
        })
    });

    group.finish()
}

criterion_group!(benches, computation);
criterion_main!(benches);
