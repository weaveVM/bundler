# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WeaveVM Bundler is a Rust library and data protocol for creating bundled EVM transactions on WeaveVM. It enables batching multiple transactions (envelopes) into single bundles to reduce fees and increase efficiency.

## Development Commands

### Build and Test
```bash
cargo build                    # Build the project
cargo test                     # Run all tests
cargo test test_name           # Run specific test
cargo run                      # Run the bundler server
```

### Key Tests
- `test_bundle_retrieval` - Tests retrieving bundle data from WeaveVM
- `test_send_bundle_*` - Various bundle sending scenarios
- `test_send_large_bundle_*` - Large bundle operations with/without SuperAccount
- `test_retrieve_large_bundle` - Large bundle reconstruction

## Architecture Overview

The project consists of several core modules organized under `src/utils/`:

### Core Components (`src/utils/core/`)
- **Bundle** (`bundle.rs`): Main bundle creation and propagation functionality for 0xbabe1 transactions
- **LargeBundle** (`large_bundle.rs`): Handles large bundles (0xbabe2) that exceed standard size limits
- **Envelope** (`envelope.rs`): Individual transaction wrapper with data, target, and optional tags
- **SuperAccount** (`super_account.rs`): Multi-wallet system for parallel large bundle chunk uploads
- **Tags** (`tags.rs`): Metadata system for envelopes (Content-Type, Protocol, etc.)

### Bundle Versions
- **0xbabe1** (`ADDRESS_BABE1`): Standard bundles up to 4MB
- **0xbabe2** (`ADDRESS_BABE2`): Large bundles up to 2GB (SDK limit) / 246GB (network limit)

### Data Flow
1. **Envelopes**: Legacy EVM transactions with data, optional target, and tags
2. **Bundles**: Collections of envelopes, Borsh-serialized and Brotli-compressed
3. **Large Bundles**: Split into 4MB chunks, sequentially connected, with final reference bundle

### Server API (`src/server/`)
HTTP API server with endpoints:
- `/v1/*` - 0xbabe1 bundle operations
- `/v2/*` - 0xbabe2 large bundle operations
- Envelope retrieval, ID extraction, and large bundle resolution

### Constants (`src/utils/constants.rs`)
Critical network and protocol parameters:
- WeaveVM Chain ID: 9496
- RPC URL: https://alphanet.load.network
- Bundle target addresses (0xbabe1, 0xbabe2)
- Size limits and chunk configurations

## Key Development Patterns

### Bundle Creation
```rust
let bundle = Bundle::new()
    .private_key(private_key)
    .envelopes(envelopes)
    .build()?
    .propagate()
    .await?;
```

### Large Bundle with SuperAccount
```rust
let large_bundle = LargeBundle::new()
    .data(data)
    .private_key(private_key)
    .super_account(super_account)
    .chunk()
    .build()?
    .super_propagate_chunks()
    .await?
    .finalize()
    .await?;
```

### Envelope Constraints
- `nonce`, `gas_limit`, `gas_price`, `value` must be 0
- Used strictly for data settling (no tWVM transfers or contract interactions)
- Tags size limit: 2048 bytes before compression

## Testing Requirements
- Tests require funded tWVM private key to execute successfully
- Example private key in tests is for reference only
- Load0 integration testing via `propagate_to_load0()` method