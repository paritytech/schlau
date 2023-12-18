#!/usr/bin/env bash

set -euo pipefail

INSTALL_DIR="${PWD}/llvm16.0/"
mkdir -p $INSTALL_DIR


# Clone Solangs LLVM and apply rv32e patch set
if [ -d "llvm-project" ]; then
	cd llvm-project
else
	git clone --depth 1 --branch solana-rustc/16.0-2023-06-05 https://github.com/solana-labs/llvm-project.git
	cd llvm-project
	patch -p1 < ../patches/llvm-D70401.patch
	patch -p1 < ../patches/compiler-rt.patch
fi


# Build LLVM, clang
mkdir -p build
cd build
cmake -G Ninja -DLLVM_ENABLE_ASSERTIONS=On \
	-DLLVM_ENABLE_TERMINFO=Off \
	-DLLVM_ENABLE_LIBXML2=Off \
	-DLLVM_ENABLE_ZLIB=Off \
	-DLLVM_ENABLE_PROJECTS='clang;lld' \
	-DLLVM_TARGETS_TO_BUILD='RISCV;WebAssembly' \
	-DLLVM_ENABLE_ZSTD=Off \
	-DCMAKE_BUILD_TYPE=MinSizeRel \
	-DCMAKE_INSTALL_PREFIX=$INSTALL_DIR \
	../llvm

ninja
ninja install


# Build compiler builtins
cd ../compiler-rt
mkdir -p build
cd build

CFLAGS="--target=riscv32 -march=rv32em -mabi=ilp32e -nostdlib -nodefaultlibs"
cmake -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=$INSTALL_DIR \
	-DCMAKE_CXX_FLAGS="$CFLAGS" \
	-DCMAKE_C_FLAGS="$CFLAGS" \
	-DCMAKE_ASM_FLAGS="$CFLAGS" \
	-DLLVM_TARGETS_TO_BUILD='RISCV' \
	-DCMAKE_ASM_COMPILER_TARGET="riscv32" \
	-DCMAKE_C_COMPILER_TARGET="riscv32" \
	-DCOMPILER_RT_BAREMETAL_BUILD=ON \
	-DCMAKE_AR=$INSTALL_DIR/bin/llvm-ar \
	-DCMAKE_ASM_COMPILER=$INSTALL_DIR/bin/clang \
	-DCMAKE_CXX_COMPILER=$INSTALL_DIR/bin/clang \
	-DCMAKE_C_COMPILER=$INSTALL_DIR/bin/clang \
	-DCMAKE_EXE_LINKER_FLAGS="-fuse-ld=lld" \
	-DCMAKE_NM=$INSTALL_DIR/bin/llvm-nm \
	-DCMAKE_RANLIB=$INSTALL_DIR/bin/llvm-ranlib \
	-DCOMPILER_RT_BUILD_BUILTINS=ON \
	-DCOMPILER_RT_BUILD_LIBFUZZER=OFF \
	-DCOMPILER_RT_BUILD_MEMPROF=OFF \
	-DCOMPILER_RT_BUILD_PROFILE=OFF \
	-DCOMPILER_RT_BUILD_SANITIZERS=OFF \
	-DCOMPILER_RT_BUILD_XRAY=OFF \
	-DCOMPILER_RT_DEFAULT_TARGET_ONLY=ON \
	-DLLVM_CONFIG_PATH=$INSTALL_DIR/bin/llvm-config \
	-DLLVM_DEFAULT_TARGET_TRIPLE="riscv32-unknown-unknown-elf" \
	..  

ninja
ninja install


# Build solang
cd ../../..
if [ ! -d "solang" ]; then
	git clone --depth 1 --branch risc-v "https://github.com/xermicus/solang.git"
fi
cd solang
PATH="${INSTALL_DIR}bin/:${PATH}" cargo build --release --no-default-features --features llvm,wasm_opt


echo ""
echo "success"
