//! Installing a downloaded update. Plain logic with no OS gating, so it is tested on Linux.

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::{UpdateError, UpdaterConfig};

/// Name of the uninstaller the Inno Setup installer leaves next to the app's exe.
const WINDOWS_UNINSTALLER: &str = "unins000.exe";

/// The arguments the Windows installer is started with.
#[must_use]
pub fn windows_installer_args() -> [&'static str; 4] {
    [
        "/VERYSILENT",
        "/SUPPRESSMSGBOXES",
        "/NORESTART",
        "/relaunch=1",
    ]
}

/// Windows: whether `exe` runs from a copy made by the installer, which always leaves its
/// uninstaller next to the program. A portable copy (the zip) has none and must not be updated by
/// running the installer, which would add a second copy somewhere else.
#[must_use]
pub fn windows_is_installed(exe: &Path) -> bool {
    exe.parent()
        .is_some_and(|dir| dir.join(WINDOWS_UNINSTALLER).is_file())
}

/// Windows: starts the downloaded installer silently and returns. The caller quits right after.
///
/// # Errors
///
/// When the installer cannot be started.
pub fn spawn_windows_installer(installer: &Path) -> io::Result<()> {
    Command::new(installer)
        .args(windows_installer_args())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(drop)
}

/// macOS: the `.app` folder that contains `exe` (`.../Postino.app` for
/// `.../Postino.app/Contents/MacOS/postino`), or `None` when it is not inside a bundle.
#[must_use]
pub fn macos_bundle_from_exe(exe: &Path) -> Option<PathBuf> {
    exe.ancestors()
        .skip(1)
        .find(|path| path.extension().is_some_and(|ext| ext == "app"))
        .map(Path::to_path_buf)
}

/// macOS: unpacks `archive` (`.app.tar.gz`) with the system `tar` into a staging folder next to
/// `bundle` (same volume, so the swap is a rename) and returns the staged
/// [`bundle_name`](UpdaterConfig::bundle_name).
///
/// # Errors
///
/// An [`UpdateError::Io`] of kind `PermissionDenied` when the folder that holds the bundle is
/// not writable (the app then falls back to opening the release page), or any other IO failure,
/// including `tar` failing or the archive not containing the app bundle.
pub fn macos_stage(
    config: &UpdaterConfig,
    archive: &Path,
    bundle: &Path,
) -> Result<PathBuf, UpdateError> {
    let parent = bundle
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the bundle has no parent"))?;
    let staging = parent.join(config.staging_dir());
    match std::fs::remove_dir_all(&staging) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    std::fs::create_dir(&staging)?;
    let output = Command::new("tar")
        .arg("-xzf")
        .arg(archive)
        .arg("-C")
        .arg(&staging)
        .stdin(Stdio::null())
        .output()?;
    if !output.status.success() {
        let _ = std::fs::remove_dir_all(&staging);
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::other(format!("tar failed: {}", detail.trim())).into());
    }
    let staged = staging.join(config.bundle_name());
    if !staged.is_dir() {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("the archive does not contain {}", config.bundle_name()),
        )
        .into());
    }
    Ok(staged)
}

/// Quotes `text` for `sh` with single quotes, so any character is safe.
fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

/// The script that swaps the bundle once the app has quit: waits for `pid` to exit, moves the old
/// bundle aside into the staging folder (the folder of `staged`), moves `staged` into place and
/// deletes the staging folder. If the new bundle cannot be moved in, the old one is restored.
/// Finally runs `reopen` with the bundle path as its last argument (`open` on macOS).
#[must_use]
pub fn macos_swap_script(
    config: &UpdaterConfig,
    pid: u32,
    bundle: &Path,
    staged: &Path,
    reopen: &str,
) -> String {
    let staging = staged.parent().unwrap_or(staged);
    let backup = staging.join(format!("{}.old.app", config.app_name));
    let bundle = quote(&bundle.to_string_lossy());
    let staged = quote(&staged.to_string_lossy());
    let staging = quote(&staging.to_string_lossy());
    let backup = quote(&backup.to_string_lossy());
    format!(
        "while kill -0 {pid} 2>/dev/null; do sleep 0.2; done\n\
         if mv {bundle} {backup}; then\n\
         \x20 if mv {staged} {bundle}; then\n\
         \x20   rm -rf {staging}\n\
         \x20 else\n\
         \x20   mv {backup} {bundle}\n\
         \x20 fi\n\
         fi\n\
         {reopen} {bundle}\n"
    )
}

