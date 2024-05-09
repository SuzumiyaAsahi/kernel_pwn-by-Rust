#!/bin/bash
cargo build --target x86_64-unknown-linux-musl --release
cp ./target/x86_64-unknown-linux-musl/release/bob ../core/
cd ..
./parcel.sh
./run.sh
