# Plan: Folder Import & Drag-Drop Support

Filename: plans/20260130-folder-import-and-drag-drop.md

## Context

- Allow users to import skills from file explorer and via drag-and-drop of folders
- Scan subfolders recursively for skills (folders containing SKILL.md matching agentskills.io spec)
- Auto-correct format/spec issues before import
- Import validated skills into Talent's central library (~/.talent/skills/)

## Interfaces

### Backend (Rust - talent-core)

- `Importer::scan_folder(path: &Path) -> Vec<ScannedSkill>`
  - Recursively scan a folder for skill directories (containing SKILL.md)
  - Return skill info with normalization suggestions

- `ScannedSkill` struct
  - name, description, source_path, needs_fixes, fixes_preview, conflict info

- `Importer::import_from_external(source: &Path, apply_fixes: bool) -> Result<PathBuf>`
  - Import a single skill from external folder
  - Optionally apply normalization fixes during import
  - Copy to ~/.talent/skills/{name}/

### Backend (Rust - Tauri)

- `scan_folder_for_skills(path: String) -> Vec<ScannedSkillInfo>`
  - Tauri command to expose folder scanning

- `import_from_folder(selections: Vec<FolderImportSelection>) -> ImportResultInfo`
  - Import selected skills from scan results

### Frontend (Svelte)

- Drag-drop zone overlay on main content area
- "Import Folder" button in sidebar action row
- Scan results modal/panel with:
  - List of discovered skills
  - Auto-fix toggle for each skill with issues
  - Conflict resolution (skip/overwrite)
  - Bulk import button

## Steps

1. [x] **Extend Tauri capabilities** - Add `dialog:allow-open` for folder picker dialog

2. [x] **Add ScannedSkill struct** - Create struct in importer.rs for external folder scan results, including normalization preview

3. [x] **Implement folder scanning** - Add `scan_folder()` to Importer that:
   - Uses WalkDir to recursively find SKILL.md files (max depth ~5)
   - Loads each skill leniently via `Skill::load_lenient()`
   - Runs `normalize_frontmatter()` to preview fixes
   - Checks for conflicts with existing skills
   - Returns list of ScannedSkill

4. [x] **Implement external import** - Add `import_from_external()` to Importer that:
   - Optionally applies frontmatter normalization
   - Renames folder to match kebab-case name
   - Copies to ~/.talent/skills/{name}/
   - Does NOT remove source (external import, not migration)

5. [x] **Add Tauri commands** - Create `scan_folder_for_skills` and `import_from_folder` commands in commands.rs

6. [x] **Add frontend types** - Add ScannedSkillInfo and FolderImportSelection to types.ts

7. [x] **Add API functions** - Add scanFolderForSkills and importFromFolder to api.ts

8. [x] **Implement drag-drop zone** - Add drop zone overlay in App.svelte that:
   - Activates on dragenter/dragover events
   - Shows visual feedback (drop zone overlay)
   - Uses Tauri file dialog or drop event to get folder path
   - Triggers folder scan

9. [x] **Add Import Folder button** - Add folder icon button to sidebar-actions-row that opens native folder picker

10. [x] **Build scan results UI** - Create ImportFromFolderModal.svelte component with:
    - List of discovered skills with checkboxes
    - Status badges (valid/needs-fixes/conflict)
    - Expand panel showing fix preview
    - "Apply fixes" toggle per skill
    - "Import Selected" button

11. [x] **Wire up import flow** - Connect UI to backend:
    - Folder drop/select → scan → show modal → import selected → refresh skills list → show snackbar

12. [ ] **Test end-to-end** - Manual testing with:
    - Empty folder (no skills found)
    - Single skill folder
    - Nested folders with multiple skills
    - Skills needing fixes (non-kebab names, missing fields, array metadata)
    - Conflict with existing skills

## Implementation State

- State: in-progress
- Current step: 12
- Last updated: 2026-01-30
- Checkpoints:
  - 2026-01-30 not-started Plan created
  - 2026-01-30 in-progress Steps 1-11 implemented

## Status Updates

- 2026-01-30 not-started Initial plan created

## Dependencies

- Internal: talent-core Importer, normalize_frontmatter(), Skill::load_lenient()
- External: Tauri dialog plugin, walkdir crate (already in use)

## Migration And Rollback

- No migrations needed (additive feature)
- Rollback: Remove new commands from invoke_handler, remove UI components

## Performance Budget

- Folder scan: <2s for 100 skill folders
- Import: <500ms per skill
- UI should show loading state during scan/import

## Rollout

- Single-phase rollout (all-or-nothing feature)
- No feature flags needed

## Observability

- Console logging for scan/import operations
- Snackbar notifications for success/error
- Error banner for partial failures

## Testing

- Unit tests:
  - `scan_folder()` with various folder structures
  - `import_from_external()` with/without fixes
  - Conflict detection
- Integration tests:
  - Full import flow via Tauri commands
- Edge cases:
  - Circular symlinks (WalkDir handles this)
  - Unreadable files (permission errors)
  - Very deep nesting (respect max_depth)
  - Unicode filenames
  - Skills with only SKILL.md (no other resources)

## Open Questions

- [x] Should max scan depth be configurable? → No, use reasonable default (5 levels)
- [x] Should we preserve source folder after import? → Yes (external import, not migration)
- [ ] Should drag-drop accept multiple folders? → Start with single folder, can extend later

## Risks

- **Risk**: User drags huge folder (e.g., home directory)
  - Mitigation: Add max file count limit, show warning for large scans, use async with progress

- **Risk**: Import fails midway through batch
  - Mitigation: Each import is independent; show partial results with errors

- **Risk**: Drag-drop events may conflict with editor text selection
  - Mitigation: Only activate drop zone when NOT in editor focus, or use distinct drop target area
