# Plan: Talent - Unified Agent Skills Manager

Filename: plans/20260129-talent-implementation.md

## Context

- Build a cross-platform GUI application (Rust + Tauri + Svelte) for managing Agent Skills across multiple AI CLI tools
- Central skill storage in `~/.talent/skills/` with symlinks to each target CLI's skills directory
- Tech Stack: Rust (backend), Tauri v2 (framework), Svelte 5 + TypeScript + Vite (frontend), CodeMirror 6 (editor), notify crate (file watching), clap (CLI)

## Interfaces

- **SkillManager**
  - `new() -> Result<Self>` - Initialize manager with config, discover targets and skills
  - `skills() -> Vec<&Skill>` - List all skills
  - `sync_all() -> Vec<SyncResult>` - Sync skills to all targets
  - `validate_skill(name: &str) -> Result<()>` - Validate a skill

- **Syncer**
  - `sync_target(target: &Target, skills: &[Skill]) -> Result<SyncResult>` - Create/remove symlinks for a target
  - `sync_all(targets: &[Target], skills: &[Skill]) -> Vec<SyncResult>` - Sync all targets

- **Validator**
  - `validate(skill: &mut Skill) -> Result<()>` - Validate skill against agentskills.io rules

- **SkillWatcher**
  - `start() -> Result<()>` - Begin watching skills directory
  - `poll() -> Vec<SkillEvent>` - Non-blocking check for file events

- **Tauri Commands**
  - `get_skills() -> Vec<SkillInfo>` - List skills for frontend
  - `get_targets() -> Vec<TargetInfo>` - List targets for frontend
  - `sync_all() -> Vec<SyncResultInfo>` - Trigger sync from UI
  - `create_skill(name: String) -> Result<String>` - Create new skill

## Steps

### Phase 1: Project Setup & Core Backend

1. [x] Install Rust and initialize workspace with `talent-core` and `talent-cli` crates
2. [x] Implement error types (`crates/talent-core/src/error.rs`)
3. [x] Implement config module with serialization (`crates/talent-core/src/config.rs`)
4. [x] Implement skill model with frontmatter parsing (`crates/talent-core/src/skill.rs`)
5. [x] Implement target model with auto-detection (`crates/talent-core/src/target.rs`)
6. [x] Implement validation engine (`crates/talent-core/src/validator.rs`)
7. [x] Implement symlink syncer (`crates/talent-core/src/syncer.rs`)
8. [x] Implement file watcher (`crates/talent-core/src/watcher.rs`)
9. [x] Create skill manager integration layer (`crates/talent-core/src/manager.rs`)
10. [x] Complete CLI implementation with all commands (`crates/talent-cli/src/main.rs`)

### Phase 2: Tauri Shell

11. [x] Initialize Tauri project with commands (`src-tauri/`)

### Phase 3: Frontend (Svelte)

12. [x] Initialize Svelte frontend with skill list UI (`src/`, `package.json`)
13. [x] Run and verify complete application

### Phase 4-6: Future (Editor, Import/Export, Polish)

14. [x] Add CodeMirror 6 editor for skill editing
15. [ ] Implement skill import/export functionality
16. [ ] Polish UI and add system tray support

## Implementation State

- State: in-progress
- Current step: 15 - Implement skill import/export functionality
- Last updated: 2026-01-29
- Checkpoints:
  - 2026-01-29 not-started Initial plan created
  - 2026-01-29 in-progress Steps 1-3 completed (workspace setup, error types, config)
  - 2026-01-29 in-progress Steps 4-6 completed (skill model, target model, validator). All 35 tests passing.
  - 2026-01-29 in-progress Steps 7-9 completed (syncer, watcher, manager). All 56 tests passing.
  - 2026-01-29 in-progress Step 10 completed (CLI implementation). All 56 tests passing. CLI commands: list, sync, doctor, targets, create, validate.
  - 2026-01-29 in-progress Steps 11-13 completed (Tauri backend, Svelte frontend, app verification). MVP complete. All 56 tests passing.
  - 2026-01-29 in-progress Step 14 completed (CodeMirror 6 editor). All 58 tests passing. Skill editing with live preview.

## Status Updates

- 2026-01-29 not-started Plan migrated to template format
- 2026-01-29 in-progress Completed batch 1: Rust workspace initialized, error types implemented with thiserror, config module with TOML serialization. All 9 tests passing.
- 2026-01-29 in-progress Completed batch 2: Skill model with frontmatter parsing (YAML between --- delimiters), target model with auto-detection for 6 CLIs (Claude Code, Codex, Gemini, Cursor, Amp, Goose), validation engine with kebab-case name enforcement. All 35 tests passing.
- 2026-01-29 in-progress Completed batch 3: Symlink syncer with create/remove/unchanged tracking, file watcher with debounced events, skill manager facade integrating all components. All 56 tests passing.
- 2026-01-29 in-progress Completed batch 4: CLI implementation with clap 4 - subcommands for list, sync, doctor, targets, create, validate. JSON output support. All 56 tests passing.
- 2026-01-29 in-progress Completed batch 5: Tauri v2 backend with 9 commands (get_skills, get_targets, sync_all, create_skill, validate_skill, validate_all, refresh_skills, delete_skill, get_stats). Svelte 5 frontend with responsive skill list and target list UI. App runs successfully. All 56 tests passing.
- 2026-01-29 in-progress Completed batch 6: CodeMirror 6 editor integration. Added get_skill_content and save_skill_content Tauri commands. Created SkillEditor Svelte component with markdown highlighting and dark theme. Split-panel UI with unsaved changes indicator. All 58 tests passing.

## Dependencies

- **External:**
  - Rust toolchain (rustup)
  - Node.js 18+
  - Tauri CLI v2
- **Crates:**
  - serde, serde_json, serde_yaml, toml (serialization)
  - thiserror (error handling)
  - chrono (timestamps)
  - walkdir (directory traversal)
  - notify (file watching)
  - clap (CLI parsing)
  - dirs (home directory)
  - tauri v2 (app framework)
- **NPM:**
  - @tauri-apps/api v2
  - svelte v5
  - vite v6
  - typescript v5

## Migration And Rollback

- No migration needed (greenfield project)
- Rollback: Users can manually manage skills in individual CLI directories

## Performance Budget

- App startup: < 500ms
- Skill sync: < 100ms per target
- File watch debounce: 500ms
- Memory: < 50MB idle

## Rollout

- Phase 1-3: MVP with basic sync functionality
- Phase 4: Editor integration
- Phase 5: Import/export for skill sharing
- Phase 6: Polish, system tray, auto-updates

## Observability

- CLI `doctor` command for diagnostics
- Validation status indicators in UI
- Sync result reporting (created/removed/errors)

## Testing

- Unit tests for each core module (config, skill, target, validator, syncer, watcher)
- Integration test: create skill -> validate -> sync -> verify symlinks
- Edge cases:
  - Missing SKILL.md
  - Invalid frontmatter
  - Broken symlinks
  - Target directory doesn't exist
  - Permission errors

## Open Questions

- Should we support skill versioning?
- How to handle conflicts when same skill name exists in target?
- Should we support skill templates beyond the basic one?

## Risks

- **Symlink permissions on Windows**: Mitigation - Use junction points or copy fallback
- **File watcher reliability**: Mitigation - Manual sync button always available
- **Target CLI updates changing paths**: Mitigation - Auto-detect logic with fallback to manual config
