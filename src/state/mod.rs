mod node_state;
pub use node_state::*;
mod utils;

#[cfg(debug_assertions)]
const SNAPSHOT_DIR: &str = "./memorydb/snapshots";
#[cfg(not(debug_assertions))]
const SNAPSHOT_DIR: &str = "/etc/memorydb/snapshots";

#[cfg(debug_assertions)]
const WAL_FILE: &str = "./memorydb/data.wal";
#[cfg(not(debug_assertions))]
const WAL_FILE: &str = "/etc/memorydb/data.wal";

const DATE_FMT: &str = "%Y-%m-%d-%H:%M:%S";
const SNAPSHOT_KEEP_AMOUNT: usize = 10;
const SNAPSHOT_WRITE_INTERVAL_SEC: u64 = 60;
