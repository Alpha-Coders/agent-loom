# Plan: Simplified Add Target Flow

Filename: plans/20260130-simplified-add-target.md

## Context

- Current "Add Target" flow requires selecting from a hardcoded dropdown of target types + entering a path manually
- This is cumbersome and requires maintaining the TargetKind enum for custom targets
- Simplify to: user picks a folder via file explorer, that folder becomes a sync target
- Remove dependency on the hardcoded target type list for custom targets

## Current Implementation

**Frontend (App.svelte):**
- `showAddTargetForm` state shows inline form
- `availableTargetTypes` fetched from backend (filters out already-added types)
- Form has `<select>` for target type + `<input>` for path
- Calls `addCustomTarget(selectedTargetType, customTargetPath)`

**Backend:**
- `get_available_target_types()` returns `Vec<(String, String)>` from `TargetKind::all()`
- `add_custom_target(target_type, path)` adds target to manager
- `TargetKind` enum has 12 hardcoded CLI tools

## New Implementation

**Frontend:**
- Replace inline form with single "Add Target" button
- Button opens native folder picker dialog
- Selected folder path is sent directly to backend
- Target name derived from folder path (e.g., last component or parent/.folder pattern)

**Backend:**
- New command `add_folder_target(path: String) -> Result<TargetInfo>`
- Creates a custom target using the folder path
- Auto-detect if path matches a known TargetKind pattern (optional enhancement)
- Target ID/name derived from path

## Interfaces

### Backend (Tauri)

- `add_folder_target(path: String) -> Result<TargetInfo, String>`
  - Add any folder as a sync target
  - Derive target name from path
  - Return the created target info

### Frontend (Svelte)

- Remove: `availableTargetTypes` state, `selectedTargetType` state, `customTargetPath` state
- Remove: `handleShowAddTarget()`, `handleAddTarget()`
- Remove: `getAvailableTargetTypes` API call
- Add: `handleAddFolderTarget()` - opens folder picker, calls backend

## Steps

1. [x] **Update backend** - Add `add_folder_target` command in commands.rs
   - Accept path string
   - Create target with auto-generated ID (e.g., "folder-{name}")
   - Display name from folder path

2. [x] **Update manager** - Add `add_folder_as_target(path: PathBuf)` method
   - Creates Target with custom path
   - Generate unique ID from path
   - Add to targets list and persist

3. [x] **Update frontend API** - Add `addFolderTarget(path: string)` in api.ts

4. [x] **Update frontend UI** - Replace add target form with folder picker flow
   - Remove dropdown and input field
   - Remove `handleShowAddTarget()` function
   - Update button to directly open folder picker
   - On folder selection, call `addFolderTarget()`
   - Show success snackbar with target name

5. [x] **Remove dead code**
   - Removed old form state variables from App.svelte
   - Removed unused CSS for add target form
   - `get_available_target_types` kept for potential future use

6. [ ] **Test end-to-end**
   - Add custom folder target
   - Verify it appears in sidebar
   - Verify sync works to new target
   - Verify persistence across app restart

## Implementation State

- State: in-progress
- Current step: 6
- Last updated: 2026-01-30
- Checkpoints:
  - 2026-01-30 not-started Plan created
  - 2026-01-30 in-progress Steps 1-5 completed

## Status Updates

- 2026-01-30 not-started Initial plan created
- 2026-01-30 in-progress Backend and frontend implementation complete

## Dependencies

- Internal: Manager, Target, Tauri dialog plugin
- External: None new

## Migration And Rollback

- No data migration needed
- Existing custom targets (if any) continue to work
- Rollback: Revert code changes, restore dropdown UI

## Performance Budget

- Folder picker: Native OS dialog, instant
- Add target: <100ms

## Rollout

- Single-phase (replace old UI with new)
- No feature flags

## Observability

- Console log when target added
- Snackbar notification on success/error

## Testing

- Unit tests:
  - `add_folder_as_target()` creates valid target
  - ID generation is unique and stable
- Integration:
  - Full flow from folder picker to target sync
- Edge cases:
  - Folder doesn't exist (create on first sync)
  - Folder already added as target (show error)
  - Path with special characters/unicode

## Open Questions

- [ ] Should we auto-detect known target types from path? (e.g., ~/.claude/skills → Claude Code)
  - Recommendation: Yes, as enhancement - show detected name instead of generic folder name
- [ ] Should custom targets have editable names?
  - Recommendation: No for MVP, can add later
- [ ] What happens if user adds a folder inside ~/.talent/skills?
  - Recommendation: Warn or prevent (would cause circular sync)

## Risks

- **Risk**: User adds very large folder as target
  - Mitigation: Only symlinks are created, not copies; warn if many existing subfolders

- **Risk**: User removes folder after adding as target
  - Mitigation: Already handled - target shows as "not exists" in UI
