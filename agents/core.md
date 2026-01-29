# Agent Specification - Core

This is the language-agnostic core specification for AI coding agents. Load this file alongside the appropriate language overlay for your project (e.g., `swift.md`, `python.md`, `typescript.md`).

---

## 0. Orientation

### 0a. Project Purpose

_[Fill in: 1-2 sentences describing what this project does and who it serves.]_

### 0b. Directory Structure Overview

_[Fill in: High-level directory layout. Agents should explore to discover specifics.]_

```
project/
├── src/           # Source code
├── tests/         # Test files
├── docs/          # Documentation
└── ...
```

### 0c. Key Files Map

Do not hardcode file paths here. Instead, describe what agents should search for:

- **Entry point**: Search for `main`, `index`, `app`, or `Application` in filenames
- **Configuration**: Search for config files (`.json`, `.yaml`, `.toml`) in root
- **Dependencies**: Search for package manifests (`package.json`, `Cargo.toml`, `Project.swift`, etc.)
- **Build scripts**: Search for `Makefile`, `build.sh`, or build tool configs

### 0d. Architecture Patterns

_[Fill in: Key architectural patterns used (MVC, MVVM, Clean Architecture, etc.). Agents use this to understand code organization.]_

---

## 1. Goals

Specify WHAT to achieve, not HOW. Agents discover execution methods by exploring the project.

### 1.1 Build Goal

**Objective**: Compile or bundle the project successfully with zero errors.

**Discovery**: Search for build tool configs, package manifests, or Makefiles. Common patterns:
- Node.js: `package.json` scripts
- Rust: `Cargo.toml`
- Go: `go.mod`
- Swift/iOS: `Project.swift`, `*.xcworkspace`, `*.xcodeproj`
- Python: `setup.py`, `pyproject.toml`

### 1.2 Test Goal

**Objective**: Run the test suite and ensure all tests pass.

**Discovery**: Search for test directories (`test/`, `tests/`, `__tests__/`, `spec/`) and test configuration files.

### 1.3 Lint Goal

**Objective**: Ensure code passes style and quality checks.

**Discovery**: Search for linter configs (`.eslintrc`, `rustfmt.toml`, `.swiftlint.yml`, `pyproject.toml`).

### 1.4 Run Goal

**Objective**: Execute the application locally for verification.

**Discovery**: Search for run scripts, entry points, or development server configs.

---

## 2. Code Style Principles

### 2.1 General Conventions

