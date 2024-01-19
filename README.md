# schlau :brain:

Benchmarking smart contract execution. German cousin of `smart-bench` :brain:

## Running Benchmarks

### Install Prerequisites

```bash
cargo install cargo-criterion
cargo install criterion-table
````

### Run Solidity Benchmarks

```bash
# run benchmarks
cargo criterion --features evm,wasm --bench solidity --message-format=json > solidity_wasm.json
cargo criterion --features riscv --bench solidity --message-format=json > solidity_riscv.json
# construct table
cat solidity_wasm.json solidity_riscv.json | criterion-table
```

### Run `ink!` Benchmarks

```bash
# run benchmarks
cargo criterion --features wasm --bench ink --message-format=json > ink_wasm.json
cargo criterion --features riscv --bench ink --message-format=json > ink_riscv.json
# construct table
cat ink_wasm.json ink_riscv.json | criterion-table
```