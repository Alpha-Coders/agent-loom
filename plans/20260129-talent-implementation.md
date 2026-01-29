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
  - `discover_importable_skills() -> Vec<DiscoveredSkill>` - Scan targets for importable skills
  - `import_skills(selections: Vec<ImportSelection>) -> ImportResult` - Import selected skills
  - `check_filemerge_available() -> bool` - Check if FileMerge (opendiff) is available
  - `open_filemerge(existing: String, incoming: String) -> Result<()>` - Open FileMerge for diff

- **Importer** (new module)
  - `discover_importable_skills(targets: &[Target]) -> Vec<DiscoveredSkill>` - Scan target directories
  - `import_skill(source: &Path, name: &str, overwrite: bool) -> Result<ImportedSkill>` - Copy skill to central storage
  - `check_conflict(name: &str, skills_dir: &Path) -> Option<ConflictInfo>` - Check if skill already exists

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
15. [x] Implement skill import functionality (see Design: Skill Import Feature below)
    - 15.1 [x] Add importer module to talent-core (`crates/talent-core/src/importer.rs`)
    - 15.2 [x] Add Tauri commands for import (`discover_importable_skills`, `import_skills`, `check_filemerge_available`, `open_filemerge`)
    - 15.3 [x] Create ImportDialog.svelte component
    - 15.4 [x] Update empty state in App.svelte with import option
    - 15.5 [x] Add Import button to toolbar
    - 15.6 [x] Write tests for importer module
16. [ ] Polish UI and add system tray support

## Design: Skill Import Feature

### Overview

Allow users to migrate existing skills from target CLI directories (Codex, Claude Code, etc.) into Talent's central storage (`~/.talent/skills/`). This addresses the "No skills found" experience for users who already have skills in their CLI tools.

### Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Discovery scope | Structured only (`SKILL.md` dirs) | Avoids format conversion, Codex skills already compatible |
| Conflict handling | Ask per-conflict | User control with visual diff option |
| Diff tool | FileMerge (macOS) | Native tool, detected via `which opendiff` |
| Migration trigger | Empty state + toolbar button | Prominent for new users, accessible for power users |
| Import flow | One-click auto-scan | Single screen with checkboxes and conflict dropdowns |
| Post-import | Auto-sync | Immediately distribute to all targets |

### Section 1: Discovery Engine

Scans all target directories for importable skills with proper `SKILL.md` structure.

```
For each target in [ClaudeCode, Codex, Gemini, Cursor, Amp, Goose]:
  1. Get skills directory (e.g., ~/.codex/skills/)
  2. Walk subdirectories (depth 1)
  3. Check for SKILL.md file
  4. Parse frontmatter to extract name, description
  5. Skip if it's a symlink pointing to ~/.talent/skills/ (already managed)
  6. Add to discovered list with source target info
```

**Key details:**
- Reuses existing `Skill::load()` parsing logic
- Filters out symlinks to avoid re-importing Talent-managed skills
- Returns: `Vec<DiscoveredSkill>` with `name`, `description`, `source_path`, `source_target`

### Section 2: Conflict Detection & Resolution

Compares discovered skills against existing `~/.talent/skills/` and provides resolution options.

**Conflict states:**

| State | Indicator | Actions Available |
|-------|-----------|-------------------|
| **New** | Green "NEW" badge | Checkbox to import |
| **Conflict** | Orange "EXISTS" badge | Skip / Overwrite / Compare |
| **Already managed** | Grey "SYNCED" | Hidden (filtered out) |

**FileMerge integration:**
```rust
fn open_filemerge(existing: &Path, incoming: &Path) -> Result<()> {
    // Check if opendiff (FileMerge CLI) is available
    if Command::new("which").arg("opendiff").status()?.success() {
        Command::new("opendiff")
            .arg(existing.join("SKILL.md"))
            .arg(incoming.join("SKILL.md"))
            .spawn()?;
    } else {
        // Fallback: show diff in UI or terminal
    }
}
```

