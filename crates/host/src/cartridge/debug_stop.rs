// SPDX-License-Identifier: MIT
//! Bounded developer stop control over the retained core's actual RSP socket.
//! Protocol: sourceware.org/gdb/current/onlinedocs/gdb.html/Overview.html and
//! Packets.html; retained QEMU gdbstub. No memory/register writes or general CLI.
use super::{desktop::Native, fault_context::hex_words};
use serde_json::json;
use std::{
    fs::File,
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::Path,
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
struct Wire {
    socket: UnixStream,
    log: File,
    started: Instant,
    budget: Duration,
}
impl Wire {
    fn byte(&mut self, native: &mut Native, serial: &mut File) -> Result<u8, Error> {
        loop {
            if self.started.elapsed() >= self.budget {
                return Err("finite stop-control deadline".into());
            }
            let mut b = [0];
            match self.socket.read(&mut b) {
                Ok(1) => return Ok(b[0]),
                Ok(_) => return Err("stop-control socket closed".into()),
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock
                            | std::io::ErrorKind::TimedOut
                            | std::io::ErrorKind::Interrupted
                    ) =>
                {
                    serial.write_all(native.serial()?.as_bytes())?;
                    if serial.metadata()?.len() > 1024 * 1024 {
                        return Err("ordinary serial output bound".into());
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
    fn exchange(
        &mut self,
        native: &mut Native,
        serial: &mut File,
        command: &str,
    ) -> Result<String, Error> {
        let sum = command.bytes().fold(0u8, u8::wrapping_add);
        self.socket
            .write_all(format!("${command}#{sum:02x}").as_bytes())?;
        loop {
            match self.byte(native, serial)? {
                b'+' => continue,
                b'$' => break,
                _ => return Err("unexpected remote acknowledgement".into()),
            }
        }
        let mut payload = Vec::new();
        loop {
            let b = self.byte(native, serial)?;
            if b == b'#' {
                break;
            }
            if payload.len() >= 4096 {
                return Err("remote packet bound".into());
            }
            payload.push(b);
        }
        let checksum = [self.byte(native, serial)?, self.byte(native, serial)?];
        let expected = u8::from_str_radix(std::str::from_utf8(&checksum)?, 16)?;
        if payload.iter().copied().fold(0u8, u8::wrapping_add) != expected {
            return Err("remote checksum mismatch".into());
        }
        self.socket.write_all(b"+")?;
        let reply = String::from_utf8(payload)?;
        writeln!(
            self.log,
            "{}",
            json!({"command":command,"reply":reply,"elapsed_ms":self.started.elapsed().as_millis()})
        )?;
        if self.log.metadata()?.len() > 65536 {
            return Err("remote transcript bound".into());
        }
        self.log.sync_all()?;
        Ok(reply)
    }
}
/// Each stop is installed while paused, executed once and independently checked
/// through HMP. Only the acknowledged owned breakpoint is removed. Closing the
/// socket does not send detach/resume; the parent reconfirms the final paused PC.
pub(super) fn run(
    native: &mut Native,
    socket: &Path,
    stops: &[u32],
    seconds: u64,
    log: File,
    serial: &mut File,
) -> Result<(), Error> {
    if stops.is_empty()
        || stops.len() > 2
        || !(30..=300).contains(&seconds)
        || stops.iter().any(|p| !(0x10000..0x80000000).contains(p))
    {
        return Err("bounded stop plan required".into());
    }
    let socket = UnixStream::connect(socket)?;
    socket.set_read_timeout(Some(Duration::from_millis(50)))?;
    socket.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut wire = Wire {
        socket,
        log,
        started: Instant::now(),
        budget: Duration::from_secs(seconds),
    };
    let initial = wire.exchange(native, serial, "?")?;
    if !(initial.starts_with('T') || initial.starts_with('S')) {
        return Err("initial stop reply required".into());
    }
    for pc in stops {
        if wire.exchange(native, serial, &format!("Z1,{pc:x},1"))? != "OK" {
            return Err("hardware stop not accepted".into());
        }
        let stop = wire.exchange(native, serial, "c")?;
        if !(stop.starts_with("T05") || stop.starts_with("S05")) {
            return Err("expected trap stop reply".into());
        }
        native.pause()?;
        let actual = native.call("human-monitor-command", json!({"command-line":"p /x $eip"}))?;
        if hex_words(actual.as_str().ok_or("stop PC response")?).last() != Some(pc) {
            return Err("remote stop PC mismatch".into());
        }
        writeln!(wire.log, "{}", json!({"verified_pc":actual}))?;
        if wire.exchange(native, serial, &format!("z1,{pc:x},1"))? != "OK" {
            return Err("owned hardware stop removal failed".into());
        }
    }
    wire.log.sync_all()?;
    serial.sync_all()?;
    Ok(())
}
