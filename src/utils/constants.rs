pub const CHAIN_ID: u64 = 9496;
pub const BLOCK_GAS_LIMIT: u128 = 500_000_000;
pub const GAS_PRIORITY_MULTIPLIER: f32 = 1.1; // 10%
pub const TAGS_SIZE_LIMIT: usize = 2048; // in bytes;
pub const WVM_RPC_URL: &str = "https://alphanet.load.network";
pub const LOAD0_ENDPOINT_URL: &str = "https://load0.network";
pub const ADDRESS_BABE1: &str = "0xbabe1d25501157043c7b4ea7CBC877B9B4D8A057";
pub const ADDRESS_BABE2: &str = "0xbabe2dCAf248F2F1214dF2a471D77bC849a2Ce84";
pub const LB_CHUNK_MAX_SIZE: u32 = 4_194_304; // 4MB
pub const ONE_MB_IN_BYTES: u32 = 1_048_576; // 1MB
pub const ONE_KILOBYTE_IN_BYTES: u32 = 1024;
pub const LB_THEORETICAL_SIZE_LIMIT: u64 =
    246 * ONE_KILOBYTE_IN_BYTES as u64 * ONE_MB_IN_BYTES as u64;
// 1 GB
pub const LB_SAFE_MAX_SIZE_LIMIT: u64 =
    (2 * ONE_KILOBYTE_IN_BYTES * ONE_KILOBYTE_IN_BYTES * ONE_KILOBYTE_IN_BYTES) as u64; //  2GB
pub const MAX_THEORETICAL_CHUNKS_IN_LB: u32 = 61680;
pub const MAX_SAFE_CHUNKS_IN_LB: u32 = 2 * 256; // 2GB
pub const SAFE_CHUNK_TOPUP: u64 = 1_000_000_000_000_000_000;
