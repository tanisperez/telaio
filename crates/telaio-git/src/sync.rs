//! Talking to a remote: push, pull, fetch and the list of commits a push would send.

use crate::{GitError, Repo};

/// The most commits [`Repo::unpushed_commits`] returns.
const MAX_COMMITS: usize = 100;

/// Where [`Repo::push`] sends the current branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushTarget {
    /// The remote name, for example `origin`.
    pub remote: String,
    /// The branch name on the remote.
    pub branch: String,
    /// Whether the push must also set the upstream, because the branch has none yet.
    pub set_upstream: bool,
}

/// How [`Repo::pull`] joins the remote commits with the local ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullMode {
    /// Honor `pull.rebase`; when it is unset, merge (git refuses divergent branches otherwise).
    FollowConfig,
    /// Always merge.
    Merge,
    /// Always rebase.
    Rebase,
}

/// A commit in a list: its short id and its first line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitSummary {
    /// The short commit id.
    pub oid: String,
    /// The first line of the message.
    pub subject: String,
}

impl Repo {
    /// Where a push of the current branch would go, `None` when `HEAD` is detached or has no
    /// commit, or when no remote can be chosen. A branch with an upstream pushes to it; one
    /// without pushes to `remote.pushDefault`, else `origin`, else the only remote, and sets
    /// the upstream.
    pub fn push_target(&self) -> Result<Option<PushTarget>, GitError> {
        // Two commands, not one per value: under WSL each one costs a round trip to `wsl.exe`.
        let Some(branch) = self.head_branch()? else {
            return Ok(None);
        };
        let config = self.push_config(&branch)?;
        if config.remotes.is_empty() {
            return Ok(None);
        }
        if let (Some(remote), Some(merge)) = (config.branch_remote, config.branch_merge)
            && config.remotes.contains(&remote)
        {
            let name = merge.strip_prefix("refs/heads/").unwrap_or(&merge);
            return Ok(Some(PushTarget {
                remote,
                branch: name.to_string(),
                set_upstream: false,
            }));
        }
        let remotes = &config.remotes;
        let remote = config
            .push_default
            .filter(|r| remotes.contains(r))
            .or_else(|| remotes.iter().find(|r| *r == "origin").cloned())
            .or_else(|| (remotes.len() == 1).then(|| remotes[0].clone()));
        Ok(remote.map(|remote| PushTarget {
            remote,
            branch,
            set_upstream: true,
        }))
    }

    /// Pushes the current branch to `target`. The first push of a branch also sets its upstream.
    pub fn push(&self, target: &PushTarget) -> Result<(), GitError> {
        let refspec = format!("HEAD:refs/heads/{}", target.branch);
        let args: Vec<&str> = if target.set_upstream {
            vec!["push", "--porcelain", "-u", &target.remote, &refspec]
        } else {
            vec!["push", "--porcelain", &target.remote, &refspec]
        };
        self.run(&args).map(|_| ())
    }

    /// Brings in the remote commits of the current branch. A merge never opens an editor.
    pub fn pull(&self, mode: PullMode) -> Result<(), GitError> {
        let merge = match mode {
            PullMode::Merge => true,
            PullMode::Rebase => false,
            PullMode::FollowConfig => self.config("pull.rebase")?.is_none(),
        };
        let args: &[&str] = match (mode, merge) {
            (PullMode::FollowConfig, false) => &["pull"],
            (_, true) => &["pull", "--no-rebase", "--no-edit"],
            (_, false) => &["pull", "--rebase"],
        };
        self.run(args).map(|_| ())
    }

    /// Downloads the remote state without changing the work tree, dropping remote branches that
    /// no longer exist.
    pub fn fetch(&self) -> Result<(), GitError> {
        self.run(&["fetch", "--quiet", "--prune"]).map(|_| ())
    }

    /// The commits a push to `target` would send, newest first, at most 100. Without an upstream
    /// those are the commits of `HEAD` that no branch of the remote has.
    pub fn unpushed_commits(&self, target: &PushTarget) -> Result<Vec<CommitSummary>, GitError> {
        let remotes = format!("--remotes={}", target.remote);
        let limit = format!("--max-count={MAX_COMMITS}");
        let mut args = vec!["log", "-z", "--format=%h%x1f%s", &limit];
        if target.set_upstream {
            args.extend(["HEAD", "--not", &remotes]);
        } else {
            args.push("@{u}..HEAD");
        }
        // A target comes from `push_target`, so `HEAD` has a commit: only a failure pays the
        // check of a repository without commits.
        let output = match self.run(&args) {
            Ok(output) => output,
            Err(GitError::Failed { .. }) if !self.has_head()? => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };
        let text = String::from_utf8_lossy(&output);
        Ok(text
            .split('\0')
            .filter_map(|record| record.trim_start_matches('\n').split_once('\x1f'))
            .map(|(oid, subject)| CommitSummary {
                oid: oid.to_string(),
                subject: subject.to_string(),
            })
            .collect())
    }

