use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    #[cfg(not(target_os = "android"))]
    if args.len() == 4 && args[1] == "boot-probe" {
        return xbx::probe::run(Path::new(&args[2]), Path::new(&args[3]));
    }
    if args.len() == 3 && args[1] == "inspect-disc" {
        let disc = xbx::disc::inspect(Path::new(&args[2]))?;
        println!(
            "XDVDFS: {} bytes, partition {}, root sector {}, root bytes {}. Title/platform compatibility is not established.",
            disc.bytes, disc.partition_offset, disc.root_sector, disc.root_bytes
        );
        println!(
            "XBE title {:?}, ID {:08x}, region mask {:08x}, version {}. Certificate metadata is unverified display information.",
            disc.title.name, disc.title.id, disc.title.region, disc.title.version
        );
        return Ok(());
    }
    Err("usage: xbx-host inspect-disc <complete-local-image> | boot-probe <source-built-xemu> <new-scratch-directory>".into())
}
