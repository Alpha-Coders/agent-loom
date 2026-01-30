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
  let conflictCount = $derived(skills.filter(s => s.has_conflict).length);
  let fixesCount = $derived(skills.filter(s => s.needs_fixes && !s.has_conflict).length);
  let needsAttentionCount = $derived(skills.filter(s => s.has_conflict || s.needs_fixes).length);

  // Auto-expand skills that need attention on mount
  $effect(() => {
    const needsAttention = skills.filter(s => s.has_conflict || s.needs_fixes);
    if (needsAttention.length > 0 && needsAttention.length <= 5) {
      expandedSkills = new Set(needsAttention.map(s => s.source_path));
    }
  });

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
      {#if needsAttentionCount > 0}
        <div class="attention-banner">
          <AlertTriangle size={14} strokeWidth={2} />
          <span>
            {#if conflictCount > 0 && fixesCount > 0}
              {conflictCount} conflict{conflictCount === 1 ? '' : 's'}, {fixesCount} need{fixesCount === 1 ? 's' : ''} fixes
            {:else if conflictCount > 0}
              {conflictCount} conflict{conflictCount === 1 ? '' : 's'} to resolve
            {:else}
              {fixesCount} skill{fixesCount === 1 ? '' : 's'} need{fixesCount === 1 ? 's' : ''} fixes
            {/if}
          </span>
        </div>
      {/if}
    </div>

    <div class="skill-list">
      {#each skills as skill (skill.source_path)}
        {@const isSelected = selectedSkills.has(skill.source_path)}
        {@const isExpanded = expandedSkills.has(skill.source_path)}
        {@const hasDetails = skill.needs_fixes || skill.has_conflict}

        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="skill-item"
          class:selected={isSelected}
          class:needs-attention={skill.has_conflict || skill.needs_fixes}
          onclick={() => !isImporting && toggleSkill(skill.source_path)}
        >
          <div class="skill-row">
            <div class="skill-checkbox" class:checked={isSelected}>
              {#if isSelected}
                <Check size={12} strokeWidth={3} />
              {/if}
            </div>

            <div class="skill-info">
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
            </div>

            {#if hasDetails}
              <button
                class="expand-button"
                onclick={(e) => { e.stopPropagation(); toggleExpand(skill.source_path); }}
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
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="skill-details" onclick={(e) => e.stopPropagation()}>
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
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
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
    width: 640px;
    max-width: 90vw;
    max-height: 80vh;
    background: var(--color-sidebar);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow:
      0 24px 80px rgba(0, 0, 0, 0.5),
      0 0 0 1px rgba(255, 255, 255, 0.05),
      inset 0 1px 0 rgba(255, 255, 255, 0.05);
    display: flex;
    flex-direction: column;
    animation: modal-appear 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
    overflow: hidden;
  }

  @keyframes modal-appear {
    from {
      opacity: 0;
      transform: translateY(16px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-5) var(--space-6);
    background: var(--color-sidebar);
  }

  .modal-header h2 {
    margin: 0;
    font-size: var(--font-xl);
    font-weight: var(--font-weight-semibold);
  }

  .close-button {
    width: 32px;
    height: 32px;
    padding: 0;
    background: var(--color-surface);
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .close-button:hover:not(:disabled) {
    color: var(--color-text);
    background: var(--color-surface-hover);
  }

  .close-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .modal-subheader {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-6);
    border-top: 1px solid var(--color-border);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .select-all {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
    cursor: pointer;
  }

  .select-all input {
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: var(--color-primary);
  }

  .attention-banner {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: rgba(255, 159, 10, 0.1);
    border-radius: var(--radius-md);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-warning);
  }

  .skill-list {
    flex: 1;
    overflow-y: auto;
    min-height: 200px;
    background: var(--color-bg);
  }

  .skill-item {
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .skill-item:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .skill-item.selected {
    background: var(--color-primary-muted);
  }

  .skill-item.selected:hover {
    background: rgba(10, 132, 255, 0.22);
  }

  .skill-item.needs-attention {
    border-left: 3px solid var(--color-warning);
  }

  .skill-item.needs-attention .skill-row {
    padding-left: calc(var(--space-6) - 3px);
  }

  .skill-row {
    display: flex;
    align-items: flex-start;
    gap: var(--space-4);
    padding: var(--space-5) var(--space-6);
  }

  .skill-checkbox {
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    margin-top: 2px;
    border: 2px solid var(--color-text-dim);
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
    background: transparent;
  }

  .skill-checkbox.checked {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .skill-info {
    flex: 1;
    min-width: 0;
  }

  .skill-name-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex-wrap: wrap;
    margin-bottom: var(--space-2);
  }

  .skill-name {
    font-size: var(--font-base);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
  }

  .skill-badges {
    display: flex;
    gap: var(--space-2);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }

  .badge-valid {
    background: rgba(48, 209, 88, 0.15);
    color: #1a7f37;
  }

  :global([data-theme="dark"]) .badge-valid {
    color: var(--color-success);
  }

  .badge-fixes {
    background: rgba(255, 159, 10, 0.18);
    color: #9a6700;
  }

  :global([data-theme="dark"]) .badge-fixes {
    color: var(--color-warning);
  }

  .badge-conflict {
    background: rgba(255, 69, 58, 0.15);
    color: #cf222e;
  }

  :global([data-theme="dark"]) .badge-conflict {
    color: var(--color-error);
  }

  .skill-description {
    font-size: var(--font-sm);
    color: var(--color-text-secondary);
    line-height: 1.4;
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
    gap: var(--space-3);
    padding: var(--space-5) var(--space-6);
    border-top: 1px solid var(--color-border);
    background: var(--color-sidebar);
  }

  .cancel-button,
  .import-button {
    padding: var(--space-3) var(--space-5);
    border: none;
    border-radius: var(--radius-lg);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .cancel-button {
    background: var(--color-surface);
    color: var(--color-text);
  }

  .cancel-button:hover:not(:disabled) {
    background: var(--color-surface-hover);
  }

  .import-button {
    background: var(--color-primary);
    color: white;
    box-shadow: 0 2px 8px rgba(10, 132, 255, 0.3);
  }

  .import-button:hover:not(:disabled) {
    background: var(--color-primary-hover);
    box-shadow: 0 4px 12px rgba(10, 132, 255, 0.4);
  }

  .import-button:active:not(:disabled) {
    transform: scale(0.98);
  }

  .cancel-button:disabled,
  .import-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    box-shadow: none;
  }
</style>
