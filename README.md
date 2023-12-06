# schlau :brain:

Benchmarking smart contract execution. German cousin of `smart-bench` :brain:

Running (RISC-V and WASM):

```bash
cargo bench --features riscv
cargo bench --features wasm
```

:warning: **Note**: The RISC-V backend is currently not working in native, so the numbers
are not currently representative. :warning: