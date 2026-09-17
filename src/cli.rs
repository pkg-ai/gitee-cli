use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command};
use gitee_api_v5::PullRequestListFilters;

use crate::auth::{AuthService, LoginRequest, LoginTokenSource};
use crate::command::{CommandError, CommandOutcome, EXIT_OK, OutputFormat};
use crate::issue::{
    IssueBodySource, IssueCommentRequest, IssueCreateRequest, IssueEditRequest, IssueListRequest,
    IssueService, IssueStateFilter, IssueViewRequest,
};
use crate::pr::{
    PrCheckoutRequest, PrCommentRequest, PrCreateRequest, PrEditRequest, PrListRequest,
    PrMergeMethod, PrMergeRequest, PrReviewAction, PrReviewRequest, PrService, PrStatusRequest,
    PrTextSource, PrViewRequest,
};
use crate::repo::{CloneTransport, RepoCloneRequest, RepoService, RepoViewRequest};
use crate::skills::{AgentKind, SkillsService};

enum ParseOutcome<T> {
    Value(T),
    Help(CommandOutcome),
}

const PR_JSON_FIELDS: &[&str] = &[
    "number",
    "title",
    "url",
    "state",
    "body",
    "createdAt",
    "updatedAt",
    "mergedAt",
    "isDraft",
    "mergeable",
    "headRefName",
    "headRefOid",
    "baseRefName",
    "baseRefOid",
];
const REPO_VIEW_JSON_FIELDS: &[&str] = &[
    "name",
    "nameWithOwner",
    "url",
    "defaultBranch",
    "sshUrl",
    "cloneUrl",
    "isFork",
];
const ISSUE_JSON_FIELDS: &[&str] = &[
    "number",
    "title",
    "url",
    "state",
    "body",
    "createdAt",
    "updatedAt",
];

pub(crate) fn json_field_selection_for_help(path: &str) -> Option<&'static [&'static str]> {
    match path {
        "pr view" | "pr create" | "pr edit" | "pr list" | "pr status" => Some(PR_JSON_FIELDS),
        "repo view" => Some(REPO_VIEW_JSON_FIELDS),
        "issue view" | "issue edit" | "issue list" => Some(ISSUE_JSON_FIELDS),
        _ => None,
    }
}

pub fn run(args: Vec<String>) -> Result<CommandOutcome, CommandError> {
    if matches!(args.as_slice(), [flag] if is_version_flag(flag)) {
        return Ok(render_version());
    }

    let Some((command, rest)) = args.split_first() else {
        return Err(CommandError::usage("missing command"));
    };

    if is_help_flag(command) {
        return Ok(render_help(root_help_command()));
    }

    match command.as_str() {
        "auth" => run_auth(rest),
        "help" => run_help(rest),
        "issue" => run_issue(rest),
        "pr" => run_pr(rest),
        "repo" => run_repo(rest),
        "skills" => run_skills(rest),
        _ => Err(CommandError::usage("unsupported command")),
    }
}

fn run_auth(args: &[String]) -> Result<CommandOutcome, CommandError> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(CommandError::usage("missing auth subcommand"));
    };

    if is_help_flag(subcommand) {
        return Ok(render_help(auth_help_command()));
    }

    let auth = AuthService::from_env();

    match subcommand.as_str() {
        "status" => execute_parsed(parse_output_only(rest, auth_status_command()), |output| {
            auth.status(output)
        }),
        "login" => execute_parsed(parse_auth_login_args(rest), |request| auth.login(request)),
        "logout" => execute_parsed(parse_output_only(rest, auth_logout_command()), |output| {
            auth.logout(output)
        }),
        _ => Err(CommandError::usage("unsupported command")),
    }
}

fn run_issue(args: &[String]) -> Result<CommandOutcome, CommandError> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(CommandError::usage("missing issue subcommand"));
    };

    if is_help_flag(subcommand) {
        return Ok(render_help(issue_help_command()));
    }

    let issue = IssueService::from_env();

    match subcommand.as_str() {
        "create" => execute_parsed(parse_issue_create_args(rest), |request| {
            issue.create(request)
        }),
        "comment" => execute_parsed(parse_issue_comment_args(rest), |request| {
            issue.comment(request)
        }),
        "edit" => execute_parsed(parse_issue_edit_args(rest), |request| issue.edit(request)),
        "list" => execute_parsed(parse_issue_list_args(rest), |request| issue.list(request)),
        "view" => execute_parsed(parse_issue_view_args(rest), |request| issue.view(request)),
        _ => Err(CommandError::usage("unsupported command")),
    }
}

fn run_pr(args: &[String]) -> Result<CommandOutcome, CommandError> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(CommandError::usage("missing pr subcommand"));
    };

    if is_help_flag(subcommand) {
        return Ok(render_help(pr_help_command()));
    }

    let pr = PrService::from_env();

    match subcommand.as_str() {
        "checkout" => execute_parsed(parse_pr_checkout_args(rest), |request| pr.checkout(request)),
        "comment" => execute_parsed(parse_pr_comment_args(rest), |request| pr.comment(request)),
        "create" => execute_parsed(parse_pr_create_args(rest), |request| pr.create(request)),
        "edit" => execute_parsed(parse_pr_edit_args(rest), |request| pr.edit(request)),
        "list" => execute_parsed(parse_pr_list_args(rest), |request| pr.list(request)),
        "merge" => execute_parsed(parse_pr_merge_args(rest), |request| pr.merge(request)),
        "review" => execute_parsed(parse_pr_review_args(rest), |request| pr.review(request)),
        "status" => execute_parsed(parse_pr_status_args(rest), |request| pr.status(request)),
        "view" => execute_parsed(parse_pr_view_args(rest), |request| pr.view(request)),
        _ => Err(CommandError::usage("unsupported command")),
    }
}

fn run_repo(args: &[String]) -> Result<CommandOutcome, CommandError> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(CommandError::usage("missing repo subcommand"));
    };

    if is_help_flag(subcommand) {
        return Ok(render_help(repo_help_command()));
    }

    let repo = RepoService::from_env();

    match subcommand.as_str() {
        "clone" => execute_parsed(parse_repo_clone_args(rest), |request| repo.clone(request)),
        "view" => execute_parsed(parse_repo_view_args(rest), |request| repo.view(request)),
        _ => Err(CommandError::usage("unsupported command")),
    }
}

