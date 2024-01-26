//! Assert that all benchmark implementations perform the expected computations
//! on all runtimes.

#[cfg(test)]
mod tests {
    #[cfg(feature = "evm")]
    use alloy_dyn_abi::DynSolValue;
    #[cfg(any(feature = "wasm", feature = "riscv"))]
    use parity_scale_codec::Encode;

    #[cfg(feature = "evm")]
    fn test_evm(contract: &str, message: &str, args: &[DynSolValue], returndata: Vec<u8>) {
        let mut contract = schlau::evm::EvmContract::init(contract);
        let args = contract.call_args(message, &args);
        assert_eq!(returndata, contract.sandbox.call(args).unwrap());
    }

    #[cfg(any(feature = "wasm", feature = "riscv"))]
    fn test_solang<Args: Encode>(contract: &str, message: &str, args: &Args, returns: Vec<u8>) {
        let mut contract = schlau::solang::SolangContract::init(contract);
        let args = contract.call_args(message, args);
        assert_eq!(returns, contract.drink_api.call(args).unwrap());
    }

    #[cfg(any(feature = "wasm", feature = "riscv"))]
    macro_rules! test_ink {
        ( $name:ident, $contract:ident, $contract_ref:ident, $message:ident, $args:ident, $returns:ident) => {
            use ink::env::DefaultEnvironment;
            use schlau::{drink::runtime::MinimalRuntime, drink_api::CallArgs, ink::InkDrink};
            use subxt_signer::sr25519::dev;
            use $name::$name::{$contract, $contract_ref};

            let contract_name = stringify!($name);

            let mut ink_drink = InkDrink::<DefaultEnvironment, MinimalRuntime>::new();
            let contract = ink_drink.build_and_instantiate::<_, $contract, _, _>(
                &format!("contracts/ink/{}/Cargo.toml", contract_name),
                &mut $contract_ref::new(),
            );

            let message = contract.$message($args);
            let call_args =
                CallArgs::from_call_builder(dev::alice(), &message).with_max_gas_limit();

            assert_eq!($returns, ink_drink.drink.call(call_args.clone()).unwrap());
        };
    }

    #[cfg(any(feature = "wasm", feature = "riscv"))]
    #[test]
    fn crypto_ink_sha3() {
        let param = 1;
        let expected = Result::<u32, ()>::encode(&Ok(1));
        test_ink!(crypto, Crypto, CryptoRef, sha3, param, expected);
    }

    #[test]
    fn odd_product() {
        let (contract, message) = ("Computation", "odd_product");
        let param = 2_000_000i32;
        let expected = -1_335_316_246_127_320_831_i64;

        #[cfg(feature = "evm")]
        {
            let args = [DynSolValue::Int(
                alloy_primitives::I256::try_from(param).unwrap(),
                32,
            )];
            let returndata = alloy_primitives::I256::try_from(expected)
                .unwrap()
                .to_be_bytes::<32>()
                .to_vec();

            test_evm(contract, message, &args, returndata);
        }

        #[cfg(any(feature = "wasm", feature = "riscv"))]
        {
            test_solang(contract, message, &param, expected.encode());

            let expected = Result::<i64, ()>::encode(&Ok(expected));
            test_ink!(
                computation,
                Computation,
                ComputationRef,
                odd_product,
                param,
                expected
            );
        }
    }

    #[test]
    fn triangle_number() {
        let (contract, message) = ("Computation", "triangle_number");
        let param = 3_000_000i64;
        let expected = 4_500_001_500_000i64;

        #[cfg(feature = "evm")]
        {
            let args = [DynSolValue::Int(
                alloy_primitives::I256::try_from(param).unwrap(),
                64,
            )];
            let returndata = alloy_primitives::I256::try_from(expected)
                .unwrap()
                .to_be_bytes::<32>()
                .to_vec();

            test_evm(contract, message, &args, returndata);
        }

        #[cfg(any(feature = "wasm", feature = "riscv"))]
        {
            test_solang(contract, message, &param, expected.encode());

            let expected = Result::<i64, ()>::encode(&Ok(expected));
            test_ink!(
                computation,
                Computation,
                ComputationRef,
                triangle_number,
                param,
                expected
            );
        }
    }

    #[test]
    fn remainders() {
        let (contract, message) = ("Arithmetics", "remainders");
        let (param_a, param_b) = (1u32, 2u32);
        let (expected_a, expected_b) = (
            hex::decode("2f1d314463072898fe68dcbfeadc1fa2ed55a9fa6fdfd6f987874cc75be14329")
                .unwrap(),
            hex::decode("1599c327374526a85df07b0fe9645bc6a121678cb142390d7a26c1564a264af8")
                .unwrap(),
        );

        #[cfg(feature = "evm")]
        {
            let args = [
                DynSolValue::Uint(alloy_primitives::U256::from(param_a), 256),
                DynSolValue::Uint(alloy_primitives::U256::from(param_b), 256),
            ];
            let mut returndata = expected_a.clone();
            returndata.append(&mut expected_b.clone());

            test_evm(contract, message, &args, returndata);
        }

        #[cfg(any(feature = "wasm", feature = "riscv"))]
        {
            let args = [sp_core::U256::from(param_a), sp_core::U256::from(param_b)];
            let returns = parity_scale_codec::Encode::encode(&(
                sp_core::U256::from_big_endian(&expected_a),
                sp_core::U256::from_big_endian(&expected_b),
            ));
            test_solang(contract, message, &args, returns);
        }
    }

    #[test]
    fn fibonacci() {
        let param = 320u32;
        let expected =
            hex::decode("000000001febdb7ecc117ac2f78666ef94dfa339b50b38ee029a6bfd0402b645")
                .unwrap();

        #[cfg(feature = "evm")]
        {
            let args = [DynSolValue::Uint(alloy_primitives::U256::from(param), 32)];

            test_evm("FibonacciIterative", "fib", &args, expected.clone());
            test_evm("FibonacciBinet", "fib", &args, expected.clone());
        }

        #[cfg(any(feature = "wasm", feature = "riscv"))]
        {
            let expected = sp_core::U256::from_big_endian(&expected);
            test_solang("FibonacciIterative", "fib", &param, expected.encode());
            test_solang("FibonacciBinet", "fib", &param, expected.encode());
        }
    }
}
