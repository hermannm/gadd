#!/usr/bin/env bash

# Fails script on first error
set -e;

crosscompile() {
    target=$1
    output=$2
    suffix=$3

    echo "Compiling ${target}..."

    if [[ ${target} == *"apple-darwin"* ]]; then
        if [[ ${target} == *"x86_64"* ]]; then
            ccs="CC=o64-clang CXX=o64-clang++"
        elif [[ ${target} == *"aarch64"* ]]; then
            ccs="CC=oa64-clang CXX=oa64-clang++"
        fi

        docker run -t --rm \
            --volume "${PWD}":/root/src \
            --workdir /root/src \
            joseluisq/rust-linux-darwin-builder:1.76.0 \
            sh -c "CARGO_BUILD_TARGET='${target}' CARGO_TARGET_DIR='target/build/${target}' ${ccs} cargo build --release --target=${target}";
    else
        # Env variables to mitigate glibc version errors (https://github.com/cross-rs/cross/wiki/FAQ#glibc-version-error)
        CARGO_BUILD_TARGET="${target}" CARGO_TARGET_DIR="target/build/${target}" \
            cross build --release --target=${target}
    fi

    ret=$?
    if [ "${ret}" = 0 ]; then
        cp target/build/${target}/${target}/release/gadd${suffix} target/${output}${suffix}
    fi
    return "${ret}"
}

crosscompile x86_64-unknown-linux-gnu   gadd-linux-x86-64
crosscompile i686-unknown-linux-gnu     gadd-linux-x86-32
crosscompile aarch64-unknown-linux-gnu  gadd-linux-arm64
crosscompile x86_64-pc-windows-gnu      gadd-windows-x86-64 .exe
crosscompile i686-pc-windows-gnu        gadd-windows-x86-32 .exe
crosscompile x86_64-apple-darwin        gadd-macos-x86-64
crosscompile aarch64-apple-darwin       gadd-macos-arm64