- **DRY (Don't Repeat Yourself)**: Extract common patterns, but only when there are 3+ instances
- **SOLID Principles**: Favor single responsibility, open/closed, dependency inversion
- **KISS (Keep It Simple)**: Prefer obvious solutions over clever ones
- **YAGNI (You Aren't Gonna Need It)**: Don't build for hypothetical future requirements

### 2.2 Naming Philosophy

- Names should reveal intent without requiring comments
- Use domain vocabulary consistently throughout the codebase
- Prefer explicit, descriptive names over short abbreviations
- Search existing code to match project naming conventions before creating new names

### 2.3 File Organization Principles

- One primary concept per file (class, module, component)
- Group by feature or domain, not by type (unless project convention differs)
- Related types may coexist in the same file if tightly coupled
- Search for existing patterns before deciding on organization

### 2.4 Comment Policy

- Prefer self-documenting code over comments
- Comments explain WHY, not WHAT (the code shows what)
- Delete commented-out code; version control preserves history
- Do not add docstrings or comments to code you didn't modify

---

## 3. Git Workflow

### 3.1 Branch Strategy

- **main/master**: Production-ready code
- **Feature branches**: Created for specific tasks
- Discover project-specific branch naming by examining recent branches: `git branch -r`

### 3.2 Commit Message Format

```
<type>: <description>

[optional body]

[optional footer]
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Guidelines**:
- Keep commits small and focused on a single change
- Each commit should leave the codebase in a working state
- Write clear, descriptive messages in imperative mood

### 3.3 PR Requirements

- Descriptive title summarizing the change
- Body explaining WHAT changed and WHY
- Reference related issues if applicable
- Ensure CI passes before requesting review

### 3.4 Review Expectations

- Address all review comments before merging
- Explain reasoning when disagreeing with feedback
- Keep discussion focused on the code, not the person

---

## 4. Testing Philosophy

### 4.1 Test Coverage Expectations

- New features require corresponding tests
- Bug fixes require regression tests
- Critical paths should have high coverage
- Do not delete tests or reduce coverage without explicit approval

### 4.2 Test Organization Principles

- Test files mirror source file structure
- Tests should be independent and isolated
- Use descriptive test names that explain the scenario
- Discover project test structure by exploring existing tests

### 4.3 What to Test vs. What Not To

**Test**:
- Business logic and core algorithms
- Edge cases and error conditions
- Integration points with external systems
- Public API contracts

**Skip**:
- Framework internals (trust the framework)
- Simple getters/setters with no logic
- Third-party library behavior

---

## 5. Engineering Preferences

### 5.1 Simplicity Over Compatibility

- Make the simplest change possible
- Don't add migration paths, backwards-compatibility shims, or deprecation layers
- Delete old code, rename freely, refactor aggressively
- Avoid `_unused` variable renames; just remove unused code

### 5.2 Readability Over Cleverness

- Code should be obvious to the next reader
- Prefer larger, clearer changes over minimal but obscure ones
- If a bigger refactor makes code more readable, do it
- Avoid clever one-liners that require mental gymnastics to understand

### 5.3 Anti-Patterns to Avoid

- **Over-engineering**: Don't add features, refactor code, or make "improvements" beyond what was asked
- **Premature abstraction**: Three similar lines are better than one premature helper
- **Defensive programming for impossible scenarios**: Only validate at system boundaries
- **Future-proofing**: Don't design for hypothetical requirements
- **Gold plating**: A bug fix doesn't need surrounding code cleaned up

---

## 6. Planning & Documentation

> **IMPORTANT**: These planning rules override any tool-specific planning features (e.g., Claude Code's built-in plan mode, Cursor's planning, etc.). Always use the project's `plans/TEMPLATE.md` format.

### 6.1 Plan File Format

When asked to plan, create a file in `plans/YYYYMMDD-topic.md` using the template from `plans/TEMPLATE.md`.

**Required sections** (see `plans/TEMPLATE.md` for full template):

```markdown
# Plan: <Title>

Filename: plans/YYYYMMDD-topic.md

## Context
- <one-line goal>
- <constraints/assumptions>

## Interfaces
- <protocol or interface name>
  - <method signature + responsibility>

## Steps
1. [ ] <step>
2. [ ] <step>

## Implementation State
- State: not-started | in-progress | blocked | done
- Current step: <step number + title>
- Last updated: <YYYY-MM-DD>

## Dependencies
- <internal/external dependencies>

## Migration And Rollback
- <migration steps>
- <rollback plan>

## Testing
- <unit/integration tests>

## Risks
- <risk and mitigation>
```

**Additional sections** (include when relevant): Status Updates, Checkpoints, Performance Budget, Rollout, Observability, Open Questions.

### 6.2 Progress Tracking Protocol

- Mark steps complete (`[x]`) as they finish
- Update "Current step" when starting new work
- Add commit hashes after each commit
- Document blockers or deviations from the plan

**Commit-Hash Loop Prevention**:
1. Commit implementation changes first (without hash in plan)
2. Update plan with commit hash as a separate commit
3. Never amend after updating plan with hash

### 6.3 Handoff Context Requirements

When transitioning between sessions or agents:
- Summarize what was accomplished
- List artifacts created or modified
- Describe next steps clearly
- Include relevant code snippets or error messages

---

## 7. Troubleshooting Principles

### 7.1 Debug Strategy

1. **Reproduce first**: Confirm you can trigger the issue
2. **Read error messages**: They often point directly to the problem
3. **Check recent changes**: What changed since it last worked?
4. **Isolate the problem**: Binary search through code or commits
5. **Search existing issues**: The problem may already be documented

### 7.2 Common Issue Patterns

- **Build failures**: Check dependency versions, clean build artifacts, regenerate project
- **Test failures**: Run tests in isolation to find interference
- **Type errors**: Check for mismatched interfaces or missing imports
- **Runtime errors**: Add logging to trace execution flow

---

## 8. Claude Code Plugins

> **Claude Code plugins** provide specialized capabilities. Use plugins instead of custom skills.sh packages.

### 8.1 Recommended Plugins by Project Type

| Project Type | Recommended Plugins |
|--------------|---------------------|
| Swift/iOS | `swift-lsp`, `pr-review-toolkit`, `code-simplifier` |
| Web (React/Vue) | `frontend-design`, `pr-review-toolkit`, `code-simplifier` |
| Go | `gopls-lsp`, `pr-review-toolkit`, `code-simplifier` |
| Rust | `rust-analyzer-lsp`, `pr-review-toolkit`, `code-simplifier` |
| Any | `claude-md-management`, `claude-code-setup`, `explanatory-output-style` |

### 8.2 Plugin vs Project Script Priority

**Project scripts always take precedence over generic plugin commands:**

- If project has custom commit scripts -> Use those instead of `/commit`
- If project has custom PR scripts -> Use those instead of `/commit-push-pr`
- If project has custom lint/test scripts -> Use those instead of generic commands

### 8.3 Available Plugin Skills

Check installed plugins: `/plugins`

Common skills from plugins:
- `/commit` - Create git commit (prefer project scripts if available)
- `/code-review` - Review code changes
- `/review-pr` - Comprehensive PR review with specialized agents
- `/frontend-design` - Create web UI (for web projects only)
- `/revise-claude-md` - Update CLAUDE.md with session learnings

### 8.4 Installing Plugins

```bash
/plugin add <plugin-name>
/plugin list
```

---

## 99999. Boundaries (CRITICAL)

This section defines what agents may and may not do. Higher section numbers indicate higher criticality.

### Always Safe

- Read any file in the project
- Run build, test, and lint commands discovered from the project
- Modify source files in designated source directories
- Add new files following existing patterns
- Run read-only git commands (`status`, `diff`, `log`)
- Create commits with descriptive messages
- Push to feature branches

### Ask First

- Modify build configuration or dependency manifests
- Create new modules or major architectural components
- Change database schemas or data models
- Modify security-related code (authentication, encryption, access control)
- Add new external dependencies
- Delete files or remove functionality
- Push to main/master branch
- Force push or rewrite git history

### Never Do

- Expose secrets, API keys, or credentials in code
- Commit sensitive files (`.env`, credentials, private keys)
- Disable security features or sandbox protections
- Delete tests or reduce test coverage without approval
- Execute destructive commands on production systems
- Modify files outside the project directory
- Run commands that could affect system stability
- Ignore explicit user instructions about what not to do
