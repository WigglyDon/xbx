use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    #[cfg(not(target_os = "android"))]
    if args.len() == 3 && args[1] == "check-startup-totals-fixture" {
        let root = Path::new(&args[2]);
        let base = xbx::observer::load(&root.join("observer-raw.bin"))?;
        base.fixture()?;
        xbx::contract::load(&root.join("context-raw.bin"), &base)?.fixture()?;
        return xbx::totals::load(&root.join("startup-totals.bin"), &base)?.fixture(&base);
    }
    #[cfg(not(target_os = "android"))]
    if args.len() == 3 && args[1] == "check-context-fixture" {
        let root = Path::new(&args[2]);
        let base = xbx::observer::load(&root.join("observer-raw.bin"))?;
        base.fixture()?;
        return xbx::contract::load(&root.join("context-raw.bin"), &base)?.fixture();
    }
    #[cfg(not(target_os = "android"))]
    if args.len() == 3 && (args[1] == "check-observer-fixture" || args[1] == "summarize-observer") {
        let capture = xbx::observer::load(Path::new(&args[2]))?;
        if args[1] == "check-observer-fixture" {
            return capture.fixture();
        }
        print!("{}", capture.receipt());
        return Ok(());
    }
    #[cfg(not(target_os = "android"))]
    if args.len() == 3 && args[1] == "check-journal-fixture" {
        return xbx::api_journal::validate_fixture(Path::new(&args[2]));
    }
    #[cfg(not(target_os = "android"))]
    if args.len() == 3 && args[1] == "check-thread-fixture" {
        return xbx::api_journal::validate_thread_fixture(Path::new(&args[2]));
    }
    #[cfg(not(target_os = "android"))]
    if args.len() == 4 && args[1] == "check-npx-fixture" {
        let corrected = match args[3].to_str() {
            Some("preserved") => true,
            Some("displaced-owner") => false,
            _ => return Err("expected preserved or displaced-owner".into()),
        };
        return xbx::api_journal::validate_npx_fixture(Path::new(&args[2]), corrected);
    }
    if args.len() == 3 && args[1] == "summarize-runtime" {
        if std::fs::metadata(&args[2])?.len() > 1024 * 1024 {
            return Err("runtime log exceeds bound".into());
        }
        let text = std::fs::read_to_string(&args[2])?;
        print!("{}", xbx::runtime_summary::Summary::parse(&text).receipt());
        return Ok(());
    }
    #[cfg(not(target_os = "android"))]
    if args.len() == 9 && args[1] == "boot-loader-fixture" {
        return xbx::probe::loader_fixture(
            Path::new(&args[2]),
            Path::new(&args[3]),
            Path::new(&args[4]),
            Path::new(&args[5]),
            Path::new(&args[6]),
            Path::new(&args[7]),
            args[8].to_str().ok_or("invalid fixture expectation")?,
        );
    }
    #[cfg(not(target_os = "android"))]
    if (args.len() == 8 || args.len() == 9) && args[1] == "run-dashboard" {
        return xbx::desktop_title::run_dashboard(
            Path::new(&args[2]),
            Path::new(&args[3]),
            Path::new(&args[4]),
            Path::new(&args[5]),
            Path::new(&args[6]),
            args[7].to_str().ok_or("invalid time budget")?.parse()?,
            args.get(8).map(Path::new),
        );
    }
    #[cfg(not(target_os = "android"))]
    if ((args.len() == 9 || args.len() == 10) && args[1] == "run-title")
        || (args.len() == 10 && args[1] == "run-title-journal")
    {
        return xbx::desktop_title::run(
            Path::new(&args[2]),
            Path::new(&args[3]),
            Path::new(&args[4]),
            Path::new(&args[5]),
            Path::new(&args[6]),
            Path::new(&args[7]),
            args[8].to_str().ok_or("invalid time budget")?.parse()?,
            if args[1] == "run-title" {
                args.get(9).map(Path::new)
            } else {
                None
            },
            if args[1] == "run-title-journal" {
                args.get(9).map(Path::new)
            } else {
                None
            },
        );
    }
    #[cfg(not(target_os = "android"))]
    if args.len() == 8 && args[1] == "boot-vm-imports" {
        return xbx::probe::vm_imports(
            Path::new(&args[2]),
            Path::new(&args[3]),
            Path::new(&args[4]),
            Path::new(&args[5]),
            Path::new(&args[6]),
            Path::new(&args[7]),
        );
    }
    #[cfg(not(target_os = "android"))]
    if args.len() == 8 && args[1] == "boot-nullcall" {
        return xbx::probe::null_call(
            Path::new(&args[2]),
            Path::new(&args[3]),
            Path::new(&args[4]),
            Path::new(&args[5]),
            Path::new(&args[6]),
            Path::new(&args[7]),
        );
    }
    #[cfg(not(target_os = "android"))]
    if args.len() == 8 && args[1] == "boot-relaunch" {
        return xbx::probe::relaunch(
            Path::new(&args[2]),
            Path::new(&args[3]),
            Path::new(&args[4]),
            Path::new(&args[5]),
            Path::new(&args[6]),
            Path::new(&args[7]),
        );
    }
    #[cfg(not(target_os = "android"))]
    if args.len() == 7 && args[1] == "boot-kernel" {
        return xbx::probe::kernel(
            Path::new(&args[2]),
            Path::new(&args[3]),
            Path::new(&args[4]),
            Path::new(&args[5]),
            Path::new(&args[6]),
        );
    }
    if args.len() == 3 && args[1] == "empty-disk" {
        xbx::disk::create_empty_raw(Path::new(&args[2]))?;
        println!(
            "Created new sparse retail-layout FATX fixture; existing files are never formatted."
        );
        return Ok(());
    }
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
    Err("usage: xbx-host inspect-disc IMAGE | run-dashboard ENGINE NEW_SCRATCH FLASH HDD EEPROM SECONDS [MCPX] | run-title ENGINE NEW_SCRATCH FLASH HDD EEPROM MEDIA SECONDS [MCPX] | boot-probe ENGINE NEW_SCRATCH".into())
}
