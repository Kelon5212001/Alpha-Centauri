#!/bin/bash
set -e

echo "🚀 Setting up SMAC Reimplementation environment..."

# Update package lists
echo "📦 Updating package lists..."
sudo apt-get update -qq

# Install essential packages
echo "🛠️ Installing essential packages..."
sudo apt-get install -y \
    git curl wget unzip p7zip-full build-essential pkg-config libssl-dev nano vim tree htop jq \
    python3 python3-pip python3-venv cmake clang llvm-dev libclang-dev

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source ~/.cargo/env
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
fi
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.cargo/env 2>/dev/null || true

# Install Rust tools
echo "🔧 Installing Rust development tools..."
cargo install --force cargo-watch cargo-edit cargo-tree

# Set up Python environment
echo "🐍 Setting up Python environment..."
python3 -m pip install --upgrade pip
pip3 install requests json5 regex beautifulsoup4 lxml pillow pyyaml

# Make project structure
echo "📁 Creating project structure..."
cd ~/Projects/SMAC_Rust_AI
mkdir -p smac_reimplementation/{src/{data,game,extraction,utils,tests},assets/{sprites,sounds,data,maps},tools/{extraction,conversion,validation},docs/{api,design,progress},scripts,examples,tests/{unit,integration},target,.cargo}

# Clone GLSMAC if not present
if [ ! -d "glsmac" ]; then
    echo "📥 Cloning GLSMAC repository..."
    git clone https://github.com/afwbkbc/glsmac.git
    echo "✅ GLSMAC cloned successfully"
else
    echo "📂 GLSMAC already exists, updating..."
    cd glsmac && git pull && cd ..
fi

# Initialize Rust project
cd smac_reimplementation

if [ ! -f "Cargo.toml" ]; then
    echo "📝 Creating Cargo.toml..."
    cat > Cargo.toml << 'EOF'
[package]
name = "smac_reimplementation"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Open-source reimplementation of Sid Meier's Alpha Centauri"
license = "MIT OR Apache-2.0"
repository = "https://github.com/kelon5212001/Alpha-Centauri"

[[bin]]
name = "extract_glsmac"
path = "src/extraction/main.rs"

[[bin]]
name = "game"
path = "src/game/main.rs"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.0"
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
tokio = { version = "1.0", features = ["full"] }
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
log = "0.4"
env_logger = "0.10"

[dev-dependencies]
tempfile = "3.0"
assert_cmd = "2.0"
predicates = "3.0"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
EOF
fi

echo "🏗️ Creating source files..."

# Main extraction binary
mkdir -p src/extraction
cat > src/extraction/main.rs << 'EOF'
//! GLSMAC Data Extraction Tool

use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(name = "extract_glsmac")]
#[command(about = "Extract data from GLSMAC project")]
struct Cli {
    /// Path to GLSMAC repository
    glsmac_path: PathBuf,

    /// Output file for extracted data
    #[arg(short, long, default_value = "extracted_data.json")]
    output: PathBuf,

    /// Generate Rust modules instead of JSON
    #[arg(long)]
    rust_modules: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::init();
    }

    log::info!("Starting GLSMAC data extraction...");
    log::info!("GLSMAC path: {:?}", cli.glsmac_path);
    log::info!("Output: {:?}", cli.output);

    // TODO: Implement extraction logic
    println!("🎯 Extraction tool ready!");
    println!("📁 GLSMAC path: {:?}", cli.glsmac_path);
    println!("💾 Output: {:?}", cli.output);

    Ok(())
}
EOF

# Game main binary placeholder
mkdir -p src/game
cat > src/game/main.rs << 'EOF'
//! SMAC Reimplementation Game Engine

fn main() {
    println!("🎮 SMAC Reimplementation Game Engine");
    println!("🚧 Under development...");
}
EOF

# Library root
cat > src/lib.rs << 'EOF'
//! SMAC Reimplementation Library

pub mod data;
pub mod game;
pub mod extraction;
pub mod utils;

pub use data::*;
pub use game::*;
pub use extraction::*;
pub use utils::*;
EOF

# Create module files
touch src/{data,game,extraction,utils}/mod.rs

# .cargo/config.toml for better defaults
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[profile.dev]
debug = 1
split-debuginfo = "unpacked"
EOF

# Development scripts
mkdir -p scripts
cat > scripts/build.sh << 'EOF'
#!/bin/bash
echo "🔨 Building SMAC Reimplementation..."
cargo build --release
echo "✅ Build complete!"
EOF

cat > scripts/extract.sh << 'EOF'
#!/bin/bash
echo "📤 Extracting GLSMAC data..."
cargo run --bin extract_glsmac -- ../glsmac --output assets/data/glsmac_data.json "$@"
echo "✅ Extraction complete!"
EOF

cat > scripts/run_game.sh << 'EOF'
#!/bin/bash
echo "🎮 Starting SMAC Reimplementation..."
cargo run --bin game
EOF

chmod +x scripts/*.sh

cat > README.md << 'EOF'
# SMAC Reimplementation

An open-source reimplementation of Sid Meier's Alpha Centauri.

## Development Environment

This project is set up for development in Codex with:
- Rust toolchain with cargo
- Python 3 with useful libraries
- GLSMAC repository for data extraction

## Quick Start

```bash
# Build the project
./scripts/build.sh

# Extract data from GLSMAC
./scripts/extract.sh

# Run the game (when implemented)
./scripts/run_game.sh
EOF
