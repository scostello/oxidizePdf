# Claude Configuration Files

This directory contains configuration files for Claude Code (CLI) when working on the oxidize-pdf project.

## File Structure

### `settings.json` (Committed to Git)
Team-wide permissions for safe, commonly-used commands. These permissions are:
- **Safe**: No destructive operations (no `git rm`, `git push`, `cargo clean`, etc.)
- **Universal**: Useful for any team member working on Rust projects
- **Read-mostly**: Emphasis on read-only operations and non-destructive builds

**Included permissions:**
- Rust toolchain commands (`cargo check`, `cargo test`, `cargo build`, etc.)
- Safe git operations (`git add`, `git commit`, `git fetch`, `git branch`, `git checkout`)
- Read-only shell utilities (`find`, `grep`, `ls`, `echo`)
- Rust LSP plugin

### `settings.local.json` (Gitignored - Personal)
Machine-specific or personal preference permissions. This file is **not committed** to the repository.

**Included permissions:**
- Documentation tools (requires MCP server setup)
  - `WebFetch(domain:docs.rs)`
  - `mcp__context7__*` (Context7 MCP server)
- Project-specific skills (`iced-gui-expert`)
- Python tooling (`uv` commands for Python utilities)
- Optional IDE integrations (JetBrains MCP)
- Optional build tools (`sccache`)
- Personal preferences (`WebSearch`, GitHub WebFetch)

## Removed Permissions (Error-Prone)

The following permissions were **intentionally removed** for safety:

### Destructive Git Commands
- ❌ `git rm` - Can permanently delete files
- ❌ `git restore` - Can discard uncommitted changes
- ❌ `git rebase` - Can cause complex merge conflicts
- ❌ `git push` - Can push to wrong branch/remote
- ❌ `git cherry-pick` - Can cause conflicts
- ❌ `git remote add` - Can misconfigure remotes
- ❌ Hardcoded `git merge` commands - Too specific, error-prone

### Destructive Cargo Commands
- ❌ `cargo clean` - Deletes build artifacts (minor but annoying)
- ❌ `cargo update` - Can break dependencies unexpectedly

## Usage

### For New Team Members
1. Clone the repository
2. `settings.json` provides immediate access to standard Rust/Git commands
3. Optionally create your own `settings.local.json` for personal preferences

### For Existing Team Members
- Your existing `settings.local.json` has been cleaned up
- Common permissions have been moved to the shared `settings.json`
- Removed dangerous commands that could cause accidental damage

## Customizing Your Local Settings

To add personal preferences, edit `.claude/settings.local.json`:

```json
{
  "permissions": {
    "allow": [
      "Bash(your-custom-command:*)"
    ]
  },
  "enabledPlugins": {
    "your-plugin@namespace": true
  }
}
```

**Note:** This file is gitignored, so your personal preferences won't affect other team members.

## Design Principles

1. **Shared by default**: Common, safe commands go in `settings.json`
2. **No destructive operations**: Requires manual user approval
3. **Read-heavy**: Prefer read-only commands over write/delete
4. **Explicit over implicit**: If it can fail badly, don't auto-allow it
5. **Machine-specific stays local**: IDE integrations, build caches, MCP servers

## Migration from Old Settings

Previously, all permissions were in `settings.local.json`. The consolidation:
- Moved **34 safe permissions** → `settings.json` (shared, committed)
- Kept **13 permissions** → `settings.local.json` (personal, gitignored)
- Removed **11 error-prone permissions** (require manual approval)

## Questions?

See the project's `CLAUDE.md` for more context on development workflows.
