/// CRDT (Conflict-free Replicated Data Types) implementations
/// for distributed synchronization without coordination

pub mod hlc;
pub mod lww_register;
pub mod or_set;

pub use hlc::{HybridLogicalClock, HLCTimestamp};
pub use lww_register::LWWRegister;
pub use or_set::{ORSet, ORSetEntry};