**Resolution dropdown per conflict:**
- **Skip** - Don't import, keep existing
- **Overwrite** - Replace existing with incoming
- **Compare** - Open FileMerge, then user picks Skip or Overwrite

### Section 3: Import Dialog UI

**Component:** `ImportDialog.svelte`

```
┌─────────────────────────────────────────────────────────┐
│  Import Skills from Other Tools                    [X]  │
├─────────────────────────────────────────────────────────┤
│  Scanning: Codex, Claude Code, Gemini, Cursor...        │
│                                                         │
│  ☑ gh-fix-ci          Codex       [NEW]                │
│  ☑ gh-address-comments Codex      [NEW]                │
│  ☑ pokemon-fetch      Codex       [NEW]                │
│  ☑ pptx               Codex       [EXISTS ▼]           │
│                                   ├─ Skip              │
│                                   ├─ Overwrite         │
│                                   └─ Compare...        │
│  ☑ create-plan        Codex       [NEW]                │
│                                                         │
│  Found 9 skills from 1 target                          │
│  ─────────────────────────────────────────────────────  │
│                              [Cancel]  [Import 8]       │
└─────────────────────────────────────────────────────────┘
```

**States:**
- Loading: "Scanning targets..."
- Empty: "No importable skills found"
- Results: Checkbox list with conflict dropdowns

### Section 4: Import Execution & Auto-Sync

**Import process:**
```rust
fn import_skill(source: &Path, name: &str, overwrite: bool) -> Result<ImportedSkill> {
    let dest = talent_skills_dir().join(name);

    if dest.exists() && !overwrite {
        return Err(Error::SkillExists(name.to_string()));
    }

    // Copy entire directory (SKILL.md + any bundled scripts/assets)
    copy_dir_recursive(source, &dest)?;

    Ok(ImportedSkill { name, path: dest })
}
```

**Post-import flow:**
1. Copy all selected skills to `~/.talent/skills/`
2. Call `SkillManager::refresh_skills()` to reload
3. Call `SkillManager::sync_all()` to distribute
4. Return results: `{ imported: 8, synced_to: 4, errors: [] }`

**UI feedback:**
- Progress bar during import
- Success toast: "Imported 8 skills, synced to 4 targets"
- Error details if any failures

### Section 5: Entry Points

**Empty state modification** (`SkillList.svelte`):

Current:
```
No skills found
Create your first skill to get started
```

New:
```
No skills found
┌─────────────────────┐  ┌──────────────────────┐
│  + Create New Skill │  │  ↓ Import Existing   │
└─────────────────────┘  └──────────────────────┘
         or scan Codex, Claude Code, Gemini...
```

**Toolbar addition:**

Current: `[Refresh] [Sync All] [New Skill]`

New: `[Refresh] [Sync All] [Import] [New Skill]`

### Data Types

```rust
// crates/talent-core/src/importer.rs

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredSkill {
    pub name: String,
    pub description: String,
    pub source_path: PathBuf,
    pub source_target: TargetKind,
    pub conflict: Option<ConflictInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictInfo {
    pub existing_path: PathBuf,
    pub existing_description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportSelection {
    pub name: String,
    pub source_path: PathBuf,
    pub resolution: ConflictResolution,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ConflictResolution {
    Import,      // No conflict, just import
    Skip,        // Conflict: keep existing
    Overwrite,   // Conflict: replace with incoming
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub imported: Vec<String>,
    pub skipped: Vec<String>,
    pub errors: Vec<(String, String)>,
    pub synced_to: usize,
}
```

## Implementation State

