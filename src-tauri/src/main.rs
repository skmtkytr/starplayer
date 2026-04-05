#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    if linux_wayland_guard() {
        return;
    }

    starplayer_lib::run();
}

/// Try launching on Wayland as-is; if the process crashes within a few seconds,
/// re-launch with `GDK_BACKEND=x11` as a fallback.
/// Returns `true` if this invocation is the guard (parent) process and should exit.
#[cfg(target_os = "linux")]
fn linux_wayland_guard() -> bool {
    // SAFETY: Called at the very start of main, before any other threads are spawned.
    unsafe {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    // Skip guard if: not a Wayland session, user set GDK_BACKEND explicitly,
    // or we are already the child process.
    if std::env::var_os("WAYLAND_DISPLAY").is_none()
        || std::env::var_os("GDK_BACKEND").is_some()
        || std::env::var_os("_STARPLAYER_LAUNCHED").is_some()
    {
        return false;
    }

    let exe = std::env::current_exe().expect("failed to get current exe");
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let start = std::time::Instant::now();

    let status = std::process::Command::new(&exe)
        .env("_STARPLAYER_LAUNCHED", "1")
        .args(&args)
        .status();

    match status {
        // Normal exit, or ran long enough that the crash isn't a display init failure
        Ok(s) if s.success() || start.elapsed().as_secs() >= 3 => {
            std::process::exit(s.code().unwrap_or(0));
        }
        // Quick crash — likely Wayland/WebKitGTK init failure, retry with X11
        _ => {
            eprintln!("starplayer: Wayland launch failed, retrying with X11 backend...");
            let code = std::process::Command::new(&exe)
                .env("_STARPLAYER_LAUNCHED", "1")
                .env("GDK_BACKEND", "x11")
                .args(&args)
                .status()
                .map(|s| s.code().unwrap_or(1))
                .unwrap_or(1);
            std::process::exit(code);
        }
    }
}
