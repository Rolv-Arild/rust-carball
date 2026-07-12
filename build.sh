
ECHO "Building for Windows..."
cargo build --release

ECHO "Building for Linux (via WSL)..."
wsl --shell-type login cargo build --release --no-default-features
