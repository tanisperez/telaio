//! Long-lived `wsl.exe` processes that run git commands sent over their standard input (telaio
//! issue #46).
//!
//! Starting `wsl.exe` costs far more than the git command it runs, and an app runs several git
//! commands per user action. So [`crate::run::run_with`] does not start one `wsl.exe` per command
//! under [`Runner::Wsl`]: it sends the command to a small shell loop that stays running inside the
//! distribution, and only the first command pays the start.
//!
//! A new session prints a greeting line before its first request: a session that does not (an
//! unknown distribution, no `sh`) never takes a command.
//!
//! The loop is POSIX `sh` plus `head`, `wc`, `mktemp` and `wslpath`, all present in every WSL
//! distribution. Each request is length prefixed, so arguments and input can hold any byte but
//! NUL (which no command line can hold either):
//!
//! ```text
//! request:  "<dir length> <arg count> <stdin length>\n" <dir>
//!           then per argument "<length>\n" <argument>, then <stdin>
//! response: "<exit code> <stdout length> <stderr length>\n" <stdout> <stderr>
//! ```
//!
//! The directory is a Linux path, or a Windows one that `wslpath -u` translates, as `wsl.exe
//! --cd` did. The loop runs each command in a subshell, with its output in temporary files, so
//! a command's output can never be mixed with the protocol.
//!
//! Sessions are kept in a pool per launcher, distribution and git program: a command takes an
//! idle one or starts a new one, so commands from several threads still run in parallel. A
//! session that fails is dropped. When the failure happens before the request reached it, the
//! command runs again with a one-off `wsl.exe`; after that point it is reported as an error,
//! since running it twice could commit or push twice.
//!
//! Setting `TELAIO_GIT_WSL_SESSION=0` turns sessions off, back to one `wsl.exe` per command.

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Mutex, OnceLock};

use crate::run::{Output, Runner};
use crate::wsl;

