//! Fixed public-kernel observer schema and bounded post-stop capture.
//! No game memory, pointer following, caller addresses or output-buffer reads.
use crate::probe::map_symbol;
use std::{
    fs,
    io::{BufReader, Write},
    os::unix::{fs::OpenOptionsExt, net::UnixStream},
    path::Path,
};

pub const WORDS: usize = 8736;
const LABELS: usize = 128;
const RING: usize = 1024;
const ROWS: usize = 32 + LABELS * 4;

#[derive(Debug, Clone)]
pub struct Call {
    pub token: u32,
    pub entry: u32,
    pub returned: u32,
    pub thread: u32,
    pub ordinal: u32,
    pub kind: u32,
    pub value: u32,
    pub flags: u32,
}
#[derive(Debug)]
pub struct Capture {
    pub header: [u32; 32],
    pub labels: Vec<[u32; 4]>,
    pub calls: Vec<Call>,
}

fn kind(ordinal: u32) -> u32 {
    match ordinal {
        184..=231 | 233..=238 | 254..=257 | 139 | 142 | 181 | 159 => 2,
        14 | 15 | 165..=169 | 177 => 3,
        17 | 108 | 258 | 284 | 285 | 289..=291 | 298 | 320 => 1,
        _ => 0,
    }
}
fn omitted(ordinal: u32) -> bool {
    matches!(
        ordinal,
        103 | 104
            | 119
            | 125
            | 126
            | 128
            | 129
            | 160
            | 161
            | 221
            | 234
            | 238
            | 277
            | 278
            | 294
            | 295
    )
}
fn invalid(message: &str) -> Box<dyn std::error::Error> {
    message.into()
}
impl Capture {
    pub fn parse(words: &[u32]) -> Result<Self, Box<dyn std::error::Error>> {
        if words.len() != WORDS {
            return Err(invalid("incomplete observer storage"));
        }
        let h: [u32; 32] = words[..32].try_into()?;
        if h[0] != 0x58424f31 || h[1] != 1 || h[2] != (WORDS * 4) as u32 {
            return Err(invalid("observer schema/size differs"));
        }
        if h[3] != 1 || h[4] != 1 {
            return Err(invalid(match h[4] {
                0 => "capture unavailable: no qualifying first event",
                2 => "capture unavailable: existing eight-event budget exhausted",
                _ => "capture unavailable: observer/order failure",
            }));
        }
        if h[6] != 0xc0000005
            || h[7] != 14
            || h[8] > 3
            || !(1..=8).contains(&h[11])
            || h[11] != h[21]
            || h[22] != 8
            || h[20] != 0
            || h[24] == 0
            || h[5] <= h[24]
            || h[12] != h[10].min(RING as u32)
            || h[13] != h[10].saturating_sub(RING as u32)
            || h[15] > LABELS as u32
            || h[9] > h[15]
            || (h[9] == 0 && h[23] == 0)
            || h[31] > h[26]
            || h[28] > 366
            || h[29] > 366
        {
            return Err(invalid("inconsistent observer event/cutoff/budget/health"));
        }
        let mut labels: Vec<[u32; 4]> = Vec::new();
        for (i, l) in words[32..ROWS].chunks_exact(4).enumerate() {
            if i < h[15] as usize {
                if l[0] != i as u32 + 1
                    || l[1] == 0
                    || l[1] > h[5]
                    || l[3] & !3 != 0
                    || (l[3] & 1 != 0 && l[2] == 0)
                    || (l[2] != 0 && (l[2] <= l[1] || l[2] >= h[5]))
                {
                    return Err(invalid("invalid lifecycle label ledger"));
                }
                labels.push(l.try_into()?);
            } else if l != [0; 4] {
                return Err(invalid("nonempty unused label row"));
            }
        }
        if h[9] != 0 && labels[h[9] as usize - 1][2] != 0 {
            return Err(invalid("fault thread label already retired"));
        }
        let mut calls = Vec::new();
        for (index, row) in words[ROWS..].chunks_exact(8).enumerate() {
            if row[0] == 0 {
                if row != [0; 8] {
                    return Err(invalid("partially initialized call row"));
                }
                continue;
            }
            let c = Call {
                token: row[0],
                entry: row[1],
                returned: row[2],
                thread: row[3],
                ordinal: row[4],
                kind: row[5],
                value: row[6],
                flags: row[7],
            };
            if c.token <= h[13]
                || ((c.token - 1) as usize % RING) != index
                || c.token > h[10]
                || c.entry == 0
                || c.entry >= h[5]
                || !(1..=366).contains(&c.ordinal)
                || c.ordinal == 232
                || omitted(c.ordinal)
                || c.kind != kind(c.ordinal)
                || c.thread > h[15]
                || c.flags & !7 != 0
                || (c.thread == 0 && h[23] == 0)
                || (c.returned != 0
                    && (c.returned <= c.entry || c.returned >= h[5] || c.flags & 3 != 0))
                || (c.returned == 0 && (c.value != 0 || c.flags & 4 != 0))
                || (c.kind != 2 && c.kind != 3 && c.value != 0)
                || (c.kind != 3 && c.flags & 4 != 0)
                || (c.kind == 3 && (c.value > 5 || (c.value == 0 && c.flags & 4 != 0)))
            {
                return Err(invalid("invalid typed call/token/completion"));
            }
            if c.thread != 0 {
                let l = labels[c.thread as usize - 1];
                if c.entry < l[1]
                    || (l[2] != 0
                        && (c.entry >= l[2]
                            || c.returned >= l[2]
                            || (c.returned == 0 && c.flags & 3 == 0)))
                {
                    return Err(invalid("call crosses thread lifetime"));
                }
            }
            calls.push(c);
        }
        calls.sort_unstable_by_key(|c| c.token);
        if calls.len() != h[12] as usize
            || calls
                .iter()
                .enumerate()
                .any(|(i, c)| c.token != h[13] + i as u32 + 1)
            || calls.windows(2).any(|pair| pair[0].entry >= pair[1].entry)
        {
            return Err(invalid("missing/duplicate/unordered observer call tokens"));
        }
        if h[19] as usize != labels.iter().filter(|row| row[2] != 0).count() {
            return Err(invalid("retirement count differs from lifetime ledger"));
        }
        let mut orders: Vec<_> = calls
            .iter()
            .flat_map(|c| [c.entry, c.returned])
            .filter(|x| *x != 0)
            .collect();
        orders.sort_unstable();
        if orders.windows(2).any(|p| p[0] == p[1]) {
            return Err(invalid("reused global call order"));
        }
        Ok(Self {
            header: h,
            labels,
            calls,
        })
    }
    pub fn receipt(&self) -> String {
        let h = &self.header;
        let mut out = format!(
            "schema=xbx-contract-observer-v1\nevent=first_observed_after_entry_not_necessarily_terminal\nevent_order={}\nexisting_observation_index={}\ncode=c0000005\norigin=14\naccess_kind={}\nthread_label={}\ncall_cutoff={}\nretained={}\noverwritten={}\nomitted={}\nlabels={}\nlabel_overflows={}\nunavailable_returns_total={}\nlate_completions={}\nretired_labels={}\npost_entries={}\npost_returns={}\npost_exits={}\npost_last_entry_ordinal={}\npost_last_return_ordinal={}\npost_unavailable={}\npost_pre_cutoff_returns={}\nfunction_return_is_not_async_work_completion=true\nrange_category_is_not_pointer_contract_proof=true\n",
            h[5],
            h[11],
            h[8],
            h[9],
            h[10],
            h[12],
            h[13],
            h[14],
            h[15],
            h[16],
            h[17],
            h[18],
            h[19],
            h[25],
            h[26],
            h[27],
            h[28],
            h[29],
            h[30],
            h[31]
        );
        out.push_str("labels_are_current_execution_not_async_origin=true\n# label birth_order retirement_order reuse_flags\n");
        for row in &self.labels {
            out.push_str(&format!("L {} {} {} {}\n", row[0], row[1], row[2], row[3]));
        }
        out.push_str("# token entry_order return_order thread ordinal kind result availability\n");
        for c in &self.calls {
            let availability = if c.flags & 3 != 0 {
                "unavailable"
            } else if c.returned == 0 {
                "pending_at_cutoff"
            } else {
                "returned"
            };
            let result = if c.returned == 0 {
                "none".into()
            } else {
                match c.kind {
                    2 => {
                        if c.value == 0x103 {
                            "ntstatus:00000103:STATUS_PENDING".into()
                        } else {
                            format!("ntstatus:{:08x}", c.value)
                        }
                    }
                    3 => format!(
                        "pointer:{}:aligned4={}",
                        [
                            "null",
                            "low",
                            "user_range",
                            "direct_range",
                            "system_range",
                            "other_range"
                        ][c.value as usize],
                        c.flags & 4 != 0
                    ),
                    1 => "void:none".into(),
                    _ => "unclassified:none".into(),
                }
            };
            out.push_str(&format!(
                "{} {} {} {} {} {} {} {}\n",
                c.token, c.entry, c.returned, c.thread, c.ordinal, c.kind, result, availability
            ));
        }
        out
    }
    pub fn fixture(&self) -> Result<(), Box<dyn std::error::Error>> {
        let h = &self.header;
        if h[9] != 1
            || h[8] != 0
            || h[11] != 1
            || h[13] == 0
            || h[18] == 0
            || h[19] < 2
            || h[15] < 3
            || h[14] == 0
            || h[16] != 0
            || h[23] != 0
            || h[30] != 0
            || h[31] == 0
            || h[25] < 1100
            || h[28] != 307
            || h[29] != 307
            || self.labels[1][2] == 0
            || self.labels[2][2] == 0
            || !self
                .calls
                .iter()
                .any(|c| c.ordinal == 255 && c.thread == 1 && c.returned == 0 && c.flags == 0)
            || !self
                .calls
                .iter()
                .any(|c| c.kind == 1 && c.returned != 0 && c.value == 0)
            || !self
                .calls
                .iter()
                .any(|c| c.kind == 3 && c.returned != 0 && c.value != 0)
            || !self.labels.iter().any(|row| row[3] & 1 != 0)
            || !self.labels.iter().any(|row| row[3] & 2 != 0)
        {
            return Err(invalid(
                "original observer guest did not prove the required event/window/lifetime/return sequence",
            ));
        }
        let release = self
            .calls
            .iter()
            .find(|call| call.ordinal == 145 && call.thread == 1)
            .ok_or("original guest release event missing")?;
        if self
            .calls
            .iter()
            .filter(|call| call.ordinal == 269)
            .any(|call| call.returned == 0 || call.returned >= release.entry)
        {
            return Err(invalid(
                "late completion rewrote a newer completed fixture call",
            ));
        }
        println!(
            "Original guest correlation, first read fault, lifetime retirement, ring reuse/late return, frozen window and post-capture completion passed. Native termination is separate."
        );
        Ok(())
    }
}
pub fn load(path: &Path) -> Result<Capture, Box<dyn std::error::Error>> {
    if fs::metadata(path)?.len() != (WORDS * 4) as u64 {
        return Err(invalid("observer file size differs from fixed storage"));
    }
    let bytes = fs::read(path)?;
    if bytes.len() != WORDS * 4 {
        return Err(invalid("incomplete observer byte capture"));
    }
    Capture::parse(
        &bytes
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect::<Vec<_>>(),
    )
}
pub fn capture(
    socket: &mut UnixStream,
    reader: &mut BufReader<UnixStream>,
    map: &Path,
    root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let map = fs::read_to_string(map)?;
    let base = map_symbol(&map, "_XkObserverStorage")?;
    crate::api_journal::kernel_range(base, (WORDS * 4) as u64)?;
    let mut raw = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join("observer-raw.bin"))?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut words = Vec::with_capacity(WORDS);
        for offset in (0..WORDS).step_by(128) {
            let block = crate::api_journal::read(
                socket,
                reader,
                base + (offset * 4) as u64,
                (WORDS - offset).min(128),
            )?;
            for word in &block {
                raw.write_all(&word.to_le_bytes())?;
            }
            words.extend(block);
        }
        // Stop acknowledgement is a caller precondition. A second header also
        // checks that post-capture bookkeeping health stayed stable while read.
        let header = crate::api_journal::read(socket, reader, base, 32)?;
        if words[..32] != header {
            return Err(invalid("observer changed during stopped capture"));
        }
        let parsed = Capture::parse(&words)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("observer.txt"))?;
        file.write_all(parsed.receipt().as_bytes())?;
        file.sync_all()?;
        Ok(())
    })();
    raw.sync_all()?; // partial reads are retained too
    if let Err(error) = &result {
        fs::write(root.join("observer-rejected.txt"), format!("{error}\n"))?;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn valid() -> Vec<u32> {
        let mut w = vec![0; WORDS];
        w[0] = 0x58424f31;
        w[1] = 1;
        w[2] = (WORDS * 4) as u32;
        w[3] = 1;
        w[4] = 1;
        w[5] = 10;
        w[6] = 0xc0000005;
        w[7] = 14;
        w[9] = 1;
        w[10] = 1;
        w[11] = 1;
        w[12] = 1;
        w[15] = 1;
        w[21] = 1;
        w[22] = 8;
        w[24] = 1;
        w[32..36].copy_from_slice(&[1, 2, 0, 0]);
        w[ROWS..ROWS + 8].copy_from_slice(&[1, 3, 4, 1, 184, 2, 0, 0]);
        w
    }
    #[test]
    fn pending_function_return_is_not_an_inflight_call_or_async_completion() {
        let mut w = valid();
        w[ROWS + 6] = 0x103;
        let returned = Capture::parse(&w).unwrap().receipt();
        assert!(returned.contains("STATUS_PENDING returned"));
        assert!(returned.contains("function_return_is_not_async_work_completion=true"));
        w[ROWS + 2] = 0;
        w[ROWS + 6] = 0;
        let inflight = Capture::parse(&w).unwrap().receipt();
        assert!(inflight.contains("none pending_at_cutoff"));
        w[ROWS + 7] = 1;
        assert!(
            Capture::parse(&w)
                .unwrap()
                .receipt()
                .contains("none unavailable")
        );
        w[ROWS + 2] = 4;
        assert!(Capture::parse(&w).is_err());
    }
    #[test]
    fn pointer_void_unknown_and_late_postcutoff_results_cannot_leak_as_statuses() {
        let mut w = valid();
        w[ROWS + 4] = 15;
        w[ROWS + 5] = 3;
        w[ROWS + 6] = 4;
        w[ROWS + 7] = 4;
        assert!(
            Capture::parse(&w)
                .unwrap()
                .receipt()
                .contains("pointer:system_range:aligned4=true")
        );
        w[ROWS + 6] = 0xd0001234;
        assert!(Capture::parse(&w).is_err());
        for (ordinal, kind) in [(285, 1), (307, 0)] {
            let mut w = valid();
            w[ROWS + 4] = ordinal;
            w[ROWS + 5] = kind;
            w[ROWS + 6] = 0xc0000005;
            assert!(Capture::parse(&w).is_err());
        }
        let mut w = valid();
        w[ROWS + 2] = 11;
        assert!(Capture::parse(&w).is_err());
        let mut w = valid();
        w[ROWS] = 2;
        assert!(Capture::parse(&w).is_err());
    }
    #[test]
    fn retired_lifetimes_cannot_be_joined_or_reused_and_unavailable_capture_refuses() {
        let mut w = valid();
        w[15] = 2;
        w[9] = 2;
        w[34] = 6;
        w[19] = 1;
        w[36..40].copy_from_slice(&[2, 7, 0, 0]);
        assert!(Capture::parse(&w).is_ok());
        w[ROWS + 3] = 2;
        assert!(Capture::parse(&w).is_err()); // entry predates new lifetime
        w[ROWS + 3] = 1;
        w[36] = 1;
        assert!(Capture::parse(&w).is_err()); // recycled label
        let mut w = valid();
        w[4] = 2;
        assert!(
            Capture::parse(&w)
                .unwrap_err()
                .to_string()
                .contains("budget exhausted")
        );
        let mut w = valid();
        w.pop();
        assert!(Capture::parse(&w).is_err());
        let mut w = valid();
        w[ROWS + 4] = 232;
        assert!(Capture::parse(&w).is_err());
    }
}
