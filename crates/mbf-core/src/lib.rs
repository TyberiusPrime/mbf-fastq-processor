//! Core data types and DNA utilities for mbf-fastq-processor
//!
//! This crate provides the fundamental data structures used throughout
//! the FastQ processing pipeline:
//! - FastQElement: Zero-copy or owned representation of FastQ components
//! - FastQRead: Complete read representation
//! - DNA utilities: Sequence manipulation, anchoring, tagging
//! - Position: Range tracking for zero-copy parsing

#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]

pub mod dna;
pub mod reads;

pub use reads::{FastQElement, FastQRead, Position, WrappedFastQRead, WrappedFastQReadMut};
pub use dna::{Anchor, Hits, HitRegion, TagValue};

// Core types used across multiple crates

/// Demultiplexing tag identifier (u64)
pub type Tag = u64;

/// Segment index wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SegmentIndex(pub usize);

impl SegmentIndex {
    #[must_use]
    pub fn get_index(&self) -> usize {
        self.0
    }
}