fn run_skills(args: &[String]) -> Result<CommandOutcome, CommandError> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(CommandError::usage("missing skills subcommand"));
    };

    if is_help_flag(subcommand) {
        return Ok(render_help(skills_help_command()));
    }

    match subcommand.as_str() {
        "install" => execute_parsed(
            parse_skills_mutation_args(rest, "skills install", skills_install_command()),
            |(agent, output)| SkillsService::from_env()?.install(agent, output),
        ),
        "uninstall" | "remove" => execute_parsed(
            parse_skills_mutation_args(rest, "skills uninstall", skills_uninstall_command()),
            |(agent, output)| SkillsService::from_env()?.uninstall(agent, output),
        ),
        "list" | "ls" => execute_parsed(parse_skills_list_args(rest), |(agent, output)| {
            Ok(SkillsService::from_env()?.list(agent, output))
        }),
        _ => Err(CommandError::usage("unsupported command")),
    }
}

fn run_help(args: &[String]) -> Result<CommandOutcome, CommandError> {
    execute_parsed(parse_matches(help_command(), args), |matches| {
        let output = output_format(&matches);
        let topics = values(&matches, "topics");

        match output {
            OutputFormat::Text => {
                let Some(topic) = resolve_help_topic(&topics) else {
                    return Err(CommandError::usage("unknown help topic"));
                };
                Ok(render_help((topic.text_command)()))
            }
            OutputFormat::Json { .. } => {
                let command = if topics.is_empty() {
                    root_help_command()
                } else {
                    let Some(topic) = resolve_help_topic(&topics) else {
                        return Err(CommandError::usage("unknown help topic"));
                    };
                    (topic.text_command)()
                };
                Ok(CommandOutcome::json(
                    EXIT_OK,
                    if topics.is_empty() {
                        crate::help::root_json(&command)
                    } else {
                        crate::help::command_json(&command)
                    },
                ))
            }
        }
    })
}

struct HelpTopic {
    text_command: fn() -> Command,
}

fn resolve_help_topic(path: &[String]) -> Option<HelpTopic> {
    let path = path.iter().map(String::as_str).collect::<Vec<_>>();

    Some(match path.as_slice() {
        [] => HelpTopic {
            text_command: root_help_command,
        },
        ["auth"] => HelpTopic {
            text_command: auth_help_command,
        },
        ["auth", "login"] => HelpTopic {
            text_command: auth_login_command,
        },
        ["auth", "logout"] => HelpTopic {
            text_command: auth_logout_command,
        },
        ["auth", "status"] => HelpTopic {
            text_command: auth_status_command,
        },
        ["issue"] => HelpTopic {
            text_command: issue_help_command,
        },
        ["issue", "comment"] => HelpTopic {
            text_command: issue_comment_command,
        },
        ["issue", "create"] => HelpTopic {
            text_command: issue_create_command,
        },
        ["issue", "edit"] => HelpTopic {
            text_command: issue_edit_command,
        },
        ["issue", "list"] => HelpTopic {
            text_command: issue_list_command,
        },
        ["issue", "view"] => HelpTopic {
            text_command: issue_view_command,
        },
        ["pr"] => HelpTopic {
            text_command: pr_help_command,
        },
        ["pr", "checkout"] => HelpTopic {
            text_command: pr_checkout_command,
        },
        ["pr", "comment"] => HelpTopic {
            text_command: pr_comment_command,
        },
        ["pr", "create"] => HelpTopic {
            text_command: pr_create_command,
        },
        ["pr", "edit"] => HelpTopic {
            text_command: pr_edit_command,
        },
        ["pr", "list"] => HelpTopic {
            text_command: pr_list_command,
        },
        ["pr", "merge"] => HelpTopic {
            text_command: pr_merge_command,
        },
        ["pr", "review"] => HelpTopic {
            text_command: pr_review_command,
        },
        ["pr", "status"] => HelpTopic {
            text_command: pr_status_command,
        },
        ["pr", "view"] => HelpTopic {
            text_command: pr_view_command,
        },
        ["repo"] => HelpTopic {
            text_command: repo_help_command,
        },
        ["repo", "clone"] => HelpTopic {
            text_command: repo_clone_command,
        },
        ["repo", "view"] => HelpTopic {
            text_command: repo_view_command,
        },
        ["skills"] => HelpTopic {
            text_command: skills_help_command,
        },
        ["skills", "install"] => HelpTopic {
            text_command: skills_install_command,
        },
        ["skills", "uninstall"] | ["skills", "remove"] => HelpTopic {
            text_command: skills_uninstall_command,
        },
        ["skills", "list"] | ["skills", "ls"] => HelpTopic {
            text_command: skills_list_command,
        },
        _ => return None,
    })
}

fn execute_parsed<T>(
    parsed: Result<ParseOutcome<T>, CommandError>,
    handler: impl FnOnce(T) -> Result<CommandOutcome, CommandError>,
) -> Result<CommandOutcome, CommandError> {
    match parsed? {
        ParseOutcome::Value(value) => handler(value),
        ParseOutcome::Help(help) => Ok(help),
    }
}

fn parse_output_only(
    args: &[String],
    command: Command,
) -> Result<ParseOutcome<OutputFormat>, CommandError> {
    let command_name = command.get_name().to_string();
    map_parsed(parse_matches(command, args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, &command_name, &[])?;
        Ok(output)
    })
}

fn parse_skills_mutation_args(
    args: &[String],
    command_name: &str,
    command: Command,
) -> Result<ParseOutcome<(AgentKind, OutputFormat)>, CommandError> {
    map_parsed(parse_matches(command, args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, command_name, &[])?;
        let agent = match last_value(&matches, "agent") {
            Some(value) => AgentKind::parse(&value)?,
            None => AgentKind::Default,
        };
        Ok((agent, output))
    })
}

fn parse_skills_list_args(
    args: &[String],
) -> Result<ParseOutcome<(Option<AgentKind>, OutputFormat)>, CommandError> {
    map_parsed(parse_matches(skills_list_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "skills list", &[])?;
        let agent = match last_value(&matches, "agent") {
            Some(value) => Some(AgentKind::parse(&value)?),
            None => None,
        };
        Ok((agent, output))
    })
}

