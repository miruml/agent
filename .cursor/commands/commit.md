# commit

Write git commit(s) for all staged and unstaged changes using Conventional Commits format.

## Instructions

You are tasked with analyzing all uncommitted changes in the codebase and creating well-structured git commit(s).

### Step 1: Gather Information

Run these commands in parallel to understand the current state:

1. `git status` - See all modified, staged, and untracked files
2. `git diff` - View unstaged changes
3. `git diff --cached` - View staged changes  
4. `git log --oneline -10` - Review recent commit history for style consistency

### Step 2: Analyze Changes

- Review ALL changed files across the entire codebase
- Consider the context from our conversation about what was being worked on
- Group related changes together logically
- Determine if changes should be ONE commit or MULTIPLE commits

**Split into multiple commits when:**
- Changes address different concerns (e.g., a bug fix AND a new feature)
- Changes affect unrelated parts of the codebase
- Test files should sometimes be committed with their implementation, sometimes separately
- Refactoring should be separate from feature additions

**Keep as one commit when:**
- All changes are part of the same logical unit of work
- Changes are tightly coupled and don't make sense independently

### Step 3: Write Commit Message(s)

Use **Conventional Commits** format: https://www.conventionalcommits.org/en/v1.0.0/

```
<type>: <description>

[optional body]

[optional footer(s)]
```

**Types (choose the most appropriate):**
- `feat`: A new feature (MINOR version bump)
- `fix`: A bug fix (PATCH version bump)
- `docs`: Documentation only changes
- `style`: Code style changes (formatting, semicolons, etc.) - no logic change
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding or correcting tests
- `build`: Changes to build system or external dependencies
- `ci`: Changes to CI configuration files and scripts
- `chore`: Other changes that don't modify src or test files

**Description guidelines:**
- Use imperative mood ("add" not "added" or "adds")
- Don't capitalize the first letter
- No period at the end
- Keep under 72 characters

**Breaking changes:**
- Add `!` after type: `feat!: remove deprecated endpoint`
- Or add `BREAKING CHANGE:` footer

### Step 4: Present Plan to User

Before executing ANY git commands, present your commit plan:

```
## Proposed Commit(s)

### Commit 1 of N
**Files to include:**
- path/to/file1.ts
- path/to/file2.ts

**Commit message:**
feat: description here

Optional body explaining the why.

---

### Commit 2 of N (if applicable)
...
```

Ask: **"Does this commit plan look good? Reply 'yes' to proceed or provide feedback."**

### Step 5: Handle User Response

**If user approves (says "yes", "y", "Y", or similar affirmation):**
- Proceed immediately to execute the commits
- Do NOT ask for additional confirmation
- Stage files and commit automatically

**If user provides feedback, suggestions, or questions:**
- Re-analyze the changes based on feedback
- Re-read files if necessary to verify
- Present an updated commit plan
- Ask for approval again before proceeding

### Step 6: Execute Commits (Only After User Approval)

For each commit in order:

1. Stage the specific files for that commit:
   ```bash
   git add <file1> <file2> ...
   ```

2. Create the commit using a HEREDOC for proper formatting:
   ```bash
   git commit -m "$(cat <<'EOF'
   type: description

   Optional body here.
   EOF
   )"
   ```

3. Verify with `git status` after each commit

4. Report success and show the commit hash

5. Do NOT push — wait for explicit push instruction

### Important Rules

- **NEVER commit without user approval**
- **NEVER use `git add .` or `git add -A`** - always stage specific files
- **NEVER use interactive flags** like `-i` 
- **NEVER push unless user EXPLICITLY says to push** (e.g., "push", "push it", "push to remote")
- **ALWAYS use HEREDOC** for commit messages to preserve formatting
- **ALWAYS reverify** files and commit messages if user provides any feedback or suggestions before proceeding
- **AUTO-PROCEED** after user says "yes", "y", or "Y" — do not ask again
- If pre-commit hooks modify files, amend the commit to include those changes
- If a commit fails, report the error and ask how to proceed
