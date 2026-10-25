// SPDX-License-Identifier: MIT
//! Private desktop transport for the same Rust product session operations.
use serde_json::{Value, json};
use std::{
    io::{self, BufRead, Write},
    path::Path,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: xbx-player APP_DATA PACKAGED_RUNTIME".into());
    }
    let mut player =
        xbx::cartridge::desktop::Player::open(Path::new(&args[1]), Path::new(&args[2]))?;
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.len() > 65536 {
            return Err("request limit".into());
        }
        let v: Value = serde_json::from_str(&line)?;
        let result = (|| -> io::Result<Value> {
            let string = |name: &str| {
                v[name]
                    .as_str()
                    .ok_or_else(|| io::Error::other("missing request field"))
            };
            Ok(match string("op")? {
                "add" => json!({"game":player.add(Path::new(string("media")?))?}),
                "list" => {
                    serde_json::to_value(xbx::cartridge::library::Library::read(&player.app)?)
                        .map_err(io::Error::other)?
                }
                "relink" => {
                    let mut l = xbx::cartridge::library::Library::read(&player.app)?;
                    l.relink(&player.app, string("game")?, Path::new(string("media")?))?;
                    json!({})
                }
                "open" => json!({"restored":player.launch(string("game")?)?}),
                "return" => json!({"opened":player.return_last()?}),
                "pause" => {
                    player.pause(true)?;
                    json!({})
                }
                "resume" => {
                    player.resume()?;
                    json!({})
                }
                "commit" => json!({"checkpoint":player.commit()?}),
                "leave" => {
                    player.depart()?;
                    json!({})
                }
                "restart" => {
                    player.restart(string("game")?)?;
                    json!({})
                }
                "stop" => {
                    player.stop()?;
                    json!({})
                }
                "fixture-input" => {
                    if !xbx::cartridge::resources::ORIGINAL_FIXTURES.contains(
                        &player
                            .live
                            .as_ref()
                            .ok_or_else(|| io::Error::other("no game"))?
                            .binding
                            .media
                            .as_str(),
                    ) {
                        return Err(io::Error::other(
                            "terminal input is restricted to the two original fixtures",
                        ));
                    }
                    player
                        .live
                        .as_mut()
                        .ok_or_else(|| io::Error::other("no game"))?
                        .native
                        .fixture_input(string("text")?)?;
                    json!({})
                }
                "serial" => {
                    json!({"text":player.live.as_mut().ok_or_else(||io::Error::other("no game"))?.native.serial()?})
                }
                "quit" => {
                    player.depart()?;
                    json!({"quit":true})
                }
                _ => return Err(io::Error::other("unknown operation")),
            })
        })();
        let response = match result {
            Ok(value) => json!({"ok":value}),
            Err(e) => json!({"error":e.to_string()}),
        };
        println!("{response}");
        io::stdout().flush()?;
        if response["ok"]["quit"] == true {
            break;
        }
    }
    Ok(())
}
