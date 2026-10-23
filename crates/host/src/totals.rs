//! Fixed typed startup totals, frozen with the observer. No pointer values.
use crate::{api_journal, observer, probe::map_symbol};
use std::{
    fs,
    io::{BufReader, Write},
    os::unix::{fs::OpenOptionsExt, net::UnixStream},
    path::Path,
};
const WORDS: usize = 8 + 366 * 4;

pub struct Totals {
    words: Vec<u32>,
}
impl Totals {
    pub fn parse(w: &[u32], base: &observer::Capture) -> Result<Self, Box<dyn std::error::Error>> {
        if w.len() != WORDS
            || base.header[1] != 2
            || w[0] != 0x58425431
            || w[1] != 1
            || w[2] != (WORDS * 4) as u32
            || w[3] != base.header[5]
            || w[4] != base.header[10]
            || w[5] != 0
            || w[6] != w[4]
            || w[7] > w[6]
        {
            return Err("invalid/incomplete startup totals schema/event/budget".into());
        }
        let mut entries = 0u64;
        let mut returns = 0u64;
        for (i, row) in w[8..].chunks_exact(4).enumerate() {
            let ordinal = i as u32 + 1;
            let kind = observer::kind(ordinal, 2);
            if row[1] > row[0]
                || row[2] > row[1]
                || (kind == 2 && (row[2] == 0) != (row[3] == 0))
                || (kind == 2 && row[3] != 0 && row[3] & 0x80000000 == 0)
                || (kind != 2 && row[3] != 0)
                || (!matches!(kind, 2 | 3) && row[2] != 0)
            {
                return Err("invalid typed startup total/result category".into());
            }
            let recent: Vec<_> = base.calls.iter().filter(|c| c.ordinal == ordinal).collect();
            let recent_returns = recent.iter().filter(|c| c.returned != 0).count();
            let signals = recent
                .iter()
                .filter(|c| {
                    c.returned != 0
                        && ((kind == 2 && c.value & 0x80000000 != 0) || (kind == 3 && c.value == 0))
                })
                .count();
            if recent.len() > row[0] as usize
                || recent_returns > row[1] as usize
                || signals > row[2] as usize
            {
                return Err("startup totals contradict the frozen recent window".into());
            }
            entries += row[0] as u64;
            returns += row[1] as u64;
        }
        if entries != w[6] as u64 || returns != w[7] as u64 {
            return Err("startup totals do not sum to declared counts".into());
        }
        Ok(Self { words: w.to_vec() })
    }
    fn row(&self, ordinal: usize) -> &[u32] {
        &self.words[8 + (ordinal - 1) * 4..8 + ordinal * 4]
    }
    pub fn receipt(&self) -> String {
        let mut out = format!(
            "schema=xbx-startup-totals-v1\nevent_order={}\ncall_cutoff={}\nentries={}\nobserved_returns={}\nomitted_APIs_not_covered=true\nfunction_return_is_not_async_completion=true\nnull_or_negative_status_is_not_cause_proof=true\n# ordinal kind entries returns signal_count first_negative_ntstatus\n",
            self.words[3], self.words[4], self.words[6], self.words[7]
        );
        for ordinal in 1..=366 {
            let r = self.row(ordinal);
            if r[0] != 0 {
                out.push_str(&format!(
                    "{ordinal} {} {} {} {} {:08x}\n",
                    observer::kind(ordinal as u32, 2),
                    r[0],
                    r[1],
                    r[2],
                    r[3]
                ));
            }
        }
        out
    }
    pub fn fixture(&self, base: &observer::Capture) -> Result<(), Box<dyn std::error::Error>> {
        // The two explicit controls occur before 1100 calls overwrite them.
        // A late observed return must still contribute to the whole-run total.
        if self.row(15) != [3, 3, 1, 0]
            || self.row(233) != [3, 3, 1, 0xc0000008]
            || self.row(159) != [2, 2, 0, 0]
            || self.row(255) != [3, 2, 0, 0]
            || self.row(258) != [2, 0, 0, 0]
            || base.calls.iter().any(|c| {
                c.returned != 0
                    && ((c.ordinal == 15 && c.value == 0)
                        || (c.ordinal == 233 && c.value == 0xc0000008))
            })
        {
            return Err("original startup totals/overwritten controls/late return failed".into());
        }
        println!(
            "Original guest early null/status controls survive ring overwrite; late return, frozen in-flight call and nonreturning exits agree with startup totals."
        );
        Ok(())
    }
}
pub fn load(path: &Path, base: &observer::Capture) -> Result<Totals, Box<dyn std::error::Error>> {
    if fs::metadata(path)?.len() != (WORDS * 4) as u64 {
        return Err("startup totals capture size differs".into());
    }
    let raw = fs::read(path)?;
    Totals::parse(
        &raw.chunks_exact(4)
            .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
            .collect::<Vec<_>>(),
        base,
    )
}
pub fn capture(
    socket: &mut UnixStream,
    reader: &mut BufReader<UnixStream>,
    map: &Path,
    root: &Path,
    base: &observer::Capture,
) -> Result<(), Box<dyn std::error::Error>> {
    let map = fs::read_to_string(map)?;
    let at = map_symbol(&map, "_XkStartupTotals")?;
    let mut words = Vec::with_capacity(WORDS);
    for start in (0..WORDS).step_by(128) {
        words.extend(api_journal::read(
            socket,
            reader,
            at + start as u64 * 4,
            (WORDS - start).min(128),
        )?);
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join("startup-totals.bin"))?;
    for w in &words {
        file.write_all(&w.to_le_bytes())?;
    }
    file.sync_all()?;
    if words[..8] != api_journal::read(socket, reader, at, 8)? {
        return Err("startup totals changed during stopped capture".into());
    }
    let totals = Totals::parse(&words, base)?;
    fs::write(root.join("startup-totals.txt"), totals.receipt())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn seed() -> (Vec<u32>, observer::Capture) {
        let mut h = [0; 32];
        h[1] = 2;
        h[5] = 100;
        h[10] = 10;
        let base = observer::Capture {
            header: h,
            labels: vec![],
            calls: vec![],
        };
        let mut w = vec![0; WORDS];
        w[..8].copy_from_slice(&[0x58425431, 1, (WORDS * 4) as u32, 100, 10, 0, 10, 9]);
        w[8 + 14 * 4..8 + 15 * 4].copy_from_slice(&[10, 9, 1, 0]);
        (w, base)
    }
    #[test]
    fn totals_refuse_cross_event_missing_counts_and_pointer_values() {
        let (w, b) = seed();
        assert!(Totals::parse(&w, &b).is_ok());
        for (i, value) in [
            (3, 101),
            (5, 1),
            (6, 9),
            (7, 10),
            (8 + 14 * 4 + 1, 11),
            (8 + 14 * 4 + 3, 0x80010000),
        ] {
            let mut bad = w.clone();
            bad[i] = value;
            assert!(Totals::parse(&bad, &b).is_err());
        }
        let mut bad = w.clone();
        bad[8 + 14 * 4 + 2] = 10;
        assert!(Totals::parse(&bad, &b).is_err());
    }
    #[test]
    fn recent_signals_cannot_exceed_whole_startup_signals() {
        let (mut w, mut b) = seed();
        b.calls.push(observer::Call {
            token: 1,
            entry: 1,
            returned: 2,
            thread: 1,
            ordinal: 15,
            kind: 3,
            value: 0,
            flags: 0,
        });
        w[8 + 14 * 4 + 2] = 0;
        assert!(Totals::parse(&w, &b).is_err());
        w[8 + 14 * 4 + 2] = 1;
        assert!(Totals::parse(&w, &b).is_ok());
        w[8 + 14 * 4 + 3] = 0xc0000008;
        assert!(Totals::parse(&w, &b).is_err());
        assert_eq!(observer::kind(24, 1), 0);
        assert_eq!(observer::kind(24, 2), 2);
    }
}
