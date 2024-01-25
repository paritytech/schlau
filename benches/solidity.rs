use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion,
};

use alloy_dyn_abi::DynSolValue;
use alloy_primitives::{I256, U256};
use parity_scale_codec::Encode;
use rand::distributions::Uniform;
use rand::Rng;
use schlau::{
    evm::{EvmContract, ACCOUNTS},
    solang::SolangContract,
};

fn bench_evm(
    group: &mut BenchmarkGroup<WallTime>,
    contract: &str,
    message: &str,
    args: &[(Vec<DynSolValue>, String)],
) {
    if cfg!(feature = "evm") {
        for (args, parameter) in args {
            let mut evm_contract = EvmContract::init(contract);

            let id = BenchmarkId::new("evm", parameter);
            let args = evm_contract.call_args(message, &args.clone());
            let mut account_index = 0;

            group.bench_function(id, |b| {
                b.iter(|| {
                    let mut args = args.clone();
                    // use a different account to avoid `BalanceLow`
                    args.source = ACCOUNTS[account_index];
                    account_index = (account_index + 1) % ACCOUNTS.len();

                    evm_contract.sandbox.call(args).unwrap();
                })
            });
        }
    }
}

fn bench_solang<Args: Encode>(
    group: &mut BenchmarkGroup<WallTime>,
    contract: &str,
    message: &str,
    args: &[(Args, String)],
) {
    for (args, parameter) in args {
        let mut solang_contract = SolangContract::init(contract);

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
    let ns = [3_000_000i64, 6_000_000, 12_000_000].map(|n| (n, n.to_string()));
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

    bench_evm(&mut group, "Computation", "triangle_number", &ns_evm);
    bench_solang(&mut group, "Computation", "triangle_number", &ns);

    group.finish()
}

fn odd_product(c: &mut Criterion) {
    let ns = [2_000_000i32, 4_000_000, 8_000_000].map(|n| (n, n.to_string()));
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

    bench_evm(&mut group, "Computation", "odd_product", &ns_evm);
    bench_solang(&mut group, "Computation", "odd_product", &ns);

    group.finish()
}

fn remainders(c: &mut Criterion) {
    let args_scale = [(
        [sp_core::U256::from(1), sp_core::U256::from(2)],
        "(1, 2)".to_owned(),
    )];
    let args_evm = [(
        vec![
            DynSolValue::Uint(U256::from(1), 256),
            DynSolValue::Uint(U256::from(2), 256),
        ],
        "(1, 2)".to_owned(),
    )];

    let mut group = c.benchmark_group("remainders");
    group.sample_size(20);

    bench_evm(&mut group, "Arithmetics", "remainders", &args_evm);
    bench_solang(&mut group, "Arithmetics", "remainders", &args_scale);

    group.finish()
}

fn baseline(c: &mut Criterion) {
    let args_scale = [(0, "(0)".to_owned())];
    let args_evm = [(vec![DynSolValue::Uint(U256::ZERO, 32)], "(1)".to_owned())];

    let mut group = c.benchmark_group("baseline");
    group.sample_size(20);

    bench_evm(&mut group, "compile_test", "test", &args_evm);
    bench_solang(&mut group, "compile_test", "test", &args_scale);

    group.finish()
}

fn fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci_iterative");
    group.sample_size(20);

    for n in [128u32, 192, 256, 320] {
        let args_scale = [(n, format!("{n}"))];
        let args_evm = [(vec![DynSolValue::Uint(U256::from(n), 32)], format!("{n}"))];

        bench_evm(&mut group, "FibonacciIterative", "fib", &args_evm);
        bench_solang(&mut group, "FibonacciIterative", "fib", &args_scale);
    }

    group.finish();

    let mut group = c.benchmark_group("fibonacci_binet");
    group.sample_size(20);

    for n in [128u32, 192, 256, 320] {
        let args_scale = [(n, format!("{n}"))];
        let args_evm = [(vec![DynSolValue::Uint(U256::from(n), 32)], format!("{n}"))];

        bench_evm(&mut group, "FibonacciBinet", "fib", &args_evm);
        bench_solang(&mut group, "FibonacciBinet", "fib", &args_scale);
    }

    group.finish();
}

fn ripemd160(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let range = Uniform::new(u8::MIN, u8::MAX);
    let pre: Vec<u8> = (0..8192).map(|_| rng.sample(&range)).collect();

    let args_scale = [(&pre, "random".to_owned())];
    let args_evm = [(vec![DynSolValue::Bytes(pre.to_vec())], "random".to_owned())];

    let mut group = c.benchmark_group("ripemd160");
    group.sample_size(20);

    bench_evm(&mut group, "Ripemd160", "rmd160", &args_evm);
    bench_solang(&mut group, "Ripemd160", "rmd160", &args_scale);

    group.finish()
}

criterion_group!(
    computation,
    baseline,
    odd_product,
    triangle_number,
    fibonacci,
    ripemd160
);
criterion_group!(arithmetics, remainders);

criterion_main!(computation, arithmetics);