- State: in-progress
- Current step: 16 - Polish UI and add system tray support
- Last updated: 2026-01-29
- Checkpoints:
  - 2026-01-29 not-started Initial plan created
  - 2026-01-29 in-progress Steps 1-3 completed (workspace setup, error types, config)
  - 2026-01-29 in-progress Steps 4-6 completed (skill model, target model, validator). All 35 tests passing.
  - 2026-01-29 in-progress Steps 7-9 completed (syncer, watcher, manager). All 56 tests passing.
  - 2026-01-29 in-progress Step 10 completed (CLI implementation). All 56 tests passing. CLI commands: list, sync, doctor, targets, create, validate.
  - 2026-01-29 in-progress Steps 11-13 completed (Tauri backend, Svelte frontend, app verification). MVP complete. All 56 tests passing.
  - 2026-01-29 in-progress Step 14 completed (CodeMirror 6 editor). All 58 tests passing. Skill editing with live preview.
  - 2026-01-29 in-progress Step 15 completed (skill import functionality). All 67 tests passing. Import dialog with conflict resolution and FileMerge integration.

## Status Updates

- 2026-01-29 not-started Plan migrated to template format
- 2026-01-29 in-progress Completed batch 1: Rust workspace initialized, error types implemented with thiserror, config module with TOML serialization. All 9 tests passing.
- 2026-01-29 in-progress Completed batch 2: Skill model with frontmatter parsing (YAML between --- delimiters), target model with auto-detection for 6 CLIs (Claude Code, Codex, Gemini, Cursor, Amp, Goose), validation engine with kebab-case name enforcement. All 35 tests passing.
- 2026-01-29 in-progress Completed batch 3: Symlink syncer with create/remove/unchanged tracking, file watcher with debounced events, skill manager facade integrating all components. All 56 tests passing.
- 2026-01-29 in-progress Completed batch 4: CLI implementation with clap 4 - subcommands for list, sync, doctor, targets, create, validate. JSON output support. All 56 tests passing.
- 2026-01-29 in-progress Completed batch 5: Tauri v2 backend with 9 commands (get_skills, get_targets, sync_all, create_skill, validate_skill, validate_all, refresh_skills, delete_skill, get_stats). Svelte 5 frontend with responsive skill list and target list UI. App runs successfully. All 56 tests passing.
- 2026-01-29 in-progress Completed batch 6: CodeMirror 6 editor integration. Added get_skill_content and save_skill_content Tauri commands. Created SkillEditor Svelte component with markdown highlighting and dark theme. Split-panel UI with unsaved changes indicator. All 58 tests passing.
- 2026-01-29 in-progress Completed batch 7: Skill import functionality. Added importer module with discovery engine, conflict detection, and FileMerge integration. Created ImportDialog component with checkbox selection and conflict resolution dropdowns. Updated App.svelte with Import button in toolbar and enhanced empty state. All 67 tests passing (9 new importer tests).

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

- Unit tests for each core module (config, skill, target, validator, syncer, watcher, importer)
- Integration test: create skill -> validate -> sync -> verify symlinks
- Integration test: discover -> import -> sync -> verify skills appear in all targets
- Edge cases:
  - Missing SKILL.md
  - Invalid frontmatter
  - Broken symlinks
  - Target directory doesn't exist
  - Permission errors
  - Import edge cases:
    - Symlink pointing to Talent (should be filtered out)
    - Skill with bundled scripts/assets (should copy entire directory)
    - Conflict with existing skill (test all resolutions)
    - Empty target directory (no skills to import)
    - FileMerge not available (fallback behavior)

## Open Questions

- Should we support skill versioning?
- ~~How to handle conflicts when same skill name exists in target?~~ **Resolved**: Ask per-conflict with Skip/Overwrite/Compare options, FileMerge integration on macOS
- Should we support skill templates beyond the basic one?

## Risks

- **Symlink permissions on Windows**: Mitigation - Use junction points or copy fallback
- **File watcher reliability**: Mitigation - Manual sync button always available
- **Target CLI updates changing paths**: Mitigation - Auto-detect logic with fallback to manual config
