//! Machine-readable `gitee help --json` rendering.
//!
//! Every command's name, summary, flags, and arguments already live in its clap
//! [`Command`] definition, so this module derives those parts from clap instead
//! of transcribing them a second time. Only the extra agent-facing metadata
//! (gh-equivalent, auth requirements, examples, notes) is kept in a compact
//! table keyed by command path.

use clap::{Arg, Command};
use serde_json::{Value, json};

use crate::cli::json_field_selection_for_help;

/// Agent-facing metadata for a leaf command, keyed by dotted path (e.g. `pr view`).
struct LeafMeta {
    gh_equivalent: &'static str,
    auth: &'static str,
    repo_flag: bool,
    repo_inference: bool,
    local_git_required: bool,
    input_sources: &'static [&'static str],
    examples: &'static [&'static str],
    notes: &'static [&'static str],
}

const fn leaf_meta(gh: &'static str, auth: &'static str) -> LeafMeta {
    LeafMeta {
        gh_equivalent: gh,
        auth,
        repo_flag: false,
        repo_inference: false,
        local_git_required: false,
        input_sources: &[],
        examples: &[],
        notes: &[],
    }
}

/// The per-command metadata table.
static LEAF_META: &[(&str, LeafMeta)] = &[
    (
        "auth status",
        LeafMeta {
            gh_equivalent: "gh auth status",
            auth: "not_required",
            examples: &["gitee auth status", "gitee auth status --json"],
            notes: &["Reads the token from GITEE_TOKEN first, then the saved config file."],
            ..leaf_meta("", "")
        },
    ),
    (
        "auth login",
        LeafMeta {
            gh_equivalent: "gh auth login",
            auth: "not_required",
            input_sources: &["--token", "--with-token (stdin)"],
            examples: &[
                "gitee auth login --token \"$GITEE_TOKEN\" --json",
                "printf '%s\\n' \"$TOKEN\" | gitee auth login --with-token --json",
            ],
            notes: &["Provide exactly one of --token or --with-token."],
            ..leaf_meta("", "")
        },
    ),
    (
        "auth logout",
        LeafMeta {
            gh_equivalent: "gh auth logout",
            auth: "not_required",
            examples: &["gitee auth logout --json"],
            notes: &["Clears the saved config token but does not unset GITEE_TOKEN."],
            ..leaf_meta("", "")
        },
    ),
    (
        "issue list",
        LeafMeta {
            gh_equivalent: "gh issue list",
            auth: "optional",
            repo_flag: true,
            repo_inference: true,
            examples: &[
                "gitee issue list --repo octo/demo --state open --json",
                "gitee issue list --state open --page 1 --per-page 20 --json",
                "gitee issue list --repo octo/demo --json number,title,url",
            ],
            notes: &[
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "issue view",
        LeafMeta {
            gh_equivalent: "gh issue view",
            auth: "optional",
            repo_flag: true,
            repo_inference: true,
            examples: &[
                "gitee issue view I123 --repo octo/demo --json",
                "gitee issue view I123 --comments --page 1 --per-page 20 --json",
            ],
            notes: &["Comments are fetched only when --comments is provided."],
            ..leaf_meta("", "")
        },
    ),
    (
        "issue comment",
        LeafMeta {
            gh_equivalent: "gh issue comment",
            auth: "required",
            repo_flag: true,
            repo_inference: true,
            input_sources: &["--body", "--body-file"],
            examples: &[
                "gitee issue comment I123 --repo octo/demo --body \"Thanks for the report\" --json",
                "gitee issue comment I123 --body-file ./comment.md --json",
            ],
            notes: &[
                "Provide exactly one of --body or --body-file.",
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "issue create",
        LeafMeta {
            gh_equivalent: "gh issue create",
            auth: "required",
            repo_flag: true,
            repo_inference: true,
            input_sources: &["--body", "--body-file"],
            examples: &[
                "gitee issue create --repo octo/demo --title \"New bug\" --body \"Steps to reproduce\" --json",
                "gitee issue create --title \"New bug\" --body-file ./issue.md --json",
            ],
            notes: &[
                "--title is required.",
                "Provide at most one of --body or --body-file.",
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "issue edit",
        LeafMeta {
            gh_equivalent: "gh issue edit",
            auth: "required",
            repo_flag: true,
            repo_inference: true,
            input_sources: &["--body", "--body-file"],
            examples: &[
                "gitee issue edit I123 --repo octo/demo --title \"Updated title\" --json",
                "gitee issue edit I123 --body-file ./body.md --state closed --json",
                "gitee issue edit I123 --repo octo/demo --state open --json number,title,url",
            ],
            notes: &[
                "Provide at least one of --title, --body, --body-file, or --state.",
                "Provide at most one of --body or --body-file.",
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "pr view",
        LeafMeta {
            gh_equivalent: "gh pr view",
            auth: "optional",
            repo_flag: true,
            repo_inference: true,
            examples: &[
                "gitee pr view 42 --repo octo/demo --json",
                "gitee pr view 42 --json",
                "gitee pr view 42 --comments --page 1 --per-page 20 --json",
                "gitee pr view 42 --repo octo/demo --json number,title,url",
            ],
            notes: &[
                "When --repo is omitted, the command can infer the repository from local git context.",
                "Comments are fetched only when --comments is provided.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "pr comment",
        LeafMeta {
            gh_equivalent: "gh pr comment",
            auth: "required",
            repo_flag: true,
            repo_inference: true,
            input_sources: &["--body", "--body-file"],
            examples: &[
                "gitee pr comment 42 --repo octo/demo --body \"Ship it\" --json",
                "gitee pr comment 42 --body-file ./comment.md --json",
            ],
            notes: &[
                "Provide exactly one of --body or --body-file.",
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "pr review",
        LeafMeta {
            gh_equivalent: "gh pr review",
            auth: "required",
            repo_flag: true,
            repo_inference: true,
            input_sources: &["--body", "--body-file"],
            examples: &[
                "gitee pr review 42 --repo octo/demo --approve --json",
                "gitee pr review 42 --comment --body \"Looks good\" --json",
            ],
            notes: &[
                "Provide exactly one of --approve or --comment.",
                "Comment reviews require --body or --body-file.",
                "Review body input is not supported with --approve.",
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "pr create",
        LeafMeta {
            gh_equivalent: "gh pr create",
            auth: "required",
            repo_flag: true,
            repo_inference: true,
            input_sources: &["--body", "--body-file"],
            examples: &[
                "gitee pr create --title \"Use local head\" --base develop --body \"Built from the local branch\"",
                "gitee pr create --repo octo/demo --head feature/body-file --title \"Read body file\" --body-file ./body.md --json",
                "gitee pr create --repo octo/demo --head feature/body-file --title \"Read body file\" --json number,title,url",
            ],
            notes: &[
                "--title is required.",
                "Provide at most one of --body or --body-file.",
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "pr edit",
        LeafMeta {
            gh_equivalent: "gh pr edit",
            auth: "required",
            repo_flag: true,
            repo_inference: true,
            input_sources: &["--body", "--body-file"],
            examples: &[
                "gitee pr edit 42 --repo octo/demo --title \"Updated title\" --json",
                "gitee pr edit 42 --body-file ./body.md --state open --ready --json",
                "gitee pr edit 42 --repo octo/demo --json number,title,url",
            ],
            notes: &[
                "Provide at least one of --title, --body, --body-file, --state, --draft, or --ready.",
                "Provide at most one of --body or --body-file.",
                "Provide at most one of --draft or --ready.",
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "pr list",
        LeafMeta {
            gh_equivalent: "gh pr list",
            auth: "optional",
            repo_flag: true,
            repo_inference: true,
            examples: &[
                "gitee pr list --repo octo/demo --state open --author octocat --limit 10 --json",
                "gitee pr list --state open --limit 10 --json",
                "gitee pr list --repo octo/demo --limit 10 --json number,title,url",
            ],
            notes: &[
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "pr merge",
        LeafMeta {
            gh_equivalent: "gh pr merge",
            auth: "required",
            repo_flag: true,
            repo_inference: true,
            examples: &[
                "gitee pr merge 42 --repo octo/demo --json",
                "gitee pr merge 42 --squash --json",
            ],
            notes: &[
                "Provide at most one of --merge, --squash, or --rebase.",
                "When no merge strategy flag is provided, the command uses the default merge strategy.",
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "pr checkout",
        LeafMeta {
            gh_equivalent: "gh pr checkout",
            auth: "optional",
            repo_flag: true,
            repo_inference: true,
            local_git_required: true,
            examples: &[
                "gitee pr checkout 42 --repo octo/demo --json",
                "gitee pr checkout 42 --json",
            ],
            notes: &[
                "Requires a local git checkout with an origin remote.",
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "pr status",
        LeafMeta {
            gh_equivalent: "gh pr status",
            auth: "required",
            repo_flag: false,
            repo_inference: false,
            local_git_required: true,
            examples: &[
                "gitee pr status --state open --limit 10 --json",
                "gitee pr status --json",
                "gitee pr status --json number,title,url",
            ],
            notes: &["Requires a local git checkout and authentication."],
            ..leaf_meta("", "")
        },
    ),
    (
        "repo view",
        LeafMeta {
            gh_equivalent: "gh repo view",
            auth: "optional",
            repo_flag: true,
            repo_inference: true,
            examples: &[
                "gitee repo view --repo octo/demo --json",
                "gitee repo view --json",
            ],
            notes: &[
                "When --repo is omitted, the command can infer the repository from local git context.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "repo clone",
        LeafMeta {
            gh_equivalent: "gh repo clone",
            auth: "optional",
            repo_flag: false,
            examples: &[
                "gitee repo clone octo/demo",
                "gitee repo clone octo/demo demo-https --https --json",
            ],
            notes: &[
                "Use at most one of --https or --ssh.",
                "When neither flag is provided, the CLI uses a saved clone protocol preference or prompts on first use.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "skills install",
        LeafMeta {
            gh_equivalent: "not_applicable",
            auth: "not_required",
            examples: &[
                "gitee skills install",
                "gitee skills install --agent claude-code --json",
            ],
            notes: &[
                "Omit --agent to install to ~/.agents/skills/using-gitee-cli.",
                "Pass --agent claude-code to install to ~/.claude/skills/using-gitee-cli.",
                "Only claude-code is a valid --agent value; claude is rejected.",
                "Existing using-gitee-cli installations are overwritten.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "skills uninstall",
        LeafMeta {
            gh_equivalent: "not_applicable",
            auth: "not_required",
            examples: &[
                "gitee skills uninstall",
                "gitee skills uninstall --agent claude-code --json",
            ],
            notes: &[
                "Alias: remove.",
                "Omit --agent to remove ~/.agents/skills/using-gitee-cli; pass --agent claude-code to remove ~/.claude/skills/using-gitee-cli.",
                "Only the selected target's directory is removed.",
                "Missing installations are treated as a successful no-op.",
            ],
            ..leaf_meta("", "")
        },
    ),
    (
        "skills list",
        LeafMeta {
            gh_equivalent: "not_applicable",
            auth: "not_required",
            examples: &[
                "gitee skills list",
                "gitee skills list --agent claude-code --json",
            ],
            notes: &[
                "Alias: ls.",
                "Omit --agent to show a row per target (default + claude-code); pass --agent claude-code to filter to one row.",
                "Only reports the bundled using-gitee-cli skill; it does not scan all installed skills.",
            ],
            ..leaf_meta("", "")
        },
    ),
];

/// The gh-style equivalents and guidance for the top-level groups.
static GROUP_META: &[(&str, &str, &str)] = &[
    (
        "auth",
        "Authenticate with gitee.com and inspect login state",
        "gh auth",
    ),
    (
        "issue",
        "Read, create, edit, and comment on issues",
        "gh issue",
    ),
    (
        "pr",
        "View, create, edit, merge, review, comment on, and check out pull requests",
        "gh pr",
    ),
    ("repo", "Inspect and clone repositories", "gh repo"),
    (
        "skills",
        "Manage the bundled using-gitee-cli skill",
        "not_applicable",
    ),
];

fn leaf_meta_for(path: &str) -> Option<&'static LeafMeta> {
    LEAF_META
        .iter()
        .find(|(key, _)| *key == path)
        .map(|(_, meta)| meta)
}

/// A command's help path without the leading `gitee ` binary prefix.
fn command_path(command: &Command) -> String {
    command
        .get_bin_name()
        .unwrap_or_else(|| command.get_name())
        .strip_prefix("gitee ")
        .unwrap_or_default()
        .to_string()
}

/// Render the CLI-wide discovery JSON (the `gitee help --json` contract).
pub fn root_json(root: &Command) -> Value {
    let commands = root.get_subcommands().map(command_json).collect::<Vec<_>>();

    json!({
        "schema_version": 1,
        "kind": "root",
        "name": root.get_name(),
        "path": root.get_name(),
        "summary": root.get_about().map(render_text).unwrap_or_default(),
        "agent_guidance": {
            "recommended_discovery_command": "gitee help --json",
            "mental_model": "The command surface is intentionally similar to GitHub gh for auth, repo, issue, and pr workflows."
        },
        "unsupported_command_groups": [
            "api", "release", "label", "workflow", "notification"
        ],
        "commands": commands,
    })
}

/// Render JSON for a single command (group or leaf) at the given path.
pub fn command_json(command: &Command) -> Value {
    if command.has_subcommands() {
        let name = command.get_name().to_string();
        let gh = GROUP_META
            .iter()
            .find(|(n, _, _)| *n == name)
            .map(|(_, _, gh)| *gh)
            .unwrap_or("not_applicable");
        let subcommands = command
            .get_subcommands()
            .map(command_json)
            .collect::<Vec<_>>();

        return json!({
            "kind": "group",
            "name": name,
            "path": command_path(command),
            "summary": command.get_about().map(render_text).unwrap_or_default(),
            "gh_equivalent": gh,
            "subcommands": subcommands,
        });
    }

    leaf_json(command)
}

fn leaf_json(command: &Command) -> Value {
    let display_path = command_path(command);
    let gh = leaf_meta_for(&display_path)
        .map(|m| m.gh_equivalent)
        .unwrap_or("");
    let meta = leaf_meta_for(&display_path);

    let json_field_selection = json_field_selection_for_help(&display_path);
    let flags = command
        .get_arguments()
        .filter(|arg| !arg.is_positional())
        .map(option_json)
        .collect::<Vec<_>>();
    let arguments = command
        .get_arguments()
        .filter(|arg| arg.is_positional())
        .map(argument_json)
        .collect::<Vec<_>>();

    json!({
        "kind": "command",
        "name": command.get_name(),
        "path": display_path,
        "summary": command.get_about().map(render_text).unwrap_or_default(),
        "gh_equivalent": gh,
        "supports_json": true,
        "json_field_selection": json_field_selection.is_some(),
        "json_fields": json_field_selection,
        "auth": meta.map(|m| m.auth).unwrap_or("not_required"),
        "repo_flag": meta.map(|m| m.repo_flag).unwrap_or(false),
        "repo_inference": meta.map(|m| m.repo_inference).unwrap_or(false),
        "local_git_required": meta.map(|m| m.local_git_required).unwrap_or(false),
        "flags": flags,
        "arguments": arguments,
        "input_sources": meta.map(|m| m.input_sources).unwrap_or(&[]),
        "examples": meta.map(|m| m.examples).unwrap_or(&[]),
        "notes": meta.map(|m| m.notes).unwrap_or(&[]),
    })
}

fn render_text(value: &clap::builder::StyledStr) -> String {
    value.to_string()
}

fn option_json(arg: &Arg) -> Value {
    let name = format!("--{}", arg.get_long().unwrap_or_default());
    let value_name = arg
        .get_value_names()
        .and_then(|names| names.first())
        .map(|name| name.to_string());

    // Optional-value flags (e.g. --json) don't expose a value name.
    let value_name = if arg_has_optional_value(arg) {
        None
    } else {
        value_name
    };
    let required = arg.is_required_set();

    match value_name {
        Some(value_name) => json!({
            "kind": "option",
            "name": name,
            "value_name": value_name,
            "description": arg.get_help().map(render_text).unwrap_or_default(),
            "required": required,
        }),
        None => json!({
            "kind": "option",
            "name": name,
            "description": arg.get_help().map(render_text).unwrap_or_default(),
            "required": required,
        }),
    }
}

fn argument_json(arg: &Arg) -> Value {
    let value_name = arg
        .get_value_names()
        .and_then(|names| names.first())
        .map(|name| name.to_string())
        .unwrap_or_default();

    json!({
        "kind": "argument",
        "name": arg.get_value_names().and_then(|n| n.first()).map(|n| n.to_string()).unwrap_or_default(),
        "value_name": value_name,
        "description": arg.get_help().map(render_text).unwrap_or_default(),
        "required": arg.is_required_set(),
    })
}

fn arg_has_optional_value(arg: &Arg) -> bool {
    matches!(arg.get_num_args(), Some(range) if range.min_values() == 0)
}