fn parse_auth_login_args(args: &[String]) -> Result<ParseOutcome<LoginRequest>, CommandError> {
    map_parsed(parse_matches(auth_login_command(), args), |matches| {
        let output = output_format(&matches);
        let token_values = values(&matches, "token");
        let with_token_count = flag_count(&matches, "with_token");

        if token_values.len() + with_token_count > 1 {
            return Err(CommandError::usage(
                "provide only one of --token or --with-token",
            ));
        }

        let Some(token_source) = token_values
            .last()
            .map(|value| LoginTokenSource::Flag(value.clone()))
            .or_else(|| (with_token_count == 1).then_some(LoginTokenSource::Stdin))
        else {
            return Err(CommandError::usage(
                "login requires --token or --with-token",
            ));
        };

        Ok(LoginRequest {
            output,
            token_source,
        })
    })
}

fn parse_issue_list_args(args: &[String]) -> Result<ParseOutcome<IssueListRequest>, CommandError> {
    map_parsed(parse_matches(issue_list_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "issue list", ISSUE_JSON_FIELDS)?;
        let repo = last_value(&matches, "repo");
        let state = match last_value(&matches, "state") {
            Some(value) => IssueStateFilter::parse(&value)?,
            None => IssueStateFilter::Open,
        };
        let search = last_value(&matches, "search");
        let page = positive_int(&matches, "page", "--page", 1)?;
        let per_page = positive_int(&matches, "per_page", "--per-page", 20)?;

        Ok(IssueListRequest {
            output,
            repo,
            state,
            search,
            page,
            per_page,
        })
    })
}

fn parse_issue_view_args(args: &[String]) -> Result<ParseOutcome<IssueViewRequest>, CommandError> {
    map_parsed(parse_matches(issue_view_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "issue view", ISSUE_JSON_FIELDS)?;
        let repo = last_value(&matches, "repo");
        let comments = flag_count(&matches, "comments") > 0;
        let page = positive_int(&matches, "page", "--page", 1)?;
        let per_page = positive_int(&matches, "per_page", "--per-page", 20)?;
        let positionals = values(&matches, "positionals");
        let number = single_positional(
            &positionals,
            "issue view",
            "an issue number",
            "issue number",
        )?;

        Ok(IssueViewRequest {
            output,
            repo,
            number: number.to_string(),
            comments,
            page,
            per_page,
        })
    })
}

fn parse_issue_comment_args(
    args: &[String],
) -> Result<ParseOutcome<IssueCommentRequest>, CommandError> {
    map_parsed(parse_matches(issue_comment_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "issue comment", &[])?;
        let repo = last_value(&matches, "repo");
        let body_values = values(&matches, "body");
        let body_file_values = values(&matches, "body_file");
        let positionals = values(&matches, "positionals");
        let number = single_positional(
            &positionals,
            "issue comment",
            "an issue number",
            "issue number",
        )?;

        let (body_ref, body_file_ref) = body_or_file(&body_values, &body_file_values)?;
        let body = body_ref
            .map(|value| IssueBodySource::Inline(value.to_string()))
            .or_else(|| body_file_ref.map(|value| IssueBodySource::File(PathBuf::from(value))));
        let Some(body) = body else {
            return Err(CommandError::usage(
                "issue comment requires --body or --body-file",
            ));
        };

        Ok(IssueCommentRequest {
            output,
            repo,
            number: number.to_string(),
            body,
        })
    })
}

fn parse_issue_create_args(
    args: &[String],
) -> Result<ParseOutcome<IssueCreateRequest>, CommandError> {
    map_parsed(parse_matches(issue_create_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "issue create", &[])?;
        let repo = last_value(&matches, "repo");
        let title = last_value(&matches, "title");
        let body_values = values(&matches, "body");
        let body_file_values = values(&matches, "body_file");

        let Some(title) = title else {
            return Err(CommandError::usage("issue create requires --title"));
        };

        let (body_ref, body_file_ref) = body_or_file(&body_values, &body_file_values)?;
        let body = body_ref
            .map(|value| IssueBodySource::Inline(value.to_string()))
            .or_else(|| body_file_ref.map(|value| IssueBodySource::File(PathBuf::from(value))));

        Ok(IssueCreateRequest {
            output,
            repo,
            title,
            body,
        })
    })
}

fn parse_issue_edit_args(args: &[String]) -> Result<ParseOutcome<IssueEditRequest>, CommandError> {
    map_parsed(parse_matches(issue_edit_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "issue edit", ISSUE_JSON_FIELDS)?;
        let repo = last_value(&matches, "repo");
        let title = last_value(&matches, "title");
        let state = match last_value(&matches, "state") {
            Some(value) => Some(parse_issue_edit_state(&value)?),
            None => None,
        };
        let body_values = values(&matches, "body");
        let body_file_values = values(&matches, "body_file");
        let positionals = values(&matches, "positionals");
        let number = single_positional(
            &positionals,
            "issue edit",
            "an issue number",
            "issue number",
        )?;

        let (body_ref, body_file_ref) = body_or_file(&body_values, &body_file_values)?;
        let body = body_ref
            .map(|value| IssueBodySource::Inline(value.to_string()))
            .or_else(|| body_file_ref.map(|value| IssueBodySource::File(PathBuf::from(value))));

        if title.is_none() && body.is_none() && state.is_none() {
            return Err(CommandError::usage(
                "issue edit requires at least one of --title, --body, --body-file, or --state",
            ));
        }

        Ok(IssueEditRequest {
            output,
            repo,
            number: number.to_string(),
            title,
            body,
            state,
        })
    })
}

fn parse_pr_view_args(args: &[String]) -> Result<ParseOutcome<PrViewRequest>, CommandError> {
    map_parsed(parse_matches(pr_view_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "pr view", PR_JSON_FIELDS)?;
        let repo = last_value(&matches, "repo");
        let positionals = values(&matches, "positionals");
        let number = parse_pr_number(&positionals, "pr view")?;
        let comments = flag_count(&matches, "comments") > 0;
        let page = positive_int(&matches, "page", "--page", 1)?;
        let per_page = positive_int(&matches, "per_page", "--per-page", 20)?;

        Ok(PrViewRequest {
            output,
            repo,
            number,
            comments,
            page,
            per_page,
        })
    })
}

fn parse_repo_view_args(args: &[String]) -> Result<ParseOutcome<RepoViewRequest>, CommandError> {
    map_parsed(parse_matches(repo_view_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "repo view", REPO_VIEW_JSON_FIELDS)?;
        Ok(RepoViewRequest {
            output,
            repo: last_value(&matches, "repo"),
        })
    })
}

