// SPDX-License-Identifier: MIT
//! Shared acknowledged native whole-machine control, including paused capture.
//! Transports supply an already connected private socket; Rust session policy
//! remains in player. No frame, preview or alternate state representation.
use super::*;
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
    time::Duration,
};
pub(crate) struct Monitor {
    reader: BufReader<UnixStream>,
    socket: UnixStream,
    sequence: u64,
}
impl Monitor {
    pub fn connected(socket: UnixStream) -> io::Result<Self> {
        socket.set_read_timeout(Some(Duration::from_secs(90)))?;
        socket.set_write_timeout(Some(Duration::from_secs(5)))?;
        let mut this = Self {
            reader: BufReader::new(socket.try_clone()?),
            socket,
            sequence: 0,
        };
        let mut greeting = String::new();
        this.reader.read_line(&mut greeting)?;
        this.call("qmp_capabilities", json!({}))?;
        this.call("query-status", json!({}))?;
        Ok(this)
    }
    pub fn call(&mut self, execute: &str, arguments: Value) -> io::Result<Value> {
        self.sequence += 1;
        let id = self.sequence;
        let request = json!({"execute":execute,"arguments":arguments,"id":id});
        writeln!(self.socket, "{request}")?;
        loop {
            let mut line = String::new();
            if self.reader.read_line(&mut line)? == 0 {
                return Err(io::Error::other("native control closed"));
            }
            if line.len() > 1024 * 1024 {
                return Err(io::Error::other("native control output limit"));
            }
            let v: Value = serde_json::from_str(&line).map_err(io::Error::other)?;
            if v.get("id") == Some(&json!(id)) {
                if let Some(e) = v.get("error") {
                    return Err(io::Error::other(format!("native refused: {e}")));
                }
                return v
                    .get("return")
                    .cloned()
                    .ok_or_else(|| io::Error::other("missing native acknowledgement"));
            }
        }
    }
    fn hmp(&mut self, command: &str) -> io::Result<String> {
        let v = self.call("human-monitor-command", json!({"command-line":command}))?;
        v.as_str()
            .map(str::to_owned)
            .ok_or_else(|| io::Error::other("invalid native HMP response"))
    }
    pub fn pause(&mut self) -> io::Result<()> {
        if self.call("query-status", json!({}))?["running"] == true {
            self.call("stop", json!({}))?;
        }
        let v = self.call("query-status", json!({}))?;
        if v["running"] != false {
            return Err(io::Error::other("machine did not quiesce"));
        }
        Ok(())
    }
    pub fn resume(&mut self) -> io::Result<()> {
        self.call("cont", json!({}))?;
        Ok(())
    }
    pub fn capture(&mut self) -> io::Result<()> {
        self.pause()?;
        let result = self.hmp("savevm xbx-session")?;
        if !result.trim().is_empty() {
            return Err(io::Error::other(format!("native capture: {result}")));
        }
        let listing = self.hmp("info snapshots")?;
        if !listing.contains("xbx-session") {
            return Err(io::Error::other("native snapshot absent"));
        }
        Ok(())
    }
    pub fn load(&mut self) -> io::Result<()> {
        self.pause()?;
        let result = self.hmp("loadvm xbx-session")?;
        if !result.trim().is_empty() {
            return Err(io::Error::other(format!("native restore: {result}")));
        }
        self.pause()
    }
}
