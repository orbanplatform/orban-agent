// Build script for Protocol Buffers compilation
use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["proto/orban.proto"], &["proto/"])?;
    Ok(())
}