fn parse_repo_clone_args(args: &[String]) -> Result<ParseOutcome<RepoCloneRequest>, CommandError> {
    map_parsed(parse_matches(repo_clone_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "repo clone", &[])?;
        let https_count = flag_count(&matches, "https");
        let ssh_count = flag_count(&matches, "ssh");
        let positionals = values(&matches, "positionals");

        if https_count + ssh_count > 1 {
            return Err(CommandError::usage("provide only one of --https or --ssh"));
        }

        let Some(repo) = positionals.first() else {
            return Err(CommandError::usage(
                "repo clone requires an owner/repo slug",
            ));
        };

        if positionals.len() > 2 {
            return Err(CommandError::usage(
                "repo clone accepts at most one destination path",
            ));
        }

        Ok(RepoCloneRequest {
            output,
            repo: repo.clone(),
            destination: positionals.get(1).cloned(),
            transport: if ssh_count == 1 {
                Some(CloneTransport::Ssh)
            } else if https_count == 1 {
                Some(CloneTransport::Https)
            } else {
                None
            },
        })
    })
}

fn parse_pr_comment_args(args: &[String]) -> Result<ParseOutcome<PrCommentRequest>, CommandError> {
    map_parsed(parse_matches(pr_comment_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "pr comment", &[])?;
        let repo = last_value(&matches, "repo");
        let body_values = values(&matches, "body");
        let body_file_values = values(&matches, "body_file");
        let positionals = values(&matches, "positionals");
        let number = parse_pr_number(&positionals, "pr comment")?;

        let (body_ref, body_file_ref) = body_or_file(&body_values, &body_file_values)?;
        let body = body_ref
            .map(|value| PrTextSource::Inline(value.to_string()))
            .or_else(|| body_file_ref.map(|value| PrTextSource::File(value.to_string())));
        let Some(body) = body else {
            return Err(CommandError::usage(
                "pr comment requires --body or --body-file",
            ));
        };

        Ok(PrCommentRequest {
            output,
            repo,
            number,
            body,
        })
    })
}

fn parse_pr_review_args(args: &[String]) -> Result<ParseOutcome<PrReviewRequest>, CommandError> {
    map_parsed(parse_matches(pr_review_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "pr review", &[])?;
        let repo = last_value(&matches, "repo");
        let body_values = values(&matches, "body");
        let body_file_values = values(&matches, "body_file");
        let positionals = values(&matches, "positionals");
        let number = parse_pr_number(&positionals, "pr review")?;

        let approve_count = flag_count(&matches, "approve");
        let comment_count = flag_count(&matches, "comment");

        if approve_count + comment_count != 1 {
            return Err(CommandError::usage(
                "provide exactly one of --approve or --comment",
            ));
        }

        let (body_value, body_file_value) = body_or_file(&body_values, &body_file_values)?;
        let body = body_value
            .map(|value| PrTextSource::Inline(value.to_string()))
            .or_else(|| body_file_value.map(|value| PrTextSource::File(value.to_string())));

        let action = if approve_count == 1 {
            if body.is_some() {
                return Err(CommandError::usage(
                    "review body is not supported with --approve",
                ));
            }
            PrReviewAction::Approve
        } else {
            let Some(body) = body else {
                return Err(CommandError::usage(
                    "pr review --comment requires --body or --body-file",
                ));
            };
            PrReviewAction::Comment(body)
        };

        Ok(PrReviewRequest {
            output,
            repo,
            number,
            action,
        })
    })
}

fn parse_pr_list_args(args: &[String]) -> Result<ParseOutcome<PrListRequest>, CommandError> {
    map_parsed(parse_matches(pr_list_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "pr list", PR_JSON_FIELDS)?;
        let repo = last_value(&matches, "repo");
        let state = match last_value(&matches, "state") {
            Some(value) => Some(parse_pr_state(&value)?),
            None => None,
        };
        let author = last_value(&matches, "author");
        let assignee = last_value(&matches, "assignee");
        let base = last_value(&matches, "base");
        let head = last_value(&matches, "head");
        let limit = match last_value(&matches, "limit") {
            Some(value) => parse_limit(&value)?,
            None => 30,
        };

        Ok(PrListRequest {
            output,
            repo,
            filters: PullRequestListFilters {
                state,
                author,
                assignee,
                base,
                head,
                limit,
            },
        })
    })
}

fn parse_pr_create_args(args: &[String]) -> Result<ParseOutcome<PrCreateRequest>, CommandError> {
    map_parsed(parse_matches(pr_create_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "pr create", PR_JSON_FIELDS)?;
        let repo = last_value(&matches, "repo");
        let head = last_value(&matches, "head");
        let base = last_value(&matches, "base");
        let title = last_value(&matches, "title");
        let body_values = values(&matches, "body");
        let body_file_values = values(&matches, "body_file");

        let Some(title) = title else {
            return Err(CommandError::usage("pr create requires --title"));
        };

        let (body_value, body_file_value) = body_or_file(&body_values, &body_file_values)?;
        let body = body_value
            .map(|value| PrTextSource::Inline(value.to_string()))
            .or_else(|| body_file_value.map(|value| PrTextSource::File(value.to_string())));

        Ok(PrCreateRequest {
            output,
            repo,
            head,
            base,
            title,
            body,
        })
    })
}

fn parse_pr_edit_args(args: &[String]) -> Result<ParseOutcome<PrEditRequest>, CommandError> {
    map_parsed(parse_matches(pr_edit_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "pr edit", PR_JSON_FIELDS)?;
        let repo = last_value(&matches, "repo");
        let title = last_value(&matches, "title");
        let state = match last_value(&matches, "state") {
            Some(value) => Some(parse_pr_edit_state(&value)?),
            None => None,
        };
        let draft_count = flag_count(&matches, "draft");
        let ready_count = flag_count(&matches, "ready");
        let body_values = values(&matches, "body");
        let body_file_values = values(&matches, "body_file");
        let positionals = values(&matches, "positionals");
        let number = parse_pr_number(&positionals, "pr edit")?;

        let (body_value, body_file_value) = body_or_file(&body_values, &body_file_values)?;
        let body = body_value
            .map(|value| PrTextSource::Inline(value.to_string()))
            .or_else(|| body_file_value.map(|value| PrTextSource::File(value.to_string())));

        if draft_count + ready_count > 1 {
            return Err(CommandError::usage(
                "provide only one of --draft or --ready",
            ));
        }

        let draft = if draft_count == 1 {
            Some(true)
        } else if ready_count == 1 {
            Some(false)
        } else {
            None
        };

        if title.is_none() && body.is_none() && state.is_none() && draft.is_none() {
            return Err(CommandError::usage(
                "pr edit requires at least one of --title, --body, --body-file, --state, --draft, or --ready",
            ));
        }

        Ok(PrEditRequest {
            output,
            repo,
            number,
            title,
            body,
            state,
            draft,
        })
    })
}

