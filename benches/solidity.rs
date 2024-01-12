use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion,
};

use alloy_dyn_abi::DynSolValue;
use alloy_primitives::I256;
use parity_scale_codec::Encode;
use schlau::{evm::EvmContract, solang::SolangContract};
use std::time::Duration;

fn bench_evm(
    group: &mut BenchmarkGroup<WallTime>,
    contract: &str,
    message: &str,
    args: &[(Vec<DynSolValue>, String)],
) {
    let mut evm_contract = EvmContract::init(contract);

    for (args, parameter) in args {
        let args = evm_contract.call_args(message, args);
        let id = BenchmarkId::new("evm", parameter);

        group.bench_function(id, |b| {
            b.iter(|| {
                evm_contract.sandbox.call(args.clone()).unwrap();
            })
        });
    }
}

fn bench_solang<Args: Encode>(
    group: &mut BenchmarkGroup<WallTime>,
    contract: &str,
    message: &str,
    args: &[(Args, String)],
) {
    let mut solang_contract = SolangContract::init(contract);

    for (args, parameter) in args {
        let args = solang_contract.call_args(message, args);
        let id = BenchmarkId::new(&format!("solang({})", schlau::target_str()), parameter);

        group.bench_function(id, |b| {
            b.iter(|| {
                solang_contract.drink_api.call(args.clone()).unwrap();
            })
        });
    }
}

fn triangle_number(c: &mut Criterion) {
    let ns = [100_000i64, 200_000, 400_000, 800_000, 1_600_000].map(|n| (n, n.to_string()));
    let ns_evm = ns
        .clone()
        .map(|(n, display)| {
            (
                vec![DynSolValue::Int(I256::try_from(n).unwrap(), 64)],
                display,
            )
        })
        .to_vec();

    let mut group = c.benchmark_group("triangle_number");
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(25));

    bench_solang(&mut group, "Computation", "triangle_number", &ns);
    bench_evm(&mut group, "Computation", "triangle_number", &ns_evm);

    group.finish()
}

fn odd_product(c: &mut Criterion) {
    let ns = [100_000i32, 200_000, 400_000, 800_000, 1_600_000].map(|n| (n, n.to_string()));
    let ns_evm = ns
        .clone()
        .map(|(n, display)| {
            (
                vec![DynSolValue::Int(I256::try_from(n).unwrap(), 32)],
                display,
            )
        })
        .to_vec();

    let mut group = c.benchmark_group("odd_product");
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(25));

    bench_solang(&mut group, "Computation", "odd_product", &ns);
    bench_evm(&mut group, "Computation", "odd_product", &ns_evm);

    group.finish()
}

criterion_group!(benches, odd_product, triangle_number);
criterion_main!(benches);
