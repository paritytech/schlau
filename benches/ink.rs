use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ink::env::DefaultEnvironment;
use subxt_signer::sr25519::dev;

use schlau::{drink::runtime::MinimalRuntime, drink_api::CallArgs, ink::InkDrink};

macro_rules! ink_contract_bench {
    ( $name:ident, $contract:ident, $contract_ref:ident, $message:ident, $args:tt) => {
        fn $message(c: &mut Criterion) {
            use $name::$name::{$contract, $contract_ref};

            let contract_name = stringify!($name);

            let mut ink_drink = InkDrink::<DefaultEnvironment, MinimalRuntime>::new();
            let contract = ink_drink.build_and_instantiate::<_, $contract, _, _>(
                &format!("contracts/ink/{}/Cargo.toml", contract_name),
                &mut $contract_ref::new(),
            );

            let mut group = c.benchmark_group(stringify!($message));
            group.sample_size(30);
            group.measurement_time(std::time::Duration::from_secs(23));

            for args in $args {
                let message = contract.$message(args);
                let call_args = CallArgs::from_call_builder(dev::alice(), &message);

                let parameter = args.to_string();
                let id = BenchmarkId::new(&format!("ink({})", schlau::target_str()), parameter);

                group.bench_function(id, |b| {
                    b.iter(|| ink_drink.drink.call(call_args.clone()).unwrap())
                });
            }

            group.finish()
        }
    };
}

ink_contract_bench!(crypto, Crypto, CryptoRef, sha3, [100, 200, 400, 800]);
ink_contract_bench!(
    computation,
    Computation,
    ComputationRef,
    odd_product,
    [100_000, 200_000, 400_000, 800_000]
);
ink_contract_bench!(
    computation,
    Computation,
    ComputationRef,
    triangle_number,
    [100_000, 200_000, 400_000, 800_000]
);

criterion_group!(benches, sha3, odd_product, triangle_number);
criterion_main!(benches);
