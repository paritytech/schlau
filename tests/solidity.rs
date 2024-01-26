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
}
