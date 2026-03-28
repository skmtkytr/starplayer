use std::process::Command;

/// Find the mpv executable on the system.
fn find_mpv() -> Option<String> {
    let candidates = if cfg!(target_os = "windows") {
        vec![
            "mpv.exe".to_string(),
            r"C:\Program Files\mpv\mpv.exe".to_string(),
            r"C:\Program Files (x86)\mpv\mpv.exe".to_string(),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "mpv".to_string(),
            "/usr/local/bin/mpv".to_string(),
            "/opt/homebrew/bin/mpv".to_string(),
        ]
    } else {
        vec!["mpv".to_string(), "/usr/bin/mpv".to_string()]
    };

    for candidate in candidates {
        let result = Command::new(&candidate).arg("--version").output();
        if result.is_ok() {
            return Some(candidate);
        }
    }
    None
}

/// Play a video file using mpv as a subprocess.
pub fn play_file(path: &str) -> Result<(), String> {
    let mpv = find_mpv().ok_or_else(|| {
        "mpv not found. Please install mpv: https://mpv.io/installation/".to_string()
    })?;

    Command::new(&mpv)
        .arg(path)
        .arg("--force-window=yes")
        .spawn()
        .map_err(|e| format!("Failed to start mpv: {e}"))?;

    Ok(())
}

/// Play multiple files in sequence using mpv.
pub fn play_files(paths: &[String]) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No files to play".to_string());
    }

    let mpv = find_mpv().ok_or_else(|| {
        "mpv not found. Please install mpv: https://mpv.io/installation/".to_string()
    })?;

    let mut cmd = Command::new(&mpv);
    cmd.arg("--force-window=yes");
    for path in paths {
        cmd.arg(path);
    }

    cmd.spawn()
        .map_err(|e| format!("Failed to start mpv: {e}"))?;

    Ok(())
}
