set -e

PLATFORM=${1:-"x86_64-linux-gnu-ubuntu-16.04"}

wget -qO- https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.0/clang+llvm-13.0.0-$PLATFORM.tar.xz | tar xJ

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

source "$HOME/.cargo/env"

curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

cd website_playground

CARGO_CFG_HYDROFLOW_GENERATE_DOCS="1" RUSTFLAGS="--cfg procmacro2_semver_exempt" CC="$PWD/../clang+llvm-13.0.0-$PLATFORM/bin/clang" wasm-pack build

cd ../docs

npm ci

LOAD_PLAYGROUND=1 npm run build