fn parse_pr_checkout_args(
    args: &[String],
) -> Result<ParseOutcome<PrCheckoutRequest>, CommandError> {
    map_parsed(parse_matches(pr_checkout_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "pr checkout", &[])?;
        let repo = last_value(&matches, "repo");
        let positionals = values(&matches, "positionals");
        let number = parse_pr_number(&positionals, "pr checkout")?;

        Ok(PrCheckoutRequest {
            output,
            repo,
            number,
        })
    })
}

fn parse_pr_merge_args(args: &[String]) -> Result<ParseOutcome<PrMergeRequest>, CommandError> {
    map_parsed(parse_matches(pr_merge_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "pr merge", &[])?;
        let repo = last_value(&matches, "repo");
        let merge_count = flag_count(&matches, "merge");
        let squash_count = flag_count(&matches, "squash");
        let rebase_count = flag_count(&matches, "rebase");
        let positionals = values(&matches, "positionals");
        let number = parse_pr_number(&positionals, "pr merge")?;

        if merge_count + squash_count + rebase_count > 1 {
            return Err(CommandError::usage(
                "provide only one of --merge, --squash, or --rebase",
            ));
        }

        let merge_method = if squash_count == 1 {
            PrMergeMethod::Squash
        } else if rebase_count == 1 {
            PrMergeMethod::Rebase
        } else {
            PrMergeMethod::Merge
        };

        Ok(PrMergeRequest {
            output,
            repo,
            number,
            merge_method,
        })
    })
}

fn parse_pr_status_args(args: &[String]) -> Result<ParseOutcome<PrStatusRequest>, CommandError> {
    map_parsed(parse_matches(pr_status_command(), args), |matches| {
        let output = output_format(&matches);
        validate_json_field_selection(&output, "pr status", PR_JSON_FIELDS)?;
        let state = match last_value(&matches, "state") {
            Some(value) => Some(parse_pr_state(&value)?),
            None => None,
        };
        let limit = match last_value(&matches, "limit") {
            Some(value) => parse_limit(&value)?,
            None => 30,
        };

        Ok(PrStatusRequest {
            output,
            filters: PullRequestListFilters {
                state,
                author: None,
                assignee: None,
                base: None,
                head: None,
                limit,
            },
        })
    })
}

fn map_parsed<T>(
    parsed: Result<ParseOutcome<ArgMatches>, CommandError>,
    mapper: impl FnOnce(ArgMatches) -> Result<T, CommandError>,
) -> Result<ParseOutcome<T>, CommandError> {
    match parsed? {
        ParseOutcome::Value(matches) => Ok(ParseOutcome::Value(mapper(matches)?)),
        ParseOutcome::Help(help) => Ok(ParseOutcome::Help(help)),
    }
}

fn parse_matches(
    command: Command,
    args: &[String],
) -> Result<ParseOutcome<ArgMatches>, CommandError> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push(command.get_name().to_string());
    argv.extend(args.iter().cloned());

    match command.clone().try_get_matches_from(argv) {
        Ok(matches) => Ok(ParseOutcome::Value(matches)),
        Err(error) => match error.kind() {
            clap::error::ErrorKind::DisplayHelp => Ok(ParseOutcome::Help(render_help(command))),
            clap::error::ErrorKind::DisplayVersion => Ok(ParseOutcome::Help(CommandOutcome::text(
                EXIT_OK,
                error.to_string().trim_end().to_string(),
            ))),
            _ => Err(map_clap_error(error)),
        },
    }
}

fn map_clap_error(error: clap::Error) -> CommandError {
    if let Some(flag) = missing_value_flag(&error.to_string()) {
        return CommandError::usage(format!("missing value for {flag}"));
    }

    match error.kind() {
        clap::error::ErrorKind::UnknownArgument
        | clap::error::ErrorKind::InvalidSubcommand
        | clap::error::ErrorKind::ArgumentConflict
        | clap::error::ErrorKind::TooManyValues
        | clap::error::ErrorKind::WrongNumberOfValues
        | clap::error::ErrorKind::NoEquals
        | clap::error::ErrorKind::ValueValidation => CommandError::usage("unsupported command"),
        _ => CommandError::usage("unsupported command"),
    }
}

fn missing_value_flag(message: &str) -> Option<String> {
    let prefix = "a value is required for '";
    let start = message.find(prefix)? + prefix.len();
    let remaining = &message[start..];
    let end = remaining.find('\'')?;
    let argument = remaining[..end].split_whitespace().next()?;
    Some(argument.to_string())
}

fn output_format(matches: &ArgMatches) -> OutputFormat {
    let raw_json_values = values(matches, "json");
    let json_values = raw_json_values
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if !raw_json_values.is_empty() {
        OutputFormat::Json {
            fields: (!json_values.is_empty()).then_some(json_values),
        }
    } else {
        OutputFormat::Text
    }
}

fn validate_json_field_selection(
    output: &OutputFormat,
    command_name: &str,
    supported_fields: &[&str],
) -> Result<(), CommandError> {
    let Some(fields) = output.json_fields() else {
        return Ok(());
    };

    if supported_fields.is_empty() {
        return Err(CommandError::usage(format!(
            "{command_name} does not support selecting JSON fields yet"
        )));
    }

    if let Some(field) = fields
        .iter()
        .find(|field| !supported_fields.contains(&field.as_str()))
    {
        return Err(CommandError::usage(format!(
            "unknown JSON field for {command_name}: {field}"
        )));
    }

    Ok(())
}

fn values(matches: &ArgMatches, id: &str) -> Vec<String> {
    matches
        .get_many::<String>(id)
        .map(|values| values.cloned().collect())
        .unwrap_or_default()
}

fn last_value(matches: &ArgMatches, id: &str) -> Option<String> {
    values(matches, id).into_iter().last()
}

fn flag_count(matches: &ArgMatches, id: &str) -> usize {
    usize::from(matches.get_count(id))
}

/// Assert exactly one positional argument and return it.
fn single_positional<'a>(
    positionals: &'a [String],
    command: &str,
    required: &str,
    accept: &str,
) -> Result<&'a str, CommandError> {
    let Some(value) = positionals.first() else {
        return Err(CommandError::usage(format!(
            "{command} requires {required}"
        )));
    };

    if positionals.len() > 1 {
        return Err(CommandError::usage(format!(
            "{command} accepts exactly one {accept}"
        )));
    }

    Ok(value)
}

