echo "Compiling for Linux x86-64...";
cross build --release --target=x86_64-unknown-linux-gnu;
cp target/x86_64-unknown-linux-gnu/release/gadd target/gadd-linux-x86-64;

echo "Compiling for Linux x86-32...";
cross build --release --target=i686-unknown-linux-gnu;
cp target/i686-unknown-linux-gnu/release/gadd target/gadd-linux-x86-32;

echo "Compiling for Linux ARM...";
cross build --release --target=aarch64-unknown-linux-gnu;
cp target/aarch64-unknown-linux-gnu/release/gadd target/gadd-linux-arm;

echo "Compiling for Windows x86-64...";
cross build --release --target=x86_64-pc-windows-gnu;
cp target/x86_64-pc-windows-gnu/release/gadd target/gadd-windows-x86-64;

echo "Compiling for Windows x86-32...";
cross build --release --target=i686-pc-windows-gnu;
cp target/i686-pc-windows-gnu/release/gadd target/gadd-windows-x86-32;

echo "Compiling for MacOS x86-64...";
docker run --rm \
    --volume "${PWD}":/root/src \
    --workdir /root/src \
    joseluisq/rust-linux-darwin-builder:1.68.2 \
    sh -c "CC=o64-clang CXX=o64-clang++ cargo build --release --target x86_64-apple-darwin";
cp target/x86_64-apple-darwin/release/gadd target/gadd-macos-x86-64;

echo "Compiling for MacOS ARM...";
docker run --rm \
    --volume "${PWD}":/root/src \
    --workdir /root/src \
    joseluisq/rust-linux-darwin-builder:1.68.2 \
    sh -c "CC=oa64-clang CXX=oa64-clang++ cargo build --release --target aarch64-apple-darwin";
cp target/aarch64-apple-darwin/release/gadd target/gadd-macos-arm;
