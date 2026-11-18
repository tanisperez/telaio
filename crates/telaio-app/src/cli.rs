//! The pieces every app of the family shares on the command line: the `<App> X.Y.Z` version line,
//! the common help header and footer, the standard exit codes and, on Windows, detaching the
//! window from the terminal a console subsystem binary was started from.
//! Parsing stays in each app, which adds its own commands.

use crate::AppIdentity;

/// Exit code of a command that did what it was asked.
pub const EXIT_SUCCESS: u8 = 0;

/// Exit code of a command line that could not be understood (unknown option, missing value).
pub const EXIT_USAGE: u8 = 2;

/// `<App> X.Y.Z`, the first line of `--version` and of `--help`.
pub fn version_line(identity: &AppIdentity) -> String {
    format!("{} {}", identity.name, identity.version)
}

/// The top of a `--help` text: the version line followed by the one line description.
pub fn help_header(identity: &AppIdentity, tagline: &str) -> String {
    format!("{}\n{tagline}\n", version_line(identity))
}

/// The bottom of a `--help` text: the link to the command line documentation.
pub fn help_footer(docs_url: &str) -> String {
    format!("Documentation: {docs_url}\n")
}

/// How the app goes on after [`detach_from_console`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiLaunch {
    /// This process opens the window.
    Continue,
    /// A detached copy of this process opens the window: this one exits with success at once.
    Relaunched,
}

/// What a console subsystem app does before it opens its window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))]
enum ConsolePlan {
    /// Nothing holds it: no console and no inherited standard handles. Started from Explorer,
    /// the Start menu or a shortcut on Windows 11 24H2 or later, where the
    /// `consoleAllocationPolicy` of the manifest is `detached`, or the detached copy itself.
    Keep,
    /// A console or inherited handles: started from a terminal (cmd, PowerShell, WSL) that
    /// waits for it, or by an older Windows that opened a console for it. Relaunch detached, so
    /// the terminal gets its prompt back at once and closing it does not close the window.
    Relaunch,
}

/// The plan for a process attached to a console shared by `processes` processes
/// (`GetConsoleProcessList`, 0 without a console), with `inherited_io` when any of its standard
/// handles is a file, a pipe or a console (not `NUL`, which the detached copy gets).
#[cfg_attr(not(windows), allow(dead_code))]
fn console_plan(processes: u32, inherited_io: bool) -> ConsolePlan {
    if processes == 0 && !inherited_io {
        ConsolePlan::Keep
    } else {
        ConsolePlan::Relaunch
    }
}

/// Call it before opening the window of a console subsystem Windows binary, after the command
/// line commands had their chance to run. A console binary (rather than `windows_subsystem =
/// "windows"`) gets the terminal's console, pipes and redirections from the shell, which waits
/// for it and reads its exit code. The window, though, must not hold the terminal:
///
/// - With nothing holding it (a double click on Windows 11 24H2 or later, with
///   `consoleAllocationPolicy` set to `detached` in the manifest) it does nothing.
/// - Otherwise it starts a detached copy of itself with the same arguments, with `NUL` as its
///   standard handles, and returns [`GuiLaunch::Relaunched`]: the caller exits with success.
///   On an older Windows, the console it opened for a double click flashes for an instant.
/// - When the copy cannot start, it frees the console and goes on in this process.
///
/// Other platforms always continue.
#[cfg(windows)]
#[allow(unsafe_code)]
pub fn detach_from_console() -> GuiLaunch {
    use std::os::windows::process::CommandExt as _;
    use std::process::{Command, Stdio};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_TYPE_CHAR, FILE_TYPE_DISK, FILE_TYPE_PIPE, GetFileType,
    };
    use windows_sys::Win32::System::Console::{
        FreeConsole, GetConsoleMode, GetConsoleProcessList, GetStdHandle, STD_ERROR_HANDLE,
        STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };

    // Process creation flags (`winbase.h`): no console for the copy, and Ctrl+C in the terminal
    // does not reach it.
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

    let mut ids = [0u32; 2];
    // SAFETY: plain Win32 queries. `GetConsoleProcessList` writes at most `ids.len()` ids into
    // the local buffer and returns the full count, or 0 without a console; `GetConsoleMode`
    // writes one local `u32`. An invalid or null handle only makes a query fail.
    let (processes, inherited_io) = unsafe {
        let processes = GetConsoleProcessList(ids.as_mut_ptr(), ids.len() as u32);
        let inherited_io = [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE]
            .into_iter()
            .any(|id| {
                let handle = GetStdHandle(id);
                let mut mode = 0;
                match GetFileType(handle) {
                    FILE_TYPE_DISK | FILE_TYPE_PIPE => true,
                    FILE_TYPE_CHAR => GetConsoleMode(handle, &mut mode) != 0,
                    _ => false,
                }
            });
        (processes, inherited_io)
    };
    match console_plan(processes, inherited_io) {
        ConsolePlan::Keep => GuiLaunch::Continue,
        ConsolePlan::Relaunch => {
            let copy = std::env::current_exe().and_then(|exe| {
                Command::new(exe)
                    .args(std::env::args_os().skip(1))
                    .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
            });
            if copy.is_ok() {
                return GuiLaunch::Relaunched;
            }
            // SAFETY: detaches this process from its console; nothing of ours points into it.
            unsafe {
                FreeConsole();
            }
            GuiLaunch::Continue
        }
    }
}

/// Other platforms have no console subsystem: the window always opens in this process.
#[cfg(not(windows))]
pub fn detach_from_console() -> GuiLaunch {
    GuiLaunch::Continue
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const IDENTITY: AppIdentity = AppIdentity {
        name: "App",
        slug: "app",
        app_id: "test.app",
        version: "1.2.3",
        env_prefix: "APP",
        repository: "https://example.test/app",
    };

    #[test]
    fn the_version_line_is_the_name_and_the_version() {
        assert_eq!(version_line(&IDENTITY), "App 1.2.3");
    }

    #[test]
    fn the_help_header_and_footer_end_in_a_newline() {
        assert_eq!(
            help_header(&IDENTITY, "Does things."),
            "App 1.2.3\nDoes things.\n"
        );
        assert_eq!(
            help_footer("https://x.test"),
            "Documentation: https://x.test\n"
        );
    }

    #[test]
    fn the_window_relaunches_detached_unless_nothing_holds_it() {
        // A double click on Windows 11 24H2, or the detached copy with NUL handles.
        assert_eq!(console_plan(0, false), ConsolePlan::Keep);
        // Pipes without a console (WSL, a script), a terminal, an older Windows' own console.
        assert_eq!(console_plan(0, true), ConsolePlan::Relaunch);
        assert_eq!(console_plan(2, true), ConsolePlan::Relaunch);
        assert_eq!(console_plan(1, false), ConsolePlan::Relaunch);
    }

    #[test]
    fn the_exit_codes_are_the_conventional_ones() {
        assert_eq!(EXIT_SUCCESS, 0);
        assert_eq!(EXIT_USAGE, 2);
    }
}