/// Parse a single PR-number positional.
fn parse_pr_number(positionals: &[String], command: &str) -> Result<u64, CommandError> {
    let raw = single_positional(
        positionals,
        command,
        "a pull request number",
        "pull request number",
    )?;
    raw.parse::<u64>().map_err(|_| {
        CommandError::usage("invalid pull request number: expected a positive integer")
    })
}

/// Resolve `--body`/`--body-file` mutual exclusion into an (inline, file) pair
/// of references, with at most one present.
fn body_or_file<'a>(
    body: &'a [String],
    body_file: &'a [String],
) -> Result<(Option<&'a str>, Option<&'a str>), CommandError> {
    if body.len() + body_file.len() > 1 {
        return Err(CommandError::usage(
            "provide only one of --body or --body-file",
        ));
    }

    Ok((
        body.last().map(String::as_str),
        body_file.last().map(String::as_str),
    ))
}

/// Parse a non-negative integer flag, falling back to `default` when absent.
fn positive_int(
    matches: &ArgMatches,
    id: &str,
    flag: &str,
    default: u32,
) -> Result<u32, CommandError> {
    match last_value(matches, id) {
        Some(value) => parse_positive_integer_flag(flag, &value),
        None => Ok(default),
    }
}

fn render_help(mut command: Command) -> CommandOutcome {
    let supports_json_field_selection = help_command_supports_json_field_selection(&command);
    let mut buffer = Vec::new();
    command
        .write_long_help(&mut buffer)
        .expect("writing clap help should succeed");

    let body = String::from_utf8(buffer)
        .expect("clap help should be utf-8")
        .trim_end()
        .to_string();
    let body = if supports_json_field_selection {
        body
    } else {
        body.replace("--json [<FIELDS>]", "--json")
    };

    CommandOutcome::text(EXIT_OK, body)
}

fn render_version() -> CommandOutcome {
    CommandOutcome::text(EXIT_OK, format!("gitee {}", env!("CARGO_PKG_VERSION")))
}

fn help_command_supports_json_field_selection(command: &Command) -> bool {
    let path = command.get_bin_name().unwrap_or(command.get_name());
    let path = path.strip_prefix("gitee ").unwrap_or(path);
    json_field_selection_for_help(path).is_some()
}

fn is_help_flag(arg: &str) -> bool {
    matches!(arg, "--help" | "-h")
}

fn is_version_flag(arg: &str) -> bool {
    matches!(arg, "--version" | "-V")
}

fn root_help_command() -> Command {
    base_command("gitee", "gitee")
        .about("Agent-first CLI for gitee.com")
        .arg(version_flag())
        .after_help(
            "Examples:\n  gitee auth status --json\n  gitee repo view --repo octo/demo --json\n  gitee help --json\n\nAgent discovery:\n  Use `gitee help --json` to inspect commands, flags, examples, and gh-style equivalents.",
        )
        .subcommand(auth_help_command())
        .subcommand(issue_help_command())
        .subcommand(pr_help_command())
        .subcommand(repo_help_command())
        .subcommand(skills_help_command())
}

fn auth_help_command() -> Command {
    base_command("auth", "gitee auth")
        .about("Authenticate with gitee.com and inspect login state")
        .subcommand(auth_status_command())
        .subcommand(auth_login_command())
        .subcommand(auth_logout_command())
}

fn issue_help_command() -> Command {
    base_command("issue", "gitee issue")
        .about("Read, create, edit, and comment on issues")
        .subcommand(issue_create_command())
        .subcommand(issue_comment_command())
        .subcommand(issue_edit_command())
        .subcommand(issue_list_command())
        .subcommand(issue_view_command())
}

fn pr_help_command() -> Command {
    base_command("pr", "gitee pr")
        .about("View, create, edit, merge, review, comment on, and check out pull requests")
        .subcommand(pr_checkout_command())
        .subcommand(pr_comment_command())
        .subcommand(pr_create_command())
        .subcommand(pr_edit_command())
        .subcommand(pr_list_command())
        .subcommand(pr_merge_command())
        .subcommand(pr_review_command())
        .subcommand(pr_status_command())
        .subcommand(pr_view_command())
}

fn repo_help_command() -> Command {
    base_command("repo", "gitee repo")
        .about("Inspect and clone repositories")
        .subcommand(repo_clone_command())
        .subcommand(repo_view_command())
}

fn skills_help_command() -> Command {
    base_command("skills", "gitee skills")
        .about("Manage the bundled using-gitee-cli skill")
        .subcommand(skills_install_command())
        .subcommand(skills_uninstall_command())
        .subcommand(skills_list_command())
}

fn auth_status_command() -> Command {
    output_only_command("status", "gitee auth status")
        .about("Check whether authentication is currently usable")
}

fn auth_login_command() -> Command {
    base_command("login", "gitee auth login")
        .about("Validate and save a personal access token")
        .arg(json_flag())
        .arg(string_option(
            "token",
            "token",
            "TOKEN",
            "Personal access token to validate and save",
        ))
        .arg(count_flag(
            "with_token",
            "with-token",
            "Read the token from stdin instead of a flag",
        ))
}

fn auth_logout_command() -> Command {
    output_only_command("logout", "gitee auth logout")
        .about("Remove the saved token from local config")
}

fn skills_install_command() -> Command {
    output_only_command("install", "gitee skills install")
        .about("Install the bundled using-gitee-cli skill into a coding agent's skills directory")
        .arg(agent_option())
}

fn skills_uninstall_command() -> Command {
    output_only_command("uninstall", "gitee skills uninstall")
        .visible_alias("remove")
        .about("Remove the bundled using-gitee-cli skill from a coding agent's skills directory")
        .arg(agent_option())
}

fn skills_list_command() -> Command {
    output_only_command("list", "gitee skills list")
        .visible_alias("ls")
        .about("List the bundled using-gitee-cli skill installation status per target")
        .arg(agent_option())
}

fn issue_list_command() -> Command {
    base_command("list", "gitee issue list")
        .about("List issues for a repository")
        .arg(json_flag())
        .arg(repo_option())
        .arg(string_option(
            "state",
            "state",
            "STATE",
            "Filter issues by state: open, closed, or all",
        ))
        .arg(string_option(
            "search",
            "search",
            "SEARCH",
            "Filter issues by keyword text",
        ))
        .arg(string_option(
            "page",
            "page",
            "PAGE",
            "1-based page number to request",
        ))
        .arg(string_option(
            "per_page",
            "per-page",
            "PER_PAGE",
            "Number of issues to return per page",
        ))
}

