#[cfg(test)]
mod tests {
    #[cfg(feature = "evm")]
    use alloy_dyn_abi::DynSolValue;
    #[cfg(feature = "evm")]
    use alloy_primitives::U256;
    #[cfg(feature = "evm")]
    use schlau::evm::EvmContract;

    #[cfg(feature = "evm")]
    fn test_evm(contract: &str, message: &str, args: &[DynSolValue], returndata: Vec<u8>) {
        let mut evm_contract = EvmContract::init(contract);
        let args = evm_contract.call_args(message, &args);
        assert_eq!(returndata, evm_contract.sandbox.call(args).unwrap());
    }

    #[cfg(feature = "evm")]
    #[test]
    fn remainders_evm() {
        let args = [
            DynSolValue::Uint(U256::from(1), 256),
            DynSolValue::Uint(U256::from(2), 256),
        ];
        let returndata= hex::decode("2f1d314463072898fe68dcbfeadc1fa2ed55a9fa6fdfd6f987874cc75be143291599c327374526a85df07b0fe9645bc6a121678cb142390d7a26c1564a264af8").unwrap();

        test_evm("Arithmetics", "remainders", &args, returndata);
    }
}
