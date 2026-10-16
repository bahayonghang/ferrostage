//! Research-only process boundary; this binary must not import native media DLLs.
#![forbid(unsafe_code)]
use std::{path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let runtime = PathBuf::from(args.next().ok_or("missing runtime directory")?);
    let helper = PathBuf::from(args.next().ok_or("missing helper")?);
    let source = args.next().ok_or("missing source")?;
    let core = runtime.join("bin/gstreamer-1.0-0.dll");
    if !core.is_file() {
        println!("media_unavailable=missing_runtime control_process_ready=true");
        return Ok(());
    }
    let mut command = Command::new(helper);
    command.arg(source);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let output = command.output()?;
    println!("helper_exit={:?} control_process_ready=true", output.status.code());
    if !output.status.success() { return Err("helper failed".into()); }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