fn issue_view_command() -> Command {
    base_command("view", "gitee issue view")
        .about("View a single issue and optionally include comments")
        .arg(json_flag())
        .arg(repo_option())
        .arg(count_flag(
            "comments",
            "comments",
            "Include issue comments in the response",
        ))
        .arg(string_option(
            "page",
            "page",
            "PAGE",
            "1-based page number for comment pagination",
        ))
        .arg(string_option(
            "per_page",
            "per-page",
            "PER_PAGE",
            "Number of comments to return per page",
        ))
        .arg(positionals_arg(
            "positionals",
            "ISSUE",
            "Issue number or identifier, such as I123",
        ))
}

fn issue_comment_command() -> Command {
    base_command("comment", "gitee issue comment")
        .about("Post a comment to an issue")
        .arg(json_flag())
        .arg(repo_option())
        .arg(string_option(
            "body",
            "body",
            "BODY",
            "Inline comment body text",
        ))
        .arg(string_option(
            "body_file",
            "body-file",
            "PATH",
            "Read comment body text from a file",
        ))
        .arg(positionals_arg(
            "positionals",
            "ISSUE",
            "Issue number or identifier, such as I123",
        ))
}

fn issue_create_command() -> Command {
    base_command("create", "gitee issue create")
        .about("Create a new issue")
        .arg(json_flag())
        .arg(repo_option())
        .arg(string_option("title", "title", "TITLE", "Issue title"))
        .arg(string_option(
            "body",
            "body",
            "BODY",
            "Inline issue body text",
        ))
        .arg(string_option(
            "body_file",
            "body-file",
            "PATH",
            "Read issue body text from a file",
        ))
}

fn issue_edit_command() -> Command {
    base_command("edit", "gitee issue edit")
        .about("Edit an existing issue")
        .arg(json_flag())
        .arg(repo_option())
        .arg(string_option(
            "title",
            "title",
            "TITLE",
            "Replace the issue title",
        ))
        .arg(string_option(
            "body",
            "body",
            "BODY",
            "Replace the issue body text",
        ))
        .arg(string_option(
            "body_file",
            "body-file",
            "PATH",
            "Read the issue body text from a file",
        ))
        .arg(string_option(
            "state",
            "state",
            "STATE",
            "Change issue state: open or closed",
        ))
        .arg(positionals_arg(
            "positionals",
            "ISSUE",
            "Issue number or identifier, such as I123",
        ))
}

fn pr_view_command() -> Command {
    base_command("view", "gitee pr view")
        .about("View a single pull request and optionally include comments")
        .arg(json_flag())
        .arg(repo_option())
        .arg(count_flag(
            "comments",
            "comments",
            "Include pull request comments in the response",
        ))
        .arg(string_option(
            "page",
            "page",
            "PAGE",
            "1-based page number for comment pagination",
        ))
        .arg(string_option(
            "per_page",
            "per-page",
            "PER_PAGE",
            "Number of comments to return per page",
        ))
        .arg(positionals_arg("positionals", "PR", "Pull request number"))
}

fn repo_view_command() -> Command {
    output_only_command("view", "gitee repo view")
        .about("View repository metadata")
        .arg(repo_option())
}

fn repo_clone_command() -> Command {
    base_command("clone", "gitee repo clone")
        .about("Clone a repository by owner/repo slug")
        .arg(json_flag())
        .arg(count_flag("https", "https", "Clone over HTTPS"))
        .arg(count_flag("ssh", "ssh", "Clone over SSH"))
        .arg(positionals_arg(
            "positionals",
            "ARG",
            "Provide OWNER/REPO first, then an optional destination path",
        ))
}

fn pr_comment_command() -> Command {
    base_command("comment", "gitee pr comment")
        .about("Post a comment to a pull request")
        .arg(json_flag())
        .arg(repo_option())
        .arg(string_option(
            "body",
            "body",
            "BODY",
            "Inline comment body text",
        ))
        .arg(string_option(
            "body_file",
            "body-file",
            "PATH",
            "Read comment body text from a file",
        ))
        .arg(positionals_arg("positionals", "PR", "Pull request number"))
}

fn pr_review_command() -> Command {
    base_command("review", "gitee pr review")
        .about("Add a review to a pull request")
        .arg(json_flag())
        .arg(repo_option())
        .arg(
            Arg::new("approve")
                .short('a')
                .long("approve")
                .action(ArgAction::Count)
                .help("Approve the pull request"),
        )
        .arg(
            Arg::new("comment")
                .short('c')
                .long("comment")
                .action(ArgAction::Count)
                .help("Comment on the pull request"),
        )
        .arg(
            Arg::new("body")
                .short('b')
                .long("body")
                .action(ArgAction::Append)
                .num_args(1)
                .value_name("BODY")
                .allow_hyphen_values(true)
                .help("Inline review body text"),
        )
        .arg(
            Arg::new("body_file")
                .short('F')
                .long("body-file")
                .action(ArgAction::Append)
                .num_args(1)
                .value_name("PATH")
                .allow_hyphen_values(true)
                .help("Read review body text from a file"),
        )
        .arg(positionals_arg("positionals", "PR", "Pull request number"))
}

fn pr_list_command() -> Command {
    base_command("list", "gitee pr list")
        .about("List pull requests with filters")
        .arg(json_flag())
        .arg(repo_option())
        .arg(string_option(
            "state",
            "state",
            "STATE",
            "Filter pull requests by state: open, closed, merged, or all",
        ))
        .arg(string_option(
            "author",
            "author",
            "AUTHOR",
            "Filter pull requests by author login",
        ))
        .arg(string_option(
            "assignee",
            "assignee",
            "ASSIGNEE",
            "Filter pull requests by assignee login",
        ))
        .arg(string_option(
            "base",
            "base",
            "BASE",
            "Filter pull requests by base branch",
        ))
        .arg(string_option(
            "head",
            "head",
            "HEAD",
            "Filter pull requests by head branch",
        ))
        .arg(string_option(
            "limit",
            "limit",
            "LIMIT",
            "Maximum number of pull requests to return",
        ))
}

