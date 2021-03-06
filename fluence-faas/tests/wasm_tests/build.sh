#!/bin/sh

# This script builds all tests
cd arguments_passing
cargo update
fce build --release
rm artifacts/*

cd ../arrays_passing
cargo update
fce build --release
rm artifacts/*

cd ../inner_records
cargo update
fce build --release
rm artifacts/*

cd ..
cp ../../../target/wasm32-wasi/release/arguments_passing_effector.wasm arguments_passing/artifacts/
cp ../../../target/wasm32-wasi/release/arguments_passing_pure.wasm arguments_passing/artifacts/
cp ../../../target/wasm32-wasi/release/arrays_passing_effector.wasm arrays_passing/artifacts/
cp ../../../target/wasm32-wasi/release/arrays_passing_pure.wasm arrays_passing/artifacts/
cp ../../../target/wasm32-wasi/release/inner_records_pure.wasm inner_records/artifacts/
