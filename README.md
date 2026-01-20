# Shoko

Shoko is a high-performance, append-only archive format and library written in Rust. It utilizes a custom Run-Length Encoding (RLE) engine and a trailing-index architecture designed for fast writes and space-efficient storage of repetitive data.

# Features

* Custom RLE Engine: Fine-tuned compression thresholds (Levels 1-9).

* Append-Only Writes: Rapidly add or update files without rewriting the entire archive.

* Space Recovery: Built-in defrag logic to reclaim orphaned space from overwritten or deleted files.

* Glob Support: Pattern-based searching and selective extraction.

* Zero-Dependency Header: The core format is designed for easy parsing and stability (only ``glob`` as a dependency, thats all.)

# Installation

Shoko is split into a core library (shoko) and a command-line utility (shokoutils).

# Clone the repository
```
git clone https://github.com/cyntheria/shoko
cd shoko
```

# Build the workspace

```
cargo build --release
```

Usage (CLI)

The ``sar`` utility (**s**hoko **ar**chiver) provides the primary interface:

Packing

```
sar pack ./my_assets -o assets.sk1
```

Reading (Tree View)

```
sar read assets.sk1
```

Selective Unpacking

```
sar unpack assets.sk1 ./output --glob='images/*.png'
```

Live Editing

Edit a file directly inside the archive using your $EDITOR:

```
sar write assets.sk1/config.json
```

Library Integration

Add Shoko to your Cargo.toml:

```toml
[dependencies]
shoko = "0.1.1-rc1"
```

```rust
use shoko::archive::ShokoArchive;

fn main() -> std::io::Result<()> {
    let mut archive = ShokoArchive::open("data.sk1")?;
    let bytes = archive.extract_file("notes.txt")?;
    println!("Content: {}", String::from_utf8_lossy(&bytes));
    Ok(())
}
```

# License

This project is licensed under the GNU Lesser General Public License v3.0 (LGPL-3.0).