    /// The checked out branch, `None` when `HEAD` is detached.
    pub(crate) fn current_branch(&self) -> Result<Option<String>, GitError> {
        match self.run(&["symbolic-ref", "--quiet", "--short", "HEAD"]) {
            Ok(output) => Ok(Some(String::from_utf8_lossy(&output).trim().to_string())),
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// The branch `HEAD` is on, `None` when it is detached or has no commit yet. One command
    /// for what `has_head` and `current_branch` tell apart.
    fn head_branch(&self) -> Result<Option<String>, GitError> {
        match self.run(&[
            "rev-parse",
            "--verify",
            "--quiet",
            "--symbolic-full-name",
            "HEAD",
        ]) {
            Ok(output) => {
                let name = String::from_utf8_lossy(&output).trim().to_string();
                // A detached `HEAD` prints `HEAD`.
                Ok(name.strip_prefix("refs/heads/").map(str::to_string))
            }
            // Exit code 1: `HEAD` has no commit.
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// What [`Self::push_target`] reads from the configuration of `branch`, in one command.
    fn push_config(&self, branch: &str) -> Result<PushConfig, GitError> {
        let pattern = format!(
            r"^(remote\..*\.(url|pushurl|fetch)|branch\.{}\.(remote|merge)|remote\.pushdefault)$",
            escape_regex(branch)
        );
        match self.run(&["config", "-z", "--get-regexp", &pattern]) {
            Ok(output) => Ok(PushConfig::parse(&String::from_utf8_lossy(&output), branch)),
            // Exit code 1: no key matches.
            Err(GitError::Failed { code: Some(1), .. }) => Ok(PushConfig::default()),
            Err(err) => Err(err),
        }
    }
}

/// The configuration [`Repo::push_target`] needs.
#[derive(Debug, Default, PartialEq)]
struct PushConfig {
    /// The remotes, in the order they are configured.
    remotes: Vec<String>,
    /// `branch.<branch>.remote`.
    branch_remote: Option<String>,
    /// `branch.<branch>.merge`.
    branch_merge: Option<String>,
    /// `remote.pushDefault`.
    push_default: Option<String>,
}

impl PushConfig {
    /// Reads the output of `git config -z --get-regexp` for `branch`: `key\nvalue` records, each
    /// ended by NUL. Section and key names come lowercased, the branch and remote names as
    /// written. A later value of a key wins, as with `git config --get`.
    fn parse(output: &str, branch: &str) -> Self {
        let mut config = Self::default();
        let branch_prefix = format!("branch.{branch}.");
        for record in output.split('\0').filter(|record| !record.is_empty()) {
            let (key, value) = record.split_once('\n').unwrap_or((record, ""));
            let value = value.to_string();
            if key == "remote.pushdefault" {
                config.push_default = Some(value).filter(|v| !v.is_empty());
            } else if let Some(name) = key.strip_prefix(&branch_prefix) {
                let value = Some(value).filter(|v| !v.is_empty());
                match name {
                    "remote" => config.branch_remote = value,
                    "merge" => config.branch_merge = value,
                    _ => {}
                }
            } else if let Some(rest) = key.strip_prefix("remote.") {
                let name = [".url", ".pushurl", ".fetch"]
                    .iter()
                    .find_map(|suffix| rest.strip_suffix(suffix));
                if let Some(name) = name
                    && !config.remotes.iter().any(|known| known == name)
                {
                    config.remotes.push(name.to_string());
                }
            }
        }
        config
    }
}

/// `text` with the characters that mean something in an extended regular expression escaped, so
/// it matches only itself (branch names may hold `.` or `+`).
fn escape_regex(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        if "\\.^$|?*+()[]{}".contains(c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{PushConfig, escape_regex};

    #[test]
    fn the_push_config_is_read_from_one_listing() {
        let output = "remote.origin.url\n/x\0remote.origin.fetch\n+refs/heads/*:refs/remotes/origin/*\0\
                      remote.my.re.url\n/y\0branch.feat/a+b.merge\nrefs/heads/main\0\
                      branch.feat/a+b.remote\norigin\0remote.pushdefault\nmy.re\0";
        assert_eq!(
            PushConfig::parse(output, "feat/a+b"),
            PushConfig {
                remotes: vec!["origin".into(), "my.re".into()],
                branch_remote: Some("origin".into()),
                branch_merge: Some("refs/heads/main".into()),
                push_default: Some("my.re".into()),
            }
        );
        assert_eq!(PushConfig::parse("", "main"), PushConfig::default());
    }

    #[test]
    fn regex_characters_of_a_branch_are_escaped() {
        assert_eq!(escape_regex("feat/a+b.c"), r"feat/a\+b\.c");
        assert_eq!(escape_regex("main"), "main");
    }
}