/// Runs `script` with `/bin/sh -c`, detached (own process group, no stdio), and returns. The
/// caller quits right after.
///
/// # Errors
///
/// When the shell cannot be started.
pub fn spawn_detached_script(script: &str) -> io::Result<()> {
    let mut command = Command::new("/bin/sh");
    command
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    command.spawn().map(drop)
}

/// Linux: the name of the install script inside the release tarball, in its top folder
/// (`postino-1.2.0-linux-x86_64/install.sh`). It is the same script the app's website serves.
pub const LINUX_INSTALL_SCRIPT: &str = "install.sh";

/// Linux: the prefix an app's install script installed `exe` into, or `None` when it did not.
///
/// The script is expected to put the binary in `<prefix>/bin/` and the list of the files it
/// installed in `<prefix>/share/<app>/installed-files`, `<app>` being the lowercase
/// [`app_name`](UpdaterConfig::app_name). Reads the filesystem, so call it once.
#[must_use]
pub fn linux_script_prefix(config: &UpdaterConfig, exe: &Path) -> Option<PathBuf> {
    let prefix = exe
        .parent()
        .filter(|bin| bin.ends_with("bin"))
        .and_then(Path::parent)?;
    prefix
        .join("share")
        .join(config.app_name.to_lowercase())
        .join("installed-files")
        .is_file()
        .then(|| prefix.to_path_buf())
}

/// Whether the user can write in `dir`: a file can be created there and removed again. A missing
/// folder counts as not writable.
#[must_use]
pub fn is_writable_dir(dir: &Path) -> bool {
    let probe = dir.join(format!(".telaio-update-probe-{}", std::process::id()));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(file) => {
            drop(file);
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Linux: whether the update of `prefix` can be installed by the user, without root: its `bin`
/// and `share` folders are writable.
#[must_use]
pub fn linux_prefix_is_writable(prefix: &Path) -> bool {
    is_writable_dir(&prefix.join("bin")) && is_writable_dir(&prefix.join("share"))
}

/// Linux: installs the verified `tarball` into `prefix` with the install script the tarball ships
/// ([`LINUX_INSTALL_SCRIPT`]), run as `bash install.sh --tarball <tarball> --prefix <prefix>`.
/// The script lays the files out, rewrites its list of installed files and refreshes the desktop
/// caches, exactly as when the user runs it. Linux lets a running executable be replaced, so the
/// app keeps running until it restarts. Blocking.
///
/// # Errors
///
/// An IO failure, `tar` failing, a tarball without the script, or the script failing, with the
/// last line it wrote to standard error.
pub fn linux_run_install_script(
    config: &UpdaterConfig,
    tarball: &Path,
    prefix: &Path,
) -> Result<(), UpdateError> {
    let parent = tarball
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the tarball has no parent"))?;
    let staging = parent.join(config.staging_dir());
    match std::fs::remove_dir_all(&staging) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    std::fs::create_dir_all(&staging)?;
    let result = stage_and_run(tarball, prefix, &staging);
    let _ = std::fs::remove_dir_all(&staging);
    result
}

/// [`linux_run_install_script`] once `staging` exists: unpacks the script and runs it.
fn stage_and_run(tarball: &Path, prefix: &Path, staging: &Path) -> Result<(), UpdateError> {
    let output = Command::new("tar")
        .arg("-xzf")
        .arg(tarball)
        .arg("-C")
        .arg(staging)
        .arg("--wildcards")
        .arg(format!("*/{LINUX_INSTALL_SCRIPT}"))
        .stdin(Stdio::null())
        .output()?;
    if !output.status.success() {
        return Err(failure("tar", &output.stderr));
    }
    let script = std::fs::read_dir(staging)?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join(LINUX_INSTALL_SCRIPT))
        .find(|script| script.is_file())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("the tarball does not contain {LINUX_INSTALL_SCRIPT}"),
            )
        })?;
    log::info!("installing the update into {}", prefix.display());
    let output = Command::new("bash")
        .arg(&script)
        .arg("--tarball")
        .arg(tarball)
        .arg("--prefix")
        .arg(prefix)
        .stdin(Stdio::null())
        .output()?;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        log::info!("{line}");
    }
    if !output.status.success() {
        return Err(failure(LINUX_INSTALL_SCRIPT, &output.stderr));
    }
    Ok(())
}