fn pr_create_command() -> Command {
    base_command("create", "gitee pr create")
        .about("Create a pull request from the current branch or an explicit head")
        .arg(json_flag())
        .arg(repo_option())
        .arg(string_option(
            "head",
            "head",
            "HEAD",
            "Head branch to use instead of the current branch",
        ))
        .arg(string_option(
            "base",
            "base",
            "BASE",
            "Base branch to target",
        ))
        .arg(string_option(
            "title",
            "title",
            "TITLE",
            "Pull request title",
        ))
        .arg(string_option(
            "body",
            "body",
            "BODY",
            "Inline pull request body text",
        ))
        .arg(string_option(
            "body_file",
            "body-file",
            "PATH",
            "Read pull request body text from a file",
        ))
}

fn pr_edit_command() -> Command {
    base_command("edit", "gitee pr edit")
        .about("Edit an existing pull request")
        .arg(json_flag())
        .arg(repo_option())
        .arg(string_option(
            "title",
            "title",
            "TITLE",
            "Replace the pull request title",
        ))
        .arg(string_option(
            "body",
            "body",
            "BODY",
            "Replace the pull request body text",
        ))
        .arg(string_option(
            "body_file",
            "body-file",
            "PATH",
            "Read the pull request body text from a file",
        ))
        .arg(string_option(
            "state",
            "state",
            "STATE",
            "Change pull request state: open or closed",
        ))
        .arg(count_flag(
            "draft",
            "draft",
            "Mark the pull request as draft",
        ))
        .arg(count_flag(
            "ready",
            "ready",
            "Mark the pull request as ready",
        ))
        .arg(positionals_arg("positionals", "PR", "Pull request number"))
}

fn pr_merge_command() -> Command {
    base_command("merge", "gitee pr merge")
        .about("Merge a pull request")
        .arg(json_flag())
        .arg(repo_option())
        .arg(count_flag("merge", "merge", "Merge the pull request"))
        .arg(count_flag(
            "squash",
            "squash",
            "Squash and merge the pull request",
        ))
        .arg(count_flag(
            "rebase",
            "rebase",
            "Rebase and merge the pull request",
        ))
        .arg(positionals_arg("positionals", "PR", "Pull request number"))
}

fn pr_checkout_command() -> Command {
    base_command("checkout", "gitee pr checkout")
        .about("Fetch and check out a pull request head branch")
        .arg(json_flag())
        .arg(repo_option())
        .arg(positionals_arg("positionals", "PR", "Pull request number"))
}

fn pr_status_command() -> Command {
    base_command("status", "gitee pr status")
        .about("Show pull requests related to the current local checkout")
        .arg(json_flag())
        .arg(string_option(
            "state",
            "state",
            "STATE",
            "Filter pull requests by state: open, closed, merged, or all",
        ))
        .arg(string_option(
            "limit",
            "limit",
            "LIMIT",
            "Maximum number of pull requests to return",
        ))
}

fn help_command() -> Command {
    base_command("help", "gitee help")
        .about("Show help for a command path or output machine-readable command metadata")
        .after_help(
            "Examples:\n  gitee help\n  gitee help --json\n  gitee help pr create\n  gitee help pr create --json",
        )
        .arg(json_flag())
        .arg(positionals_arg(
            "topics",
            "TOPIC",
            "Command path to inspect, such as `pr` or `pr create`",
        ))
}

fn output_only_command(name: &'static str, bin_name: &'static str) -> Command {
    base_command(name, bin_name).arg(json_flag())
}

fn base_command(name: &'static str, bin_name: &'static str) -> Command {
    Command::new(name)
        .bin_name(bin_name)
        .disable_version_flag(true)
}

fn json_flag() -> Arg {
    Arg::new("json")
        .long("json")
        .action(ArgAction::Append)
        .num_args(0..=1)
        .default_missing_value("")
        .value_delimiter(',')
        .value_name("FIELDS")
        .help("Output machine-readable JSON")
}

fn version_flag() -> Arg {
    Arg::new("version")
        .short('V')
        .long("version")
        .action(ArgAction::SetTrue)
        .help("Print version")
}

fn count_flag(id: &'static str, long: &'static str, help: &'static str) -> Arg {
    Arg::new(id).long(long).action(ArgAction::Count).help(help)
}

fn string_option(
    id: &'static str,
    long: &'static str,
    value_name: &'static str,
    help: &'static str,
) -> Arg {
    Arg::new(id)
        .long(long)
        .action(ArgAction::Append)
        .num_args(1)
        .value_name(value_name)
        .allow_hyphen_values(true)
        .help(help)
}

fn repo_option() -> Arg {
    string_option(
        "repo",
        "repo",
        "REPO",
        "Target repository as OWNER/REPO; defaults to local git context when supported",
    )
}

fn agent_option() -> Arg {
    string_option(
        "agent",
        "agent",
        "AGENT",
        "Install target: claude-code (omit the flag for the default cross-client target)",
    )
}

fn positionals_arg(id: &'static str, value_name: &'static str, help: &'static str) -> Arg {
    Arg::new(id)
        .index(1)
        .action(ArgAction::Append)
        .num_args(0..)
        .value_name(value_name)
        .help(help)
}

fn parse_pr_state(value: &str) -> Result<String, CommandError> {
    match value {
        "open" | "closed" | "merged" | "all" => Ok(value.to_string()),
        _ => Err(CommandError::usage(
            "invalid value for --state: expected open, closed, merged, or all",
        )),
    }
}

fn parse_pr_edit_state(value: &str) -> Result<String, CommandError> {
    parse_open_closed_state(value)
}

fn parse_issue_edit_state(value: &str) -> Result<String, CommandError> {
    parse_open_closed_state(value)
}

fn parse_open_closed_state(value: &str) -> Result<String, CommandError> {
    match value {
        "open" | "closed" => Ok(value.to_string()),
        _ => Err(CommandError::usage(
            "invalid value for --state: expected open or closed",
        )),
    }
}

fn parse_limit(value: &str) -> Result<usize, CommandError> {
    let parsed = value.parse::<usize>().map_err(|_| {
        CommandError::usage("invalid value for --limit: expected a positive integer")
    })?;

    if parsed == 0 {
        return Err(CommandError::usage(
            "invalid value for --limit: expected a positive integer",
        ));
    }

    Ok(parsed)
}

fn parse_positive_integer_flag(flag: &str, value: &str) -> Result<u32, CommandError> {
    let parsed = value.parse::<u32>().ok().filter(|candidate| *candidate > 0);

    parsed.ok_or_else(|| {
        CommandError::usage(format!(
            "invalid value for {flag}: expected a positive integer"
        ))
    })
}
