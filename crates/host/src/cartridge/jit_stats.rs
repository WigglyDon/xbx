// SPDX-License-Identifier: MIT
//! Aggregate counters from the retained core's public x-query-jit response.
//! Never retain its free-form response or expose a guest inspection interface.
use serde::Serialize;
use std::{
    io,
    time::{Duration, Instant},
};

#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct Counters {
    one_instruction_per_block: bool,
    generated_bytes: u64,
    capacity_bytes: u64,
    blocks: u64,
    block_flushes: u64,
    block_invalidations: u64,
    tlb_full_flushes: u64,
    tlb_partial_flushes: u64,
    tlb_elided_flushes: u64,
}
pub(crate) fn parse(text: &str) -> io::Result<Counters> {
    if text.len() > 65536 {
        return Err(io::Error::other("native aggregate response too large"));
    }
    let field = |prefix: &str| -> io::Result<&str> {
        let mut matches = text.lines().filter_map(|l| l.strip_prefix(prefix));
        let v = matches
            .next()
            .ok_or_else(|| io::Error::other("missing aggregate counter"))?
            .trim();
        if matches.next().is_some() {
            return Err(io::Error::other("duplicate aggregate counter"));
        }
        Ok(v)
    };
    let number = |prefix| -> io::Result<u64> {
        let v = field(prefix)?;
        if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
            return Err(io::Error::other("invalid aggregate counter"));
        }
        v.parse().map_err(io::Error::other)
    };
    let one_instruction_per_block = match field("one-insn-per-tb:")? {
        "on" => true,
        "off" => false,
        _ => return Err(io::Error::other("invalid block setting")),
    };
    let sizes = field("gen code size       ")?
        .split_once('/')
        .ok_or_else(|| io::Error::other("invalid aggregate sizes"))?;
    let size = |s: &str| -> io::Result<u64> {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(io::Error::other("invalid aggregate size"));
        }
        s.parse().map_err(io::Error::other)
    };
    let generated_bytes = size(sizes.0)?;
    let capacity_bytes = size(sizes.1)?;
    if generated_bytes > capacity_bytes {
        return Err(io::Error::other("invalid aggregate capacity"));
    }
    Ok(Counters {
        one_instruction_per_block,
        generated_bytes,
        capacity_bytes,
        blocks: number("TB count            ")?,
        block_flushes: number("TB flush count      ")?,
        block_invalidations: number("TB invalidate count ")?,
        tlb_full_flushes: number("TLB full flushes    ")?,
        tlb_partial_flushes: number("TLB partial flushes ")?,
        tlb_elided_flushes: number("TLB elided flushes  ")?,
    })
}

/// Three attempts only, owned by the existing session/control thread. A failed
/// optional query consumes its attempt and cannot fail the player's session.
pub(crate) struct Observation {
    next: Option<Instant>,
    attempted: u8,
}
impl Observation {
    pub(crate) fn new() -> Self {
        Self {
            next: None,
            attempted: 0,
        }
    }
    pub(crate) fn due(&mut self, now: Instant) -> Option<u8> {
        if self.attempted >= 3 || self.next.is_some_and(|next| now < next) {
            return None;
        }
        self.attempted += 1;
        self.next = Some(now + Duration::from_secs(15));
        Some(self.attempted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const RESPONSE: &str = "Accelerator settings:\none-insn-per-tb: off\n\nTranslation buffer state:\ngen code size       123/456\nTB count            7\nTB avg host size    42 bytes (expansion ratio: 2.0)\nStatistics:\nTB flush count      1\nTB invalidate count 2\nTLB full flushes    3\nTLB partial flushes 4\nTLB elided flushes  5\n";
    #[test]
    fn retains_only_named_aggregate_values() {
        let c = parse(&format!("{RESPONSE}unrelated private text ignored\n")).unwrap();
        assert_eq!(
            c,
            Counters {
                one_instruction_per_block: false,
                generated_bytes: 123,
                capacity_bytes: 456,
                blocks: 7,
                block_flushes: 1,
                block_invalidations: 2,
                tlb_full_flushes: 3,
                tlb_partial_flushes: 4,
                tlb_elided_flushes: 5
            }
        );
        assert!(!serde_json::to_string(&c).unwrap().contains("private"));
    }
    #[test]
    fn refuses_incomplete_duplicate_invalid_or_overlarge_responses() {
        for bad in [
            RESPONSE.replace("TB flush count      1\n", ""),
            format!("{RESPONSE}TB count            8\n"),
            RESPONSE.replace("123/456", "457/456"),
            RESPONSE.replace("count 2", "count -2"),
            RESPONSE.replace("count 2", "count 18446744073709551616"),
            RESPONSE.replace("off", "unknown"),
            "x".repeat(65537),
        ] {
            assert!(parse(&bad).is_err());
        }
    }
    #[test]
    fn attempts_are_finite_even_after_failures() {
        let mut o = Observation::new();
        let t = Instant::now();
        assert_eq!(o.due(t), Some(1));
        assert_eq!(o.due(t), None);
        assert_eq!(o.due(t + Duration::from_secs(15)), Some(2));
        assert_eq!(o.due(t + Duration::from_secs(30)), Some(3));
        assert_eq!(o.due(t + Duration::from_secs(300)), None);
    }
}
