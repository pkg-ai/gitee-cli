# Agent-First Workflows for gitee.com

`gitee-cli` is an agent-first command-line tool for working with `gitee.com`
from scripts, local terminals, and AI-driven workflows.

It gives you a small, stable command surface for authentication, repository
inspection, issue triage, and pull request workflows without dropping down to
raw Gitee API calls. The installed executable is named `gitee`.

For agent and LLM discovery, start with:

```bash
gitee help --json
```

This returns a machine-readable manifest of command groups, subcommands, flags,
examples, and `gh`-style equivalents. For one command, use a topic path such as
`gitee help pr create --json`.

> `gitee-cli` is an unofficial community project, not affiliated with, endorsed
> by, or sponsored by Gitee or `gitee.com`. Gitee and `gitee.com` are trademarks
> of their respective owners, referenced here only to identify platform
> compatibility.

## Why This Exists

`gitee-cli` is built for the high-frequency Gitee tasks that show up in
automation and day-to-day development:

- checking whether auth is usable before starting work
- inspecting repository metadata from a slug or a local checkout
- reading issue history before making a change
- viewing, listing, creating, merging, reviewing, commenting on, and checking out pull requests
- producing stable `--json` output and meaningful exit codes for scripts

The project is intentionally opinionated:

- it targets `gitee.com`
- it prefers explicit, non-interactive workflows
- it supports both human-readable output and stable `--json`
- it uses local git context when that makes common workflows faster

## Who It Is For

Use `gitee-cli` if you want:

- a Gitee workflow tool that fits AI agents and automation
- a terminal-friendly way to inspect repos, issues, and pull requests
- write operations that accept flags, files, or stdin instead of prompts
- predictable behavior that can be scripted safely

## Install

Install the latest stable release with npm:

```bash
npm install -g @pkg-ai/gitee-cli
gitee --version
```

Or run without a global install:

```bash
npx @pkg-ai/gitee-cli --version
```

The npm package includes prebuilt binaries for Apple Silicon macOS
(`aarch64-apple-darwin`) and Linux x86_64 (`x86_64-unknown-linux-musl`).

## Install the Bundled Skill in Coding Agents

Install the bundled `using-gitee-cli` skill. By default it goes to
`~/.agents/skills`, the cross-client Agent Skills standard directory:

```bash
gitee skills install
```

For Claude Code's personal skill directory, pass `--agent claude-code`:

```bash
gitee skills install --agent claude-code
```

`--agent` supports only `claude-code`; omit it for the default cross-client
target. Use `gitee skills list` to check status, `gitee skills uninstall` to
remove a target.

## Common Workflows

### Fix an issue and ship a pull request, end to end

This is one full cycle in the terminal: read an issue, fix it, open a PR,
field and address review comments, get it approved, and merge.

Start by reading the issue you'll work on, including its discussion:

```bash
gitee issue view I123 --repo octo/demo --comments --page 1 --per-page 20 --json
```

Write the fix in your editor, then open a PR from the current branch:

```bash
gitee pr create --title "Fix I123" --base develop --body "Closes I123" --json
```

Bring up the PR for code review — review it yourself, or reply to a
reviewer's comment in place:

```bash
gitee pr view 42 --repo octo/demo --json
gitee pr comment 42 --repo octo/demo --body "Fixed, please re-review" --json
```

Pull the PR's comments to see what needs fixing, apply the changes, and push:

```bash
gitee pr view 42 --repo octo/demo --comments --page 1 --per-page 20 --json
```

Once the feedback is addressed, approve and merge:

```bash
gitee pr review 42 --repo octo/demo --approve --json
gitee pr merge 42 --repo octo/demo --squash --json
```

Gitee has no GitHub-style request-changes review state. Comment reviews require
a body; approval reviews do not accept one.

## Local Repository Context

When `--repo` is omitted, `gitee-cli` infers the repository from the local git
checkout, keeping commands short when you are already in the right repository.

<details>
<summary>Supported <code>origin</code> URL forms</summary>

- `git@gitee.com:owner/repo.git`
- `ssh://git@gitee.com/owner/repo.git`
- `https://gitee.com/owner/repo.git`
- `http://gitee.com/owner/repo.git`

</details>

## Authentication And Configuration

Most reads work without a saved token on public repositories. Authentication is
required for writes and some user-specific flows; private repositories and some
human-name fallback lookups may still require it.

Runtime token resolution order:

1. `GITEE_TOKEN`
2. saved config file token

Config directory resolution order:

1. `GITEE_CONFIG_DIR`
2. `XDG_CONFIG_HOME/gitee`
3. `HOME/.config/gitee`
4. current directory `./.gitee`

By default `~/.config/gitee/config.toml` stores the saved token and non-secret
clone protocol preference.

<details>
<summary>Relevant environment variables</summary>

- `GITEE_TOKEN`: overrides the saved token at runtime
- `GITEE_CONFIG_DIR`: points directly to the config directory
- `XDG_CONFIG_HOME`: used when `GITEE_CONFIG_DIR` is not set
- `HOME`: used for the default config path
- `GITEE_BASE_URL`: overrides the API base URL (default `https://gitee.com/api`);
  mainly for tests or local API mocking

</details>

## Automation Contracts

`gitee-cli` is designed to be scriptable:

- successful output goes to `stdout`
- errors go to `stderr`
- core commands support `--json`
- exit codes are stable enough to branch on in automation

Exit codes:

- `0`: success
- `2`: usage error
- `3`: authentication error or authentication required
- `4`: config error
- `5`: remote request error
- `6`: resource not found
- `7`: local git error

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for local development, testing, and
pull request guidance.

## License

MIT. See [LICENSE](./LICENSE).
