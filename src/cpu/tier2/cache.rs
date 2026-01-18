//! Decode cache implementation
//!
//! Caches decoded instructions indexed by their physical address (linear address from CS:IP).
//! This allows skipping the decode phase for frequently executed code, particularly in loops.

use std::collections::HashMap;

use crate::cpu::decode::instruction::DecodedInstruction;

/// Default maximum number of entries in the decode cache
/// When this limit is reached, the cache is cleared entirely
const DEFAULT_MAX_ENTRIES: usize = 8192;

/// A cached instruction entry with execution count tracking
#[derive(Clone)]
pub struct CacheEntry {
    /// The decoded instruction
    pub instruction: DecodedInstruction,
    /// Number of times this instruction has been executed from cache
    /// Used for tier 3 hot path detection
    pub hit_count: u32,
}

impl CacheEntry {
    /// Create a new cache entry for a decoded instruction
    pub fn new(instruction: DecodedInstruction) -> Self {
        Self {
            instruction,
            hit_count: 0,
        }
    }

    /// Increment hit count and return the new value
    #[inline(always)]
    pub fn record_hit(&mut self) -> u32 {
        self.hit_count = self.hit_count.saturating_add(1);
        self.hit_count
    }
}

/// Decode cache for tier 2 execution
///
/// Maps physical addresses (20-bit linear addresses) to decoded instructions.
/// When the cache reaches its maximum capacity, it is cleared entirely (simple eviction).
pub struct DecodeCache {
    /// Map from physical address to cached instruction
    entries: HashMap<u32, CacheEntry>,
    /// Maximum number of entries before clearing
    max_entries: usize,
    /// Total cache hits (for statistics)
    total_hits: u64,
    /// Total cache misses (for statistics)
    total_misses: u64,
}

impl DecodeCache {
    /// Create a new decode cache with default capacity
    pub fn new() -> Self {
        Self {
            entries: HashMap::with_capacity(DEFAULT_MAX_ENTRIES),
            max_entries: DEFAULT_MAX_ENTRIES,
            total_hits: 0,
            total_misses: 0,
        }
    }

    /// Create a new decode cache with custom capacity
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            max_entries,
            total_hits: 0,
            total_misses: 0,
        }
    }

    /// Look up a cached instruction by physical address
    ///
    /// Returns the cached entry if found, incrementing its hit count.
    /// Returns None if the address is not in the cache.
    #[inline(always)]
    pub fn get(&mut self, addr: u32) -> Option<&CacheEntry> {
        if let Some(entry) = self.entries.get_mut(&addr) {
            entry.record_hit();
            self.total_hits += 1;
            // Re-borrow as immutable to return
            self.entries.get(&addr)
        } else {
            self.total_misses += 1;
            None
        }
    }

    /// Insert a decoded instruction into the cache
    ///
    /// If the cache is at capacity, it is cleared before inserting.
    /// Returns true if the cache was cleared, false otherwise.
    #[inline(always)]
    pub fn insert(&mut self, addr: u32, instruction: DecodedInstruction) -> bool {
        let mut cleared = false;

        // Check if we need to clear the cache
        if self.entries.len() >= self.max_entries {
            self.entries.clear();
            cleared = true;
        }

        self.entries.insert(addr, CacheEntry::new(instruction));
        cleared
    }

    /// Invalidate a single cache entry by address
    ///
    /// Called when memory at a cached address is written to (self-modifying code).
    /// Returns true if an entry was invalidated, false if address wasn't cached.
    #[inline(always)]
    pub fn invalidate(&mut self, addr: u32) -> bool {
        self.entries.remove(&addr).is_some()
    }

    /// Invalidate a range of addresses
    ///
    /// Used when a multi-byte write could affect multiple cached instructions.
    /// Invalidates all entries in the range [start_addr, start_addr + len).
    pub fn invalidate_range(&mut self, start_addr: u32, len: u32) {
        for offset in 0..len {
            self.entries.remove(&(start_addr + offset));
        }
    }

    /// Check if an address is in the cache (without recording a hit)
    ///
    /// Used to determine if a memory write needs to invalidate the cache.
    #[inline(always)]
    pub fn contains(&self, addr: u32) -> bool {
        self.entries.contains_key(&addr)
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get the number of entries currently in the cache
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get total cache hits
    pub fn total_hits(&self) -> u64 {
        self.total_hits
    }

    /// Get total cache misses
    pub fn total_misses(&self) -> u64 {
        self.total_misses
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_hits + self.total_misses;
        if total == 0 {
            0.0
        } else {
            self.total_hits as f64 / total as f64
        }
    }

    /// Reset statistics counters
    pub fn reset_stats(&mut self) {
        self.total_hits = 0;
        self.total_misses = 0;
    }
}

impl Default for DecodeCache {
    fn default() -> Self {
        Self::new()
    }
}
