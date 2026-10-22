//! Durable identity values carry no process, descriptor or renderer placement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameId(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualConsoleProfileId(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedSession {
    pub game: GameId,
    pub profile: VirtualConsoleProfileId,
    pub media_sha256: String,
    pub firmware_sha256: String,
    // This first player cold-boots current durable storage. No RAM checkpoint
    // is published or restored; profile dirty epochs remain authoritative.
}
