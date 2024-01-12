use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
};

use alloy_dyn_abi::DynSolValue;
use alloy_primitives::{I256, U256};
use parity_scale_codec::Encode;
use schlau::{evm::EvmContract, solang::SolangContract};

fn bench_evm(
    group: &mut BenchmarkGroup<WallTime>,
    contract: &str,
    message: &str,
    args: &[DynSolValue],
) {
    let mut evm_contract = EvmContract::init(contract);

    let args = evm_contract.call_args(message, args);

    group.bench_function("evm", |b| {
        b.iter(|| {
            evm_contract.sandbox.call(args.clone()).unwrap();
        })
    });
}

fn bench_solang<Args: Encode>(
    group: &mut BenchmarkGroup<WallTime>,
    contract: &str,
    message: &str,
    args: Args,
) {
    let mut solang_contract = SolangContract::init(contract);

    let args = solang_contract.call_args(message, args);

    group.bench_function("solang", |b| {
        b.iter(|| {
            solang_contract.drink_api.call(args.clone()).unwrap();
        })
    });
}

fn triangle_number(c: &mut Criterion) {
    let n = 100_000i64;
    let n_evm = DynSolValue::Int(I256::try_from(n).unwrap(), 64);

    let mut group = c.benchmark_group(format!("triangle_number_{}", n));
    group.sample_size(20);

    bench_solang(&mut group, "Computation", "triangle_number", n);
    bench_evm(&mut group, "Computation", "triangle_number", &[n_evm]);

    group.finish()
}

fn odd_product(c: &mut Criterion) {
    let n = 100_000i32;
    let n_evm = DynSolValue::Int(I256::try_from(n).unwrap(), 32);

    let mut group = c.benchmark_group(format!("odd_product_{}", n));
    group.sample_size(20);

    bench_solang(&mut group, "Computation", "odd_product", n);
    bench_evm(&mut group, "Computation", "odd_product", &[n_evm]);

    group.finish()
}

fn remainders(c: &mut Criterion) {
    let args_scale = &[sp_core::U256::from(1), sp_core::U256::from(2)];
    let args_evm = &[
        DynSolValue::Uint(U256::from(1), 256),
        DynSolValue::Uint(U256::from(2), 256),
    ];

    let mut group = c.benchmark_group("remainders");
    group.sample_size(20);

    bench_solang(&mut group, "Arithmetics", "remainders", args_scale);
    bench_evm(&mut group, "Arithmetics", "remainders", args_evm);

    group.finish()
}

criterion_group!(computation, odd_product, triangle_number);
criterion_group!(arithmetics, remainders);

criterion_main!(computation, arithmetics);
