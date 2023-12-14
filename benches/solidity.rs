use criterion::{criterion_group, criterion_main, Criterion};
use schlau::evm::EvmSandbox;

fn computation(c: &mut Criterion) {
    let contract = schlau::solc::build_contract("contracts/solidity/computation.sol").unwrap();

    let mut group = c.benchmark_group("computation");
    group.sample_size(30);

    let bench_name = format!("odd_product_{}", 100_000);

    // group.bench_function(bench_name, |b| {
    //     b.iter(|| {
    //         let mut product = 1;
    //         for i in 1..=100_000 {
    //             if i % 2 == 1 {
    //                 product *= i;
    //             }
    //         }
    //         product
    //     })
    // });

    group.finish()
}

criterion_group!(benches, computation);
criterion_main!(benches);
