<script lang="ts">
  import { X, ChevronDown, ChevronRight, AlertTriangle, Check, Wrench } from 'lucide-svelte';
  import type { ScannedSkillInfo, FolderImportSelectionInfo } from './types';

  interface Props {
    skills: ScannedSkillInfo[];
    isImporting: boolean;
    onimport: (selections: FolderImportSelectionInfo[]) => void;
    onclose: () => void;
  }

  let { skills, isImporting, onimport, onclose }: Props = $props();

  // Selection state - initialized from props on mount (intentionally captures initial value)
  // svelte-ignore state_referenced_locally
  let selectedSkills = $state<Set<string>>(new Set(skills.map(s => s.source_path)));
  // svelte-ignore state_referenced_locally
  let applyFixes = $state<Record<string, boolean>>(
    Object.fromEntries(skills.filter(s => s.needs_fixes).map(s => [s.source_path, true]))
  );
  let expandedSkills = $state<Set<string>>(new Set());
  // svelte-ignore state_referenced_locally
  let conflictResolutions = $state<Record<string, 'overwrite' | 'skip'>>(
    Object.fromEntries(skills.filter(s => s.has_conflict).map(s => [s.source_path, 'skip']))
  );

  // Derived state
  let selectedCount = $derived(selectedSkills.size);
  let hasAnySelected = $derived(selectedCount > 0);

  function toggleSkill(path: string) {
    if (selectedSkills.has(path)) {
      selectedSkills.delete(path);
    } else {
      selectedSkills.add(path);
    }
    selectedSkills = new Set(selectedSkills);
  }

  function toggleAll() {
    if (selectedCount === skills.length) {
      selectedSkills = new Set();
    } else {
      selectedSkills = new Set(skills.map(s => s.source_path));
    }
  }

  function toggleExpand(path: string) {
    if (expandedSkills.has(path)) {
      expandedSkills.delete(path);
    } else {
      expandedSkills.add(path);
    }
    expandedSkills = new Set(expandedSkills);
  }

  function handleImport() {
    const selections: FolderImportSelectionInfo[] = skills
      .filter(s => selectedSkills.has(s.source_path))
      .map(skill => ({
        name: skill.name,
        source_path: skill.source_path,
        apply_fixes: applyFixes[skill.source_path] ?? false,
        resolution: skill.has_conflict
          ? conflictResolutions[skill.source_path] ?? 'skip'
          : 'import',
      }));

    onimport(selections);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onclose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={onclose}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h2>Import Skills from Folder</h2>
      <button class="close-button" onclick={onclose} disabled={isImporting}>
        <X size={18} strokeWidth={1.5} />
      </button>
    </div>

    <div class="modal-subheader">
      <label class="select-all">
        <input
          type="checkbox"
          checked={selectedCount === skills.length}
          indeterminate={selectedCount > 0 && selectedCount < skills.length}
          onchange={toggleAll}
          disabled={isImporting}
        />
        <span>
          {#if selectedCount === 0}
            Select all ({skills.length})
          {:else}
            {selectedCount} of {skills.length} selected
          {/if}
        </span>
      </label>
    </div>

    <div class="skill-list">
      {#each skills as skill (skill.source_path)}
        {@const isSelected = selectedSkills.has(skill.source_path)}
        {@const isExpanded = expandedSkills.has(skill.source_path)}
        {@const hasDetails = skill.needs_fixes || skill.has_conflict}

        <div class="skill-item" class:selected={isSelected}>
          <div class="skill-row">
            <label class="skill-checkbox">
              <input
                type="checkbox"
                checked={isSelected}
                onchange={() => toggleSkill(skill.source_path)}
                disabled={isImporting}
              />
            </label>

            <div class="skill-info" onclick={() => hasDetails && toggleExpand(skill.source_path)}>
              <div class="skill-name-row">
                <span class="skill-name">{skill.name}</span>
                <div class="skill-badges">
                  {#if skill.has_conflict}
                    <span class="badge badge-conflict" title="Conflicts with existing skill">
                      <AlertTriangle size={12} strokeWidth={2} />
                      Conflict
                    </span>
                  {/if}
                  {#if skill.needs_fixes}
                    <span class="badge badge-fixes" title="Has auto-fixable issues">
                      <Wrench size={12} strokeWidth={2} />
                      Needs fixes
                    </span>
                  {:else}
                    <span class="badge badge-valid">
                      <Check size={12} strokeWidth={2} />
                      Valid
                    </span>
                  {/if}
                </div>
              </div>
              <div class="skill-description">{skill.description}</div>
              <div class="skill-path">{skill.source_path}</div>
            </div>

            {#if hasDetails}
              <button
                class="expand-button"
                onclick={() => toggleExpand(skill.source_path)}
                disabled={isImporting}
              >
                {#if isExpanded}
                  <ChevronDown size={16} strokeWidth={1.5} />
                {:else}
                  <ChevronRight size={16} strokeWidth={1.5} />
                {/if}
              </button>
            {/if}
          </div>

          {#if isExpanded && hasDetails}
            <div class="skill-details">
              {#if skill.needs_fixes && skill.fixes_preview.length > 0}
                <div class="detail-section">
                  <div class="detail-header">
                    <label class="fix-toggle">
                      <input
                        type="checkbox"
                        checked={applyFixes[skill.source_path] ?? true}
                        onchange={(e) => applyFixes[skill.source_path] = e.currentTarget.checked}
                        disabled={isImporting}
                      />
                      <span>Apply fixes during import</span>
                    </label>
                  </div>
                  <ul class="fixes-list">
                    {#each skill.fixes_preview as fix}
                      <li>{fix}</li>
                    {/each}
                  </ul>
                </div>
              {/if}

              {#if skill.has_conflict}
                <div class="detail-section">
                  <div class="detail-header">Conflict Resolution</div>
                  <div class="conflict-info">
                    <p>A skill named "{skill.name}" already exists:</p>
                    <p class="existing-desc">{skill.existing_description}</p>
                  </div>
                  <div class="conflict-options">
                    <label>
                      <input
                        type="radio"
                        name="conflict-{skill.source_path}"
                        value="skip"
                        checked={conflictResolutions[skill.source_path] === 'skip'}
                        onchange={() => conflictResolutions[skill.source_path] = 'skip'}
                        disabled={isImporting}
                      />
                      <span>Skip (keep existing)</span>
                    </label>
                    <label>
                      <input
                        type="radio"
                        name="conflict-{skill.source_path}"
                        value="overwrite"
                        checked={conflictResolutions[skill.source_path] === 'overwrite'}
                        onchange={() => conflictResolutions[skill.source_path] = 'overwrite'}
                        disabled={isImporting}
                      />
                      <span>Overwrite existing</span>
                    </label>
                  </div>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      {/each}

      {#if skills.length === 0}
        <div class="empty-state">
          <p>No skills found in the selected folder.</p>
          <p class="hint">Skills must contain a SKILL.md file.</p>
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      <button class="cancel-button" onclick={onclose} disabled={isImporting}>
        Cancel
      </button>
      <button
        class="import-button"
        onclick={handleImport}
        disabled={!hasAnySelected || isImporting}
      >
        {#if isImporting}
          Importing...
        {:else}
          Import {selectedCount} Skill{selectedCount === 1 ? '' : 's'}
        {/if}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fade-in 0.15s ease-out;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal {
    width: 600px;
    max-width: 90vw;
    max-height: 80vh;
    background: var(--color-sidebar);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    animation: slide-up 0.2s ease-out;
  }

  @keyframes slide-up {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--color-border);
  }

  .modal-header h2 {
    margin: 0;
    font-size: var(--font-lg);
    font-weight: var(--font-weight-semibold);
  }

  .close-button {
    width: 28px;
    height: 28px;
    padding: 0;
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .close-button:hover:not(:disabled) {
    color: var(--color-text);
    background: var(--color-surface);
  }

  .close-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .modal-subheader {
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg);
  }

  .select-all {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .select-all input {
    cursor: pointer;
  }

  .skill-list {
    flex: 1;
    overflow-y: auto;
    min-height: 200px;
  }

  .skill-item {
    border-bottom: 1px solid var(--color-border);
  }

  .skill-item.selected {
    background: rgba(10, 132, 255, 0.08);
  }

  .skill-row {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-5);
  }

  .skill-checkbox {
    flex-shrink: 0;
    padding-top: 2px;
  }

  .skill-checkbox input {
    cursor: pointer;
  }

  .skill-info {
    flex: 1;
    min-width: 0;
    cursor: pointer;
  }

  .skill-name-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
    margin-bottom: 2px;
  }

  .skill-name {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
  }

  .skill-badges {
    display: flex;
    gap: var(--space-1);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 10px;
    font-weight: var(--font-weight-medium);
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }

  .badge-valid {
    background: rgba(48, 209, 88, 0.15);
    color: var(--color-success);
  }

  .badge-fixes {
    background: rgba(255, 159, 10, 0.15);
    color: var(--color-warning);
  }

  .badge-conflict {
    background: rgba(255, 69, 58, 0.15);
    color: var(--color-error);
  }

  .skill-description {
    font-size: var(--font-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .skill-path {
    font-size: 10px;
    color: var(--color-text-dim);
    font-family: 'SF Mono', Monaco, monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-top: 2px;
  }

  .expand-button {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    padding: 0;
    background: none;
    border: none;
    color: var(--color-text-dim);
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .expand-button:hover:not(:disabled) {
    color: var(--color-text);
    background: var(--color-surface);
  }

  .skill-details {
    padding: 0 var(--space-5) var(--space-3);
    padding-left: calc(var(--space-5) + 20px + var(--space-3));
    animation: expand 0.15s ease-out;
  }

  @keyframes expand {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .detail-section {
    background: var(--color-bg);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    margin-bottom: var(--space-2);
  }

  .detail-header {
    font-size: var(--font-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-muted);
    margin-bottom: var(--space-2);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .fix-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-secondary);
    cursor: pointer;
    text-transform: none;
    letter-spacing: normal;
  }

  .fixes-list {
    margin: 0;
    padding-left: var(--space-4);
    font-size: var(--font-xs);
    color: var(--color-text-muted);
    list-style: disc;
  }

  .fixes-list li {
    padding: 2px 0;
  }

  .conflict-info {
    font-size: var(--font-xs);
    color: var(--color-text-muted);
    margin-bottom: var(--space-2);
  }

  .conflict-info p {
    margin: 0 0 4px 0;
  }

  .existing-desc {
    padding-left: var(--space-3);
    border-left: 2px solid var(--color-border);
    color: var(--color-text-dim);
  }

  .conflict-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .conflict-options label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-xs);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .empty-state {
    padding: var(--space-8) var(--space-5);
    text-align: center;
    color: var(--color-text-muted);
  }

  .empty-state p {
    margin: 0;
    font-size: var(--font-sm);
  }

  .empty-state .hint {
    font-size: var(--font-xs);
    color: var(--color-text-dim);
    margin-top: var(--space-2);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-5);
    border-top: 1px solid var(--color-border);
    background: var(--color-bg);
  }

  .cancel-button,
  .import-button {
    padding: var(--space-2) var(--space-4);
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .cancel-button {
    background: var(--color-surface);
    color: var(--color-text-secondary);
  }

  .cancel-button:hover:not(:disabled) {
    background: var(--color-surface-hover);
  }

  .import-button {
    background: var(--color-primary);
    color: white;
  }

  .import-button:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .cancel-button:disabled,
  .import-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
