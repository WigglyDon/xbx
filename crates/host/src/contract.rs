//! Scalar public-kernel context/dispatch facts; no game memory or raw addresses.
use crate::{
    api_journal,
    observer::{self, Capture},
    probe::map_symbol,
};
use std::{
    fs,
    io::{BufReader, Write},
    os::unix::{fs::OpenOptionsExt, net::UnixStream},
    path::Path,
};

pub struct Context {
    pub words: [u32; 64],
}
impl Context {
    pub fn parse(w: &[u32], base: &Capture) -> Result<Self, Box<dyn std::error::Error>> {
        if w.len() != 64
            || w[0] != 0x58425331
            || w[1] != 1
            || w[2] != 1
            || w[3] != base.header[5]
            || w[4] != base.header[9]
            || w[5] & !127 != 0
            || w[6] > 4096
            || w[7] > 0x400000
            || w[8] > 31
            || w[9] > 255
            || w[10] > 8
            || w[31] > 1
            || w[32] > 4
            || w[39] > 3
            || w[40] > w[41]
            || w[41] > 32768
            || w[42..].iter().any(|v| *v != 0)
            || (w[5] & 6 != 0 && w[5] & 1 == 0)
            || (w[5] & 2 == 0 && w[7] != 0)
            || w[34..38].iter().try_fold(0u32, |a, b| a.checked_add(*b)) != Some(w[33])
        {
            return Err("invalid public context schema/event/categories".into());
        }
        for i in 0..8 {
            let token = w[11 + i * 2];
            let ordinal = w[12 + i * 2];
            if i >= w[10] as usize {
                if token != 0 || ordinal != 0 {
                    return Err("nonempty unused active-call slot".into());
                }
                continue;
            }
            if token == 0 || token > base.header[10] || !(1..=366).contains(&ordinal) {
                return Err("invalid active public call".into());
            }
            if let Some(c) = base.calls.iter().find(|c| c.token == token)
                && (c.ordinal != ordinal || c.thread != w[4] || c.returned != 0)
            {
                return Err("active call contradicts frozen journal".into());
            }
        }
        for (token, flags, ordinal) in [(w[27], w[28], 291), (w[29], w[30], 217)] {
            if token > base.header[10]
                || flags & !15 != 0
                || flags & 7 > 5
                || (flags & 7 == 0 && flags & 8 != 0)
                || (token == 0 && flags != 0)
            {
                return Err("invalid public argument-category facts".into());
            }
            if let Some(c) = base.calls.iter().find(|c| c.token == token)
                && (c.ordinal != ordinal || c.thread != w[4])
            {
                return Err("argument category has wrong call association".into());
            }
        }
        Ok(Self {
            words: w.try_into()?,
        })
    }
    pub fn receipt(&self) -> String {
        let w = &self.words;
        let mut out = format!(
            "schema=xbx-public-context-v1\nevent_order={}\nthread_label={}\nthread_flags={}\nstack_pages={}\ntls_dwords={}\nirql={}\nnpx_state={}\nactive_call_depth={}\ncritical_token={}\ncritical_argument_category_flags={}\nquery_vm_token={}\nquery_vm_argument_category_flags={}\nmm_global_data_imported={}\ndispatch_outcome={}\nhandlers={}\ncontinue_search={}\ncontinue_execution={}\nnested={}\nother_disposition={}\ninvalid_chain={}\nhead_category={}\nphysical_pages_available={}\nphysical_pages_total={}\n",
            w[3],
            w[4],
            w[5],
            w[6],
            w[7],
            w[8],
            w[9],
            w[10],
            w[27],
            w[28],
            w[29],
            w[30],
            w[31],
            w[32],
            w[33],
            w[34],
            w[35],
            w[36],
            w[37],
            w[38],
            w[39],
            w[40],
            w[41]
        );
        for i in 0..w[10] as usize {
            out.push_str(&format!("active {} {}\n", w[11 + i * 2], w[12 + i * 2]));
        }
        out
    }
    pub fn fixture(&self) -> Result<(), Box<dyn std::error::Error>> {
        let w = &self.words;
        if w[5] != 120
            || w[7] != 0
            || w[32] != 2
            || w[33] != 1
            || w[35] != 1
            || w[34] != 0
            || w[38] != 0
            || w[39] != 2
            || w[10] != 1
            || w[12] != 255
        {
            return Err("original guest public context/handled-dispatch result failed".into());
        }
        println!(
            "Original guest public thread bounds/self flags, active create call and handled dispatch outcome verified."
        );
        Ok(())
    }
}
pub fn load(path: &Path, base: &Capture) -> Result<Context, Box<dyn std::error::Error>> {
    if fs::metadata(path)?.len() != 256 {
        return Err("context capture size differs".into());
    }
    let b = fs::read(path)?;
    Context::parse(
        &b.chunks_exact(4)
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
) -> Result<(), Box<dyn std::error::Error>> {
    observer::capture(socket, reader, map, root)?;
    let base = observer::load(&root.join("observer-raw.bin"))?;
    let text = fs::read_to_string(map)?;
    let at = map_symbol(&text, "_XkContractState")?;
    let words = api_journal::read(socket, reader, at, 64)?;
    let mut raw = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join("context-raw.bin"))?;
    for w in &words {
        raw.write_all(&w.to_le_bytes())?;
    }
    raw.sync_all()?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        if words != api_journal::read(socket, reader, at, 64)? {
            return Err("context changed during stopped capture".into());
        }
        let context = Context::parse(&words, &base)?;
        fs::write(root.join("context.txt"), context.receipt())?;
        if base.header[1] == 2 {
            crate::totals::capture(socket, reader, map, root, &base)?;
        }
        Ok(())
    })();
    if let Err(e) = &result {
        fs::write(root.join("context-rejected.txt"), format!("{e}\n"))?;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn inputs() -> ([u32; 64], Capture) {
        let mut h = [0; 32];
        h[5] = 10;
        h[9] = 1;
        h[10] = 7;
        let base = Capture {
            header: h,
            labels: vec![],
            calls: vec![crate::observer::Call {
                token: 7,
                entry: 9,
                returned: 0,
                thread: 1,
                ordinal: 255,
                kind: 2,
                value: 0,
                flags: 0,
            }],
        };
        let mut w = [0; 64];
        w[0] = 0x58425331;
        w[1] = 1;
        w[2] = 1;
        w[3] = 10;
        w[4] = 1;
        w[5] = 120;
        w[6] = 4;
        w[10] = 1;
        w[11] = 7;
        w[12] = 255;
        w[32] = 2;
        w[33] = 1;
        w[35] = 1;
        w[39] = 2;
        w[40] = 100;
        w[41] = 16384;
        (w, base)
    }
    #[test]
    fn accepts_public_handled_context_but_refuses_cross_event_or_raw_pointer_fields() {
        let (mut w, base) = inputs();
        Context::parse(&w, &base).unwrap().fixture().unwrap();
        w[4] = 2;
        assert!(Context::parse(&w, &base).is_err());
        w[4] = 1;
        w[28] = 0xd0001000;
        assert!(Context::parse(&w, &base).is_err());
        w[28] = 0;
        w[42] = 0x80010000;
        assert!(Context::parse(&w, &base).is_err());
    }
    #[test]
    fn active_call_and_dispatch_outcome_are_independent_validated_facts() {
        let (mut w, mut base) = inputs();
        w[32] = 1;
        w[34] = 1;
        w[35] = 0;
        assert!(Context::parse(&w, &base).is_ok());
        assert!(Context::parse(&w, &base).unwrap().fixture().is_err());
        base.calls[0].returned = 9;
        assert!(Context::parse(&w, &base).is_err());
    }
}
