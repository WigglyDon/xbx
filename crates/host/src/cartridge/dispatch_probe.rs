// SPDX-License-Identifier: MIT
//! Private, single-word diagnostic over the retained native RSP transport.
//! No general debugger CLI, target writes, instruction reads or retry of a read.
use super::{self as c, monitor::Monitor};
use serde::Deserialize;
use serde_json::json;
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Plan {
    id: String,
    media: String,
    core: String,
    open_system: String,
    address: u32,
    expected: u32,
    stop_pc: u32,
    index: u32,
}
impl Plan {
    fn validate(&self, media: &str, core: &str, open: &str) -> io::Result<()> {
        if !c::valid_id(&self.id)
            || self.media != media
            || self.core != core
            || self.open_system != open
            || self.index != 0
            || self.address & 3 != 0
            || !(0x10000..=0x7fff_fffc).contains(&self.address)
            || !(0x8001_0000..0x8040_0000).contains(&self.stop_pc)
        {
            return Err(io::Error::other("private single-word bindings refused"));
        }
        Ok(())
    }
}
struct Wire {
    socket: UnixStream,
    buffer: Vec<u8>,
}
impl Wire {
    fn send(&mut self, text: &str) -> io::Result<()> {
        let sum = text.bytes().fold(0u8, u8::wrapping_add);
        self.socket
            .write_all(format!("${text}#{sum:02x}").as_bytes())
    }
    fn packet(&mut self) -> io::Result<Option<String>> {
        while self.buffer.first() == Some(&b'+') {
            self.buffer.remove(0);
        }
        if self.buffer.is_empty() {
            return Ok(None);
        }
        if self.buffer[0] != b'$' {
            return Err(io::Error::other("unexpected diagnostic packet"));
        }
        let Some(end) = self.buffer.iter().position(|&b| b == b'#') else {
            return Ok(None);
        };
        if self.buffer.len() < end + 3 {
            return Ok(None);
        }
        let expected = u8::from_str_radix(
            std::str::from_utf8(&self.buffer[end + 1..end + 3]).map_err(io::Error::other)?,
            16,
        )
        .map_err(io::Error::other)?;
        if self.buffer[1..end]
            .iter()
            .copied()
            .fold(0u8, u8::wrapping_add)
            != expected
        {
            return Err(io::Error::other("diagnostic checksum mismatch"));
        }
        let text = String::from_utf8(self.buffer[1..end].to_vec()).map_err(io::Error::other)?;
        self.buffer.drain(..end + 3);
        self.socket.write_all(b"+")?;
        Ok(Some(text))
    }
    fn poll(&mut self) -> io::Result<Option<String>> {
        if let Some(text) = self.packet()? {
            return Ok(Some(text));
        }
        let mut bytes = [0; 1024];
        match self.socket.read(&mut bytes) {
            Ok(0) => return Err(io::Error::other("diagnostic transport closed")),
            Ok(n) => {
                if self.buffer.len() + n > 4096 {
                    return Err(io::Error::other("diagnostic packet limit"));
                }
                self.buffer.extend_from_slice(&bytes[..n]);
            }
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(e) => return Err(e),
        }
        self.packet()
    }
    fn command(&mut self, text: &str) -> io::Result<String> {
        self.send(text)?;
        let end = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(reply) = self.poll()? {
                return Ok(reply);
            }
            if Instant::now() > end {
                return Err(io::Error::other("diagnostic response deadline"));
            }
        }
    }
    fn word(&mut self, text: &str) -> io::Result<u32> {
        decode_word(&self.command(text)?)
    }
}
fn decode_word(text: &str) -> io::Result<u32> {
    if text.len() != 8 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(io::Error::other("diagnostic word unavailable"));
    }
    let mut bytes = [0; 4];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).map_err(io::Error::other)?;
    }
    Ok(u32::from_le_bytes(bytes))
}
pub(crate) struct Probe {
    plan: Plan,
    wire: Wire,
    base: PathBuf,
    socket_path: PathBuf,
    result: PathBuf,
    began: bool,
    complete: bool,
    attached: bool,
    started: Instant,
}
impl Probe {
    pub(crate) fn prepare(
        base: &Path,
        media: &str,
        core: &str,
        open: &str,
        socket_path: &Path,
    ) -> io::Result<Self> {
        let input = base.join("dispatch-probe.json");
        let metadata = fs::symlink_metadata(&input)?;
        if !metadata.is_file() || metadata.len() > 4096 {
            return Err(io::Error::other("private probe plan refused"));
        }
        let plan: Plan = c::read_json(&input)?;
        plan.validate(media, core, open)?;
        let claim = base.join(format!("dispatch-{}.claimed", plan.id));
        c::write_new(&claim, b"Single prepared attempt; no automatic replay.\n")?;
        let retained = base.join(format!("dispatch-{}.plan.json", plan.id));
        if retained.try_exists()? {
            return Err(io::Error::other("probe plan already retained"));
        }
        fs::rename(input, retained)?;
        File::open(base)?.sync_all()?;
        use std::os::unix::fs::FileTypeExt;
        if socket_path.parent() != Some(base)
            || socket_path.as_os_str().len() > 100
            || !fs::symlink_metadata(socket_path)?.file_type().is_socket()
        {
            return Err(io::Error::other("private diagnostic socket refused"));
        }
        c::write_new(
            &base.join(format!("dispatch-{}.server.json", plan.id)),
            &c::json(
                &json!({"route":"native startup -gdb private Unix socket","no_HMP_gdbserver_dependency":true}),
            )?,
        )?;
        let socket = UnixStream::connect(socket_path)?;
        socket.set_read_timeout(Some(Duration::from_millis(10)))?;
        socket.set_write_timeout(Some(Duration::from_secs(2)))?;
        let mut wire = Wire {
            socket,
            buffer: Vec::new(),
        };
        let stopped = wire.command("?")?;
        if !stopped.starts_with(['S', 'T']) {
            return Err(io::Error::other("native diagnostic target not stopped"));
        }
        if wire.command(&format!("Z1,{:x},1", plan.stop_pc))? != "OK" {
            return Err(io::Error::other("native execution breakpoint refused"));
        }
        let result = base.join(format!("dispatch-{}.result.json", plan.id));
        Ok(Self {
            plan,
            wire,
            base: base.into(),
            socket_path: socket_path.into(),
            result,
            began: false,
            complete: false,
            attached: true,
            started: Instant::now(),
        })
    }
    pub(crate) fn begin(&mut self) -> io::Result<()> {
        if !self.began {
            self.wire.send("c")?;
            self.began = true;
            self.started = Instant::now();
        }
        Ok(())
    }
    pub(crate) fn pending(&self) -> bool {
        !self.complete || self.attached
    }
    pub(crate) fn poll(&mut self, monitor: &mut Monitor) -> io::Result<()> {
        if !self.began || self.complete {
            return Ok(());
        }
        let packet = self.wire.poll();
        if matches!(packet, Ok(None)) && self.started.elapsed() < Duration::from_secs(90) {
            return Ok(());
        }
        let mut attempted = false;
        let compared = (|| -> io::Result<(bool, bool)> {
            let stop = packet?.ok_or_else(|| io::Error::other("diagnostic stop deadline"))?;
            if !stop.starts_with("T05") && !stop.starts_with("S05") {
                return Err(io::Error::other("unexpected diagnostic stop"));
            }
            if self.wire.word("p8")? != self.plan.stop_pc
                || self.wire.word("p2")? != self.plan.index
            {
                return Err(io::Error::other(
                    "diagnostic PC or index differs; no title read",
                ));
            }
            // Durable refusal of a repeat precedes the one m packet, including
            // a disconnect after submission. Never follow the resulting word.
            c::write_new(
                &self
                    .base
                    .join(format!("dispatch-{}.read-requested", self.plan.id)),
                b"4\n",
            )?;
            attempted = true;
            let value = self.wire.word(&format!("m{:x},4", self.plan.address))?;
            Ok((value == 0, value == self.plan.expected))
        })();
        self.complete = true;
        let result = match compared {
            Ok((zero, equal)) => {
                json!({"completed":true,"read_attempted":true,"bytes_requested":4,"valid_word":true,"zero":zero,"matches_file":equal,"PC_and_index_matched":true,"raw_value_published":false})
            }
            Err(e) => {
                json!({"completed":false,"read_attempted":attempted,"bytes_requested":if attempted {4} else {0},"error":e.to_string(),"raw_value_published":false})
            }
        };
        c::write_new(&self.result, &c::json(&result)?)?;
        let removed = self.wire.command(&format!("z1,{:x},1", self.plan.stop_pc));
        let detached = self.wire.command("D");
        self.attached = !matches!(detached.as_deref(), Ok("OK"));
        c::write_new(
            &self.result.with_extension("detach.json"),
            &c::json(
                &json!({"acknowledged":!self.attached,"owned_breakpoint_removed":matches!(removed.as_deref(),Ok("OK"))}),
            )?,
        )?;
        monitor.resume()?;
        Ok(())
    }
    pub(crate) fn close(&mut self) {
        if self.attached {
            let _ = self.wire.command(&format!("z1,{:x},1", self.plan.stop_pc));
            let _ = self.wire.command("D");
        }
        let _ = self.wire.socket.shutdown(std::net::Shutdown::Both);
    }
    pub(crate) fn remove_socket_after_native_exit(&self) {
        let _ = fs::remove_file(&self.socket_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_little_endian_word_only() {
        assert_eq!(decode_word("78563412").unwrap(), 0x12345678);
        for text in ["E14", "0000000000", "0000", "xxxxxxxx"] {
            assert!(decode_word(text).is_err());
        }
    }
    #[test]
    fn binding_and_range_refusal() {
        let mut p = Plan {
            id: "a".repeat(32),
            media: "media".into(),
            core: "core".into(),
            open_system: "open".into(),
            address: 0x12000,
            expected: 1,
            stop_pc: 0x80012001,
            index: 0,
        };
        assert!(p.validate("media", "core", "open").is_ok());
        assert!(p.validate("other", "core", "open").is_err());
        p.address = 0x12001;
        assert!(p.validate("media", "core", "open").is_err());
        p.address = 0x12000;
        p.index = 1;
        assert!(p.validate("media", "core", "open").is_err());
    }
    #[test]
    fn fragmented_packet_and_checksum_refusal() {
        let (s, _peer) = UnixStream::pair().unwrap();
        let mut w = Wire {
            socket: s,
            buffer: b"+$01000000#".to_vec(),
        };
        assert!(w.packet().unwrap().is_none());
        w.buffer.extend_from_slice(b"81");
        assert_eq!(w.packet().unwrap(), Some("01000000".into()));
        w.buffer.extend_from_slice(b"$00000000#00");
        assert!(w.packet().is_err());
    }
}