/// An error for a command that failed, with the last line of its standard error.
fn failure(command: &str, stderr: &[u8]) -> UpdateError {
    let stderr = String::from_utf8_lossy(stderr);
    let detail = stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("no output");
    io::Error::other(format!("{command} failed: {}", detail.trim())).into()
}

/// Linux: the path of the executable to start after an update, `exe` as the running process sees
/// it. Once the install replaced the file, the kernel reports the old one as
/// `<path> (deleted)`; the suffix is dropped so the new file starts.
#[must_use]
pub fn linux_restart_path(exe: &Path) -> PathBuf {
    let text = exe.to_string_lossy();
    PathBuf::from(text.strip_suffix(" (deleted)").unwrap_or(&text))
}

/// Linux: the script that starts `exe` again once the app has quit: waits for `pid` to exit, then
/// runs it. The new files are already in place.
#[must_use]
pub fn linux_restart_script(pid: u32, exe: &Path) -> String {
    let exe = quote(&exe.to_string_lossy());
    format!("while kill -0 {pid} 2>/dev/null; do sleep 0.2; done\nexec {exe}\n")
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;

    const CONFIG: UpdaterConfig = UpdaterConfig {
        owner: "tanisperez",
        repo: "postino",
        app_name: "Postino",
        url_env: "POSTINO_UPDATE_URL",
        version: "0.1.0",
    };

    #[test]
    fn windows_args() {
        assert_eq!(
            windows_installer_args(),
            [
                "/VERYSILENT",
                "/SUPPRESSMSGBOXES",
                "/NORESTART",
                "/relaunch=1"
            ]
        );
    }

    #[test]
    fn windows_installed_copy_has_its_uninstaller_next_to_the_exe() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("postino.exe");
        fs::write(&exe, "exe").unwrap();
        assert!(!windows_is_installed(&exe));
        fs::write(dir.path().join(WINDOWS_UNINSTALLER), "uninstaller").unwrap();
        assert!(windows_is_installed(&exe));
    }

    #[test]
    fn windows_uninstaller_elsewhere_does_not_count() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(WINDOWS_UNINSTALLER), "uninstaller").unwrap();
        let portable = dir.path().join("portable");
        fs::create_dir(&portable).unwrap();
        assert!(!windows_is_installed(&portable.join("postino.exe")));
        // A folder with that name is not an uninstaller.
        let fake = dir.path().join("fake");
        fs::create_dir_all(fake.join(WINDOWS_UNINSTALLER)).unwrap();
        assert!(!windows_is_installed(&fake.join("postino.exe")));
        assert!(!windows_is_installed(Path::new("")));
    }

    #[test]
    fn bundle_from_exe() {
        let exe = Path::new("/Applications/Postino.app/Contents/MacOS/postino");
        assert_eq!(
            macos_bundle_from_exe(exe),
            Some(PathBuf::from("/Applications/Postino.app"))
        );
        assert_eq!(macos_bundle_from_exe(Path::new("/usr/bin/postino")), None);
        assert_eq!(macos_bundle_from_exe(Path::new("/x/Postino.app")), None);
    }

    #[test]
    fn quoting() {
        assert_eq!(quote("a b"), "'a b'");
        assert_eq!(quote("it's"), "'it'\\''s'");
    }

    fn make_bundle(path: &Path, marker: &str) {
        fs::create_dir_all(path.join("Contents")).unwrap();
        fs::write(path.join("Contents/marker"), marker).unwrap();
    }

    fn exited_pid() -> u32 {
        let mut child = Command::new("true").spawn().unwrap();
        let pid = child.id();
        child.wait().unwrap();
        pid
    }

    fn run_script(script: &str) {
        let status = Command::new("/bin/sh")
            .arg("-c")
            .arg(script)
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[test]
    fn swap_script_replaces_the_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("my apps it's");
        let bundle = apps.join("Postino.app");
        make_bundle(&bundle, "old");
        let staging = apps.join(CONFIG.staging_dir());
        let staged = staging.join("Postino.app");
        make_bundle(&staged, "new");
        let reopened = dir.path().join("reopened file");
        let reopen = format!("printf '%s\\n' > {}", quote(&reopened.to_string_lossy()));

        run_script(&macos_swap_script(
            &CONFIG,
            exited_pid(),
            &bundle,
            &staged,
            &reopen,
        ));

        assert_eq!(
            fs::read_to_string(bundle.join("Contents/marker")).unwrap(),
            "new"
        );
        assert!(!staging.exists());
        assert_eq!(
            fs::read_to_string(reopened).unwrap().trim(),
            bundle.to_string_lossy()
        );
    }

    #[test]
    fn swap_script_restores_the_old_bundle_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("Postino.app");
        make_bundle(&bundle, "old");
        let staging = dir.path().join(CONFIG.staging_dir());
        fs::create_dir(&staging).unwrap();
        let staged = staging.join("Postino.app"); // does not exist, so the second move fails
        let reopened = dir.path().join("reopened");
        let reopen = format!("printf '%s\\n' > {}", quote(&reopened.to_string_lossy()));

        run_script(&macos_swap_script(
            &CONFIG,
            exited_pid(),
            &bundle,
            &staged,
            &reopen,
        ));

        assert_eq!(
            fs::read_to_string(bundle.join("Contents/marker")).unwrap(),
            "old"
        );
        assert!(reopened.exists());
    }

    #[test]
    fn swap_script_waits_for_the_process() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("Postino.app");
        make_bundle(&bundle, "old");
        let staging = dir.path().join(CONFIG.staging_dir());
        let staged = staging.join("Postino.app");
        make_bundle(&staged, "new");
        let mut sleeper = Command::new("sleep").arg("1").spawn().unwrap();
        let script = macos_swap_script(&CONFIG, sleeper.id(), &bundle, &staged, "true");
        let mut shell = Command::new("/bin/sh")
            .arg("-c")
            .arg(script)
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert_eq!(
            fs::read_to_string(bundle.join("Contents/marker")).unwrap(),
            "old"
        );
        sleeper.wait().unwrap();
        assert!(shell.wait().unwrap().success());
        assert_eq!(
            fs::read_to_string(bundle.join("Contents/marker")).unwrap(),
            "new"
        );
    }

    fn make_archive(dir: &Path) -> PathBuf {
        let src = dir.join("src");
        make_bundle(&src.join("Postino.app"), "new");
        let archive = dir.join("Postino.app.tar.gz");
        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(&src)
            .arg("Postino.app")
            .status()
            .unwrap();
        assert!(status.success());
        archive
    }

    #[test]
    fn stage_unpacks_next_to_the_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let archive = make_archive(dir.path());
        let apps = dir.path().join("apps");
        fs::create_dir(&apps).unwrap();
        // A leftover from a previous attempt is wiped.
        fs::create_dir_all(apps.join(CONFIG.staging_dir())).unwrap();
        fs::write(apps.join(CONFIG.staging_dir()).join("junk"), "x").unwrap();

        let staged = macos_stage(&CONFIG, &archive, &apps.join("Postino.app")).unwrap();

        assert_eq!(staged, apps.join(CONFIG.staging_dir()).join("Postino.app"));
        assert_eq!(
            fs::read_to_string(staged.join("Contents/marker")).unwrap(),
            "new"
        );
        assert!(!apps.join(CONFIG.staging_dir()).join("junk").exists());
    }

    #[test]
    fn stage_fails_without_the_app_in_the_archive() {
        let dir = tempfile::tempdir().unwrap();
        let archive = make_archive(dir.path());
        // Re-pack with another name inside.
        let other = dir.path().join("other");
        fs::create_dir_all(other.join("Other.app")).unwrap();
        fs::write(other.join("Other.app/x"), "x").unwrap();
        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(&other)
            .arg("Other.app")
            .status()
            .unwrap();
        assert!(status.success());
        let apps = dir.path().join("apps");
        fs::create_dir(&apps).unwrap();
        assert!(macos_stage(&CONFIG, &archive, &apps.join("Postino.app")).is_err());
        assert!(!apps.join(CONFIG.staging_dir()).exists());
    }

    #[test]
    fn names_follow_the_app_name() {
        let other = UpdaterConfig {
            app_name: "Vano",
            ..CONFIG
        };
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        make_bundle(&src.join("Vano.app"), "new");
        let archive = dir.path().join("Vano.app.tar.gz");
        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(&src)
            .arg("Vano.app")
            .status()
            .unwrap();
        assert!(status.success());
        let apps = dir.path().join("apps");
        fs::create_dir(&apps).unwrap();

        let staged = macos_stage(&other, &archive, &apps.join("Vano.app")).unwrap();
        assert_eq!(staged, apps.join(".vano-update").join("Vano.app"));
        let script = macos_swap_script(&other, 1, &apps.join("Vano.app"), &staged, "open");
        assert!(script.contains("Vano.old.app"));
        // The Postino archive layout is not accepted by a Vano config.
        assert!(macos_stage(&CONFIG, &archive, &apps.join("Postino.app")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn stage_reports_permission_denied() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let archive = make_archive(dir.path());
        let apps = dir.path().join("apps");
        fs::create_dir(&apps).unwrap();
        fs::set_permissions(&apps, fs::Permissions::from_mode(0o555)).unwrap();
        let result = macos_stage(&CONFIG, &archive, &apps.join("Postino.app"));
        fs::set_permissions(&apps, fs::Permissions::from_mode(0o755)).unwrap();
        match result {
            // Running as root ignores the permissions: nothing to check.
            Ok(_) => {}
            Err(UpdateError::Io(error)) => {
                assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
            }
            Err(other) => panic!("unexpected error: {other}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn detached_script_runs() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("ran");
        spawn_detached_script(&format!("touch {}", quote(&marker.to_string_lossy()))).unwrap();
        for _ in 0..50 {
            if marker.exists() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        panic!("the script did not run");
    }

    /// A tarball with `install.sh` in its top folder, the script being `body`.
    fn fake_tarball(dir: &Path, body: &str) -> PathBuf {
        let top = dir.join("src/postino-0.2.0-linux-x86_64");
        fs::create_dir_all(top.join("bin")).unwrap();
        fs::write(top.join("bin/postino"), "new binary").unwrap();
        fs::write(top.join(LINUX_INSTALL_SCRIPT), body).unwrap();
        let downloads = dir.join("downloads");
        fs::create_dir_all(&downloads).unwrap();
        let tarball = downloads.join("postino-0.2.0-linux-x86_64.tar.gz");
        let status = Command::new("tar")
            .arg("-czf")
            .arg(&tarball)
            .arg("-C")
            .arg(dir.join("src"))
            .arg("postino-0.2.0-linux-x86_64")
            .status()
            .unwrap();
        assert!(status.success());
        tarball
    }

    #[test]
    fn a_script_install_is_told_by_its_list_of_files() {
        let dir = tempfile::tempdir().unwrap();
        let prefix = dir.path().join("local");
        let exe = prefix.join("bin/postino");
        fs::create_dir_all(prefix.join("bin")).unwrap();
        fs::write(&exe, "binary").unwrap();
        assert_eq!(linux_script_prefix(&CONFIG, &exe), None);
        fs::create_dir_all(prefix.join("share/postino")).unwrap();
        fs::write(
            prefix.join("share/postino/installed-files"),
            "# Postino 0.1.0\n",
        )
        .unwrap();
        assert_eq!(linux_script_prefix(&CONFIG, &exe), Some(prefix.clone()));
        // The binary must be in the prefix's bin folder.
        assert_eq!(linux_script_prefix(&CONFIG, &prefix.join("postino")), None);
        assert!(linux_prefix_is_writable(&prefix));
        assert!(!linux_prefix_is_writable(&dir.path().join("missing")));
    }

    #[test]
    fn the_install_script_of_the_tarball_runs_over_the_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let prefix = dir.path().join("my prefix");
        fs::create_dir_all(&prefix).unwrap();
        let tarball = fake_tarball(
            dir.path(),
            "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > \"$4/args\"\necho installed\n",
        );

        linux_run_install_script(&CONFIG, &tarball, &prefix).unwrap();

        let args = fs::read_to_string(prefix.join("args")).unwrap();
        let expected = format!(
            "--tarball\n{}\n--prefix\n{}\n",
            tarball.display(),
            prefix.display()
        );
        assert_eq!(args, expected);
        // The staging folder is gone, the tarball stays for a retry.
        assert!(
            !tarball
                .parent()
                .unwrap()
                .join(CONFIG.staging_dir())
                .exists()
        );
        assert!(tarball.is_file());
    }

    #[test]
    fn a_failing_install_script_reports_its_last_error_line() {
        let dir = tempfile::tempdir().unwrap();
        let tarball = fake_tarball(
            dir.path(),
            "echo 'first' >&2\necho 'error: cannot write to /x' >&2\nexit 1\n",
        );
        let error = linux_run_install_script(&CONFIG, &tarball, dir.path())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("install.sh failed: error: cannot write to /x"),
            "{error}"
        );
    }

    #[test]
    fn a_tarball_without_the_script_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src/postino-0.2.0-linux-x86_64");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("README.md"), "readme").unwrap();
        let tarball = dir.path().join("old.tar.gz");
        let status = Command::new("tar")
            .arg("-czf")
            .arg(&tarball)
            .arg("-C")
            .arg(dir.path().join("src"))
            .arg("postino-0.2.0-linux-x86_64")
            .status()
            .unwrap();
        assert!(status.success());
        assert!(linux_run_install_script(&CONFIG, &tarball, dir.path()).is_err());
    }

    #[test]
    fn the_restart_starts_the_new_file() {
        assert_eq!(
            linux_restart_path(Path::new("/home/me/.local/bin/postino (deleted)")),
            PathBuf::from("/home/me/.local/bin/postino")
        );
        assert_eq!(
            linux_restart_path(Path::new("/home/me/.local/bin/postino")),
            PathBuf::from("/home/me/.local/bin/postino")
        );

        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("started");
        let exe = dir.path().join("new postino");
        fs::write(
            &exe,
            format!(
                "#!/bin/sh\necho yes > {}\n",
                quote(&marker.to_string_lossy())
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&exe, fs::Permissions::from_mode(0o755)).unwrap();
        }
        run_script(&linux_restart_script(exited_pid(), &exe));
        assert_eq!(fs::read_to_string(marker).unwrap().trim(), "yes");
    }
}
