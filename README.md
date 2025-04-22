# Chicken Door
This repo contains components for an automatic chicken coop door, including:
- A daemon/web server for automation and user interface
- An (outdated) simulation of the simple motor controller
    - Can be imported to [falstad](https://www.falstad.com/circuit/circuitjs.html)
- Alpine Linux configuration
## Compiling the server
Compiling requires `cargo-leptos` and optionally `cross` for cross compilation (recommended).

1. Set `bin-target-triple` in `Cargo.toml` to the correct target for your control board
	- The current target is for a Raspberry Pi 3 running Alpine
2. `LEPTOS_BIN_CARGO_COMMAND=cross cargo leptos build --release`
## Deploying the server
The following items must be copied to your target:
1. The server binary: `target/<your target>/release/chicken-door`
2. The site directory: `target/<your target>/release/chicken-door`
3. This repo's Cargo.toml

Example:
```bash
TARGET="aarch64-unknown-linux-musl"
DEPLOY_DIR="/root"

scp target/$TARGET/release/chicken-door root@chickendoor:$DEPLOY_DIR
scp -r target/site/ root@chickendoor:$DEPLOY_DIR/target/site
scp Cargo.toml root@chickendoor:$DEPLOY_DIR
```

## Running
Run the binary on the target device. The web ui will be available at the printed address.
