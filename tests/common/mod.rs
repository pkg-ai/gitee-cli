//! Shared helpers for integration tests.
//!
//! Every `tests/*_cli.rs` file declares `mod common;` and imports from here so
//! the command-wiring, git-fixture, and mock-payload scaffolding lives in one
//! place instead of being copied into each file.
#![allow(dead_code)]
use std::path::Path;
use std::process::{Command as ProcessCommand, Output};

use assert_cmd::Command;
use httpmock::MockServer;
use serde_json::{Value, json};
use tempfile::TempDir;

/// Spawn a fresh `gitee` CLI command (use `.env(...)` / `.current_dir(...)` /
/// `.args(...)` on it as needed).
pub fn cmd() -> Command {
    Command::cargo_bin("gitee").unwrap()
}

/// Assert a command exited successfully with empty stderr.
pub fn assert_ok(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "expected success, got {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "expected empty stderr, got:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Run `gitee args` against a mock server, assert success, and return parsed JSON stdout.
pub fn run_json(server: &MockServer, args: &[&str]) -> Value {
    let output = cmd()
        .env("GITEE_BASE_URL", server.base_url())
        .args(args)
        .output()
        .unwrap();
    assert_ok(&output);
    serde_json::from_slice(&output.stdout).unwrap()
}

/// Run `gitee args` (no mock server) and return the raw output without asserting.
pub fn run(args: &[&str]) -> Output {
    cmd().args(args).output().unwrap()
}

/// Parse a command's stdout as JSON.
pub fn parse_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

// --- git fixtures -----------------------------------------------------------

pub fn git_repo_without_remote(branch: &str) -> TempDir {
    let repo_dir = TempDir::new().unwrap();
    run_git(repo_dir.path(), &["init"]);
    run_git(repo_dir.path(), &["checkout", "-b", branch]);
    repo_dir
}

pub fn git_repo_with_remote(remote_url: &str, branch: &str) -> TempDir {
    let repo_dir = git_repo_without_remote(branch);
    run_git(repo_dir.path(), &["remote", "add", "origin", remote_url]);
    repo_dir
}

pub fn git_repo_with_remote_and_commit(remote_url: &str, branch: &str) -> TempDir {
    let repo_dir = TempDir::new().unwrap();
    run_git(repo_dir.path(), &["init"]);
    std::fs::write(repo_dir.path().join("README.md"), "hello\n").unwrap();
    run_git(repo_dir.path(), &["add", "README.md"]);
    run_git(
        repo_dir.path(),
        &[
            "-c",
            "user.name=Test User",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "init",
        ],
    );
    run_git(repo_dir.path(), &["checkout", "-b", branch]);
    run_git(repo_dir.path(), &["remote", "add", "origin", remote_url]);
    repo_dir
}

pub fn git_repo_with_detached_head(remote_url: &str) -> TempDir {
    let repo_dir = git_repo_with_remote_and_commit(remote_url, "feature/detached");
    run_git(repo_dir.path(), &["checkout", "--detach"]);
    repo_dir
}

pub fn set_branch_upstream(repo_dir: &Path, branch: &str, remote: &str, remote_branch: &str) {
    run_git(
        repo_dir,
        &["config", &format!("branch.{branch}.remote"), remote],
    );
    run_git(
        repo_dir,
        &[
            "config",
            &format!("branch.{branch}.merge"),
            &format!("refs/heads/{remote_branch}"),
        ],
    );
    run_git(
        repo_dir,
        &[
            "update-ref",
            &format!("refs/remotes/{remote}/{remote_branch}"),
            "HEAD",
        ],
    );
}

pub fn run_git(repo_dir: &Path, args: &[&str]) {
    let output = ProcessCommand::new("git")
        .args(args)
        .current_dir(repo_dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git command failed: git {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// --- mock payload builders --------------------------------------------------

/// A canonical pull-request API response.
pub fn pull_request_payload(number: u64) -> Value {
    json!({
        "number": number,
        "state": "open",
        "title": format!("Pull request #{number}"),
        "body": "PR body",
        "html_url": format!("https://gitee.com/octo/demo/pulls/{number}"),
        "draft": false,
        "mergeable": true,
        "created_at": "2026-03-20T09:00:00+08:00",
        "updated_at": "2026-03-20T10:00:00+08:00",
        "merged_at": null,
        "user": { "login": "octocat" },
        "head": {
            "ref": "feature/pr",
            "sha": "abc123",
            "repo": { "full_name": "octo/demo" }
        },
        "base": {
            "ref": "main",
            "sha": "def456",
            "repo": { "full_name": "octo/demo" }
        }
    })
}

/// A canonical repository API response.
pub fn repository_payload() -> Value {
    json!({
        "full_name": "octo/demo",
        "path": "demo",
        "html_url": "https://gitee.com/octo/demo",
        "ssh_url": "git@gitee.com:octo/demo.git",
        "clone_url": "https://gitee.com/octo/demo.git",
        "fork": false,
        "default_branch": "main"
    })
}

/// A canonical issue API response.
pub fn issue_payload(number: &str) -> Value {
    json!({
        "number": number,
        "title": format!("Issue #{number}"),
        "state": "open",
        "body": "Issue body",
        "html_url": format!("https://gitee.com/octo/demo/issues/{number}"),
        "comments": 0,
        "created_at": "2026-03-20T09:00:00+08:00",
        "updated_at": "2026-03-20T10:00:00+08:00",
        "user": { "login": "octocat" }
    })
}

/// A canonical PR-comment API response.
pub fn pull_request_comment_payload(id: u64) -> Value {
    json!({
        "id": id,
        "body": "Comment body",
        "html_url": format!("https://gitee.com/octo/demo/pulls/42#note_{id}"),
        "created_at": "2026-03-20T12:30:00+08:00",
        "updated_at": "2026-03-20T12:31:00+08:00",
        "user": { "login": "carol" }
    })
}

/// A canonical issue-comment API response.
pub fn issue_comment_payload(id: u64) -> Value {
    json!({
        "id": id,
        "body": "Comment body",
        "html_url": format!("https://gitee.com/octo/demo/issues/1#note_{id}"),
        "created_at": "2026-03-20T12:30:00+08:00",
        "updated_at": "2026-03-20T12:31:00+08:00",
        "user": { "login": "carol" }
    })
}