/// The shell loop each session runs, see the module docs.
const LOOP: &str = r#"rm -f "$f"
t=$(mktemp -d) || exit 1
trap 'rm -rf "$t"' EXIT
echo telaio-git-session
take() { v=$(head -c "$1"; printf x); v=${v%x}; }
while IFS=' ' read -r dl n il; do
  take "$dl"; dir=$v
  set --
  i=0
  while [ "$i" -lt "$n" ]; do
    IFS= read -r al; take "$al"; set -- "$@" "$v"; i=$((i + 1))
  done
  head -c "$il" >"$t/in"
  case $dir in
    '' | /*) ;;
    *) dir=$(wslpath -u "$dir" 2>/dev/null || printf '%s' "$dir") ;;
  esac
  (if [ -n "$dir" ]; then cd "$dir" || exit 128; fi; exec "$@") <"$t/in" >"$t/out" 2>"$t/err"
  c=$?
  printf '%s %s %s\n' "$c" "$(wc -c <"$t/out")" "$(wc -c <"$t/err")"
  cat "$t/out" "$t/err"
done
"#;

/// The line the loop prints once it is ready.
const GREETING: &str = "telaio-git-session";

/// The environment variable that turns sessions off when set to `0`.
const DISABLE_VARIABLE: &str = "TELAIO_GIT_WSL_SESSION";

/// What identifies the sessions a [`Runner::Wsl`] can use.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    launcher: PathBuf,
    distro: String,
}

/// One running `wsl.exe` with the loop.
struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Why a command did not get a response from a session.
pub(crate) enum SessionError {
    /// The request did not reach the session (no session could start, or writing failed): the
    /// command did not run and can run another way.
    NotSent(io::Error),
    /// The request was sent but the response is missing or malformed: the command may have run.
    Lost(io::Error),
}

/// The idle sessions, per launcher and distribution.
fn pool() -> &'static Mutex<HashMap<Key, Vec<Session>>> {
    static POOL: OnceLock<Mutex<HashMap<Key, Vec<Session>>>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Whether commands go through sessions: unless `TELAIO_GIT_WSL_SESSION` is `0`.
pub(crate) fn enabled() -> bool {
    std::env::var(DISABLE_VARIABLE).map_or(true, |value| value.trim() != "0")
}

/// Runs `program args` in `cwd` through a session of `launcher` and `distro`, with `stdin` as
/// its standard input.
pub(crate) fn run(
    launcher: &Path,
    distro: &str,
    program: &str,
    cwd: Option<&Path>,
    args: &[&str],
    stdin: Option<&[u8]>,
) -> Result<Output, SessionError> {
    let key = Key {
        launcher: launcher.to_path_buf(),
        distro: distro.to_string(),
    };
    let idle = pool()
        .lock()
        .ok()
        .and_then(|mut pool| pool.get_mut(&key).and_then(Vec::pop));
    let mut session = match idle {
        Some(session) => session,
        None => start(&key).map_err(SessionError::NotSent)?,
    };
    let request = encode_request(cwd.map(wsl::cd_argument).as_deref(), program, args, stdin);
    session
        .stdin
        .write_all(&request)
        .and_then(|()| session.stdin.flush())
        .map_err(SessionError::NotSent)?;
    let output = read_response(&mut session.stdout).map_err(SessionError::Lost)?;
    if let Ok(mut pool) = pool().lock() {
        pool.entry(key).or_default().push(session);
    }
    Ok(output)
}

/// Starts a session. The loop is too long to pass as an argument through `wsl.exe` safely, so a
/// short bootstrap reads it from the standard input, the exact number of bytes, into a file it
/// then sources.
fn start(key: &Key) -> io::Result<Session> {
    let bootstrap = format!("f=$(mktemp) && head -c {} >$f && . $f", LOOP.len());
    let mut cmd = Command::new(&key.launcher);
    cmd.args(["-d", key.distro.as_str(), "--exec"])
        // The same environment one-off commands get, see `Runner::command`.
        .args([
            "env",
            "-u",
            "LC_ALL",
            "-u",
            "LANGUAGE",
            "GIT_TERMINAL_PROMPT=0",
            "LC_MESSAGES=C",
            "sh",
            "-c",
            bootstrap.as_str(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW, as in `run::run_with`.
        cmd.creation_flags(0x0800_0000);
    }
    let mut child = cmd.spawn()?;
    let (Some(mut stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(io::Error::other("the session has no pipes"));
    };
    if let Err(error) = stdin.write_all(LOOP.as_bytes()) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let mut session = Session {
        child,
        stdin,
        stdout: BufReader::new(stdout),
    };
    let mut greeting = String::new();
    session.stdout.read_line(&mut greeting)?;
    if greeting.trim_end() != GREETING {
        return Err(io::Error::other("the WSL session did not start"));
    }
    log::debug!("git: started a WSL session for {}", key.distro);
    Ok(session)
}

/// The bytes of one request, see the module docs.
fn encode_request(
    dir: Option<&str>,
    program: &str,
    args: &[&str],
    stdin: Option<&[u8]>,
) -> Vec<u8> {
    let dir = dir.unwrap_or_default();
    let stdin = stdin.unwrap_or_default();
    let mut request = format!("{} {} {}\n", dir.len(), args.len() + 1, stdin.len()).into_bytes();
    request.extend_from_slice(dir.as_bytes());
    for arg in std::iter::once(&program).chain(args) {
        request.extend_from_slice(format!("{}\n", arg.len()).as_bytes());
        request.extend_from_slice(arg.as_bytes());
    }
    request.extend_from_slice(stdin);
    request
}

/// Reads one response, see the module docs.
fn read_response(reader: &mut impl BufRead) -> io::Result<Output> {
    let mut header = String::new();
    if reader.read_line(&mut header)? == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "the WSL session ended",
        ));
    }
    let malformed = || io::Error::new(io::ErrorKind::InvalidData, format!("bad header {header:?}"));
    let numbers: Vec<i64> = header
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| malformed())?;
    let [code, out_len, err_len] = numbers[..] else {
        return Err(malformed());
    };
    let (Ok(out_len), Ok(err_len)) = (usize::try_from(out_len), usize::try_from(err_len)) else {
        return Err(malformed());
    };
    let mut stdout = vec![0; out_len];
    reader.read_exact(&mut stdout)?;
    let mut stderr = vec![0; err_len];
    reader.read_exact(&mut stderr)?;
    Ok(Output {
        // A shell reports a command killed by a signal as 128 plus the signal, never "no code".
        code: i32::try_from(code).ok(),
        stdout,
        stderr,
    })
}

/// Runs `program args` the way [`run`] does when `runner` is a WSL one and sessions are on.
/// `None` for a native runner, or when sessions are off.
pub(crate) fn run_for(
    runner: &Runner,
    cwd: Option<&Path>,
    args: &[&str],
    stdin: Option<&[u8]>,
) -> Option<Result<Output, SessionError>> {
    match runner {
        Runner::Wsl {
            launcher,
            distro,
            program,
        } if enabled() => Some(run(launcher, distro, program, cwd, args, stdin)),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_request_prefixes_every_part_with_its_length() {
        let request = encode_request(Some("/repo"), "git", &["log", "a b\n"], Some(b"in"));
        assert_eq!(request, b"5 3 2\n/repo3\ngit3\nlog4\na b\nin".to_vec());
        assert_eq!(
            encode_request(None, "git", &[], None),
            b"0 1 0\n3\ngit".to_vec()
        );
    }

    #[test]
    fn a_response_is_read_by_its_lengths() {
        let mut reader = io::Cursor::new(b"1 3 4\nouterr\n2 0 0\n".to_vec());
        let first = read_response(&mut reader).unwrap();
        assert_eq!(first.code, Some(1));
        assert_eq!(first.stdout, b"out");
        assert_eq!(first.stderr, b"err\n");
        let second = read_response(&mut reader).unwrap();
        assert_eq!(second.code, Some(2));
        assert!(second.stdout.is_empty());
        assert!(read_response(&mut reader).is_err(), "the end is an error");
    }

    #[test]
    fn a_malformed_header_is_an_error() {
        assert!(read_response(&mut io::Cursor::new(b"oops\n".to_vec())).is_err());
        assert!(read_response(&mut io::Cursor::new(b"0 -1 0\n".to_vec())).is_err());
        assert!(read_response(&mut io::Cursor::new(b"0 9 0\nshort".to_vec())).is_err());
    }

    /// A stand-in for `wsl.exe` like the one of `wsl.rs`, that also counts its starts in
    /// `starts` and takes `delay` seconds to start, as the real one does.
    #[cfg(unix)]
    fn fake_wsl(dir: &Path, delay: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let script = format!(
            "#!/bin/sh\nsleep {delay}\necho >>'{}'\n[ \"$1\" = -d ] && [ \"$2\" = Test ] || exit 1\n\
             shift 2\n[ \"$1\" = --exec ] || exit 93\nshift\nexec \"$@\"\n",
            dir.join("starts").display()
        );
        let path = dir.join("wsl.exe");
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[cfg(unix)]
    fn starts(dir: &Path) -> usize {
        std::fs::read_to_string(dir.join("starts"))
            .map(|text| text.lines().count())
            .unwrap_or(0)
    }

    /// `sh -c SCRIPT` through a session, so the test does not depend on git.
    #[cfg(unix)]
    fn sh(
        launcher: &Path,
        cwd: Option<&Path>,
        script: &str,
        args: &[&str],
        stdin: Option<&[u8]>,
    ) -> Output {
        let mut all = vec!["-c", script, "sh"];
        all.extend_from_slice(args);
        match run(launcher, "Test", "sh", cwd, &all, stdin) {
            Ok(output) => output,
            Err(SessionError::NotSent(error) | SessionError::Lost(error)) => panic!("{error}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_session_runs_any_bytes_and_starts_once() {
        let tools = tempfile::tempdir().unwrap();
        let launcher = fake_wsl(tools.path(), "0");

        // Arguments with spaces, new lines, a trailing new line and no text at all.
        let output = sh(
            &launcher,
            None,
            r#"for a in "$@"; do printf '[%s]' "$a"; done"#,
            &["a b", "line\nnext\n", "", "ñandú"],
            None,
        );
        assert_eq!(output.code, Some(0));
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "[a b][line\nnext\n][][ñandú]"
        );

        // The input, with every byte value but NUL and without a final new line.
        let payload: Vec<u8> = (1..=255).collect();
        let output = sh(&launcher, None, "cat", &[], Some(&payload));
        assert_eq!(output.stdout, payload);

        // A failure keeps its code and its standard error apart.
        let output = sh(&launcher, None, "echo out; echo err >&2; exit 3", &[], None);
        assert_eq!(output.code, Some(3));
        assert_eq!(output.stdout, b"out\n");
        assert_eq!(output.stderr, b"err\n");

        // The working directory, with a space in it; and a missing one fails like `cd`.
        let work = tempfile::tempdir().unwrap();
        let dir = work.path().join("my repo");
        std::fs::create_dir(&dir).unwrap();
        let output = sh(&launcher, Some(&dir), "pwd", &[], None);
        let printed = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            Path::new(printed.trim_end()),
            dunce::canonicalize(&dir).unwrap()
        );
        let output = sh(&launcher, Some(&dir.join("gone")), "pwd", &[], None);
        assert_eq!(output.code, Some(128));

        assert_eq!(starts(tools.path()), 1, "one wsl.exe for every command");
    }

    #[cfg(unix)]
    #[test]
    fn parallel_commands_get_sessions_of_their_own() {
        let tools = tempfile::tempdir().unwrap();
        let launcher = fake_wsl(tools.path(), "0");
        std::thread::scope(|scope| {
            for n in 0..4 {
                let launcher = &launcher;
                scope.spawn(move || {
                    let text = n.to_string();
                    let output = sh(
                        launcher,
                        None,
                        "sleep 0.2; printf %s \"$1\"",
                        &[&text],
                        None,
                    );
                    assert_eq!(output.stdout, text.as_bytes());
                });
            }
        });
        let started = starts(tools.path());
        assert!((2..=4).contains(&started), "{started} sessions");
        // Later commands reuse them.
        sh(&launcher, None, "true", &[], None);
        assert_eq!(starts(tools.path()), started);
    }

    #[cfg(unix)]
    #[test]
    fn a_launcher_that_cannot_start_a_session_sends_nothing() {
        let tools = tempfile::tempdir().unwrap();
        let launcher = fake_wsl(tools.path(), "0");
        match run(&launcher, "Nope", "sh", None, &["-c", "true"], None) {
            Err(SessionError::NotSent(_)) => {}
            Err(SessionError::Lost(error)) => panic!("lost: {error}"),
            Ok(_) => panic!("ran in an unknown distribution"),
        }
    }
}
