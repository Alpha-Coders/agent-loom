<script lang="ts">
  import { discoverImportableSkills, importSkills, isFileMergeAvailable, launchFileMerge } from './api';
  import type { DiscoveredSkillInfo, ImportSelectionInfo } from './types';

  interface Props {
    open: boolean;
    onclose: () => void;
    onimported: () => void;
  }

  let { open, onclose, onimported }: Props = $props();

  type ResolutionType = 'import' | 'skip' | 'overwrite';

  interface SkillSelection {
    skill: DiscoveredSkillInfo;
    selected: boolean;
    resolution: ResolutionType;
  }

  let isScanning = $state(true);
  let isImporting = $state(false);
  let hasFileMerge = $state(false);
  let error = $state<string | null>(null);
  let selections = $state<SkillSelection[]>([]);
  let importResult = $state<{ imported: number; skipped: number; errors: number; synced_to: number } | null>(null);

  let selectedCount = $derived(
    selections.filter(s => s.selected && s.resolution !== 'skip').length
  );

  async function scan() {
    isScanning = true;
    error = null;
    importResult = null;

    try {
      const [discovered, fileMergeAvailable] = await Promise.all([
        discoverImportableSkills(),
        isFileMergeAvailable()
      ]);

      hasFileMerge = fileMergeAvailable;

      selections = discovered.map(skill => ({
        skill,
        selected: true,
        resolution: skill.has_conflict ? 'skip' : 'import' as ResolutionType
      }));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isScanning = false;
    }
  }

  async function handleImport() {
    const toImport: ImportSelectionInfo[] = selections
      .filter(s => s.selected)
      .map(s => ({
        name: s.skill.name,
        source_path: s.skill.source_path,
        resolution: s.resolution
      }));

    if (toImport.length === 0) return;

    isImporting = true;
    error = null;

    try {
      const result = await importSkills(toImport);
      importResult = {
        imported: result.imported.length,
        skipped: result.skipped.length,
        errors: result.errors.length,
        synced_to: result.synced_to
      };

      if (result.imported.length > 0) {
        onimported();
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isImporting = false;
    }
  }

  async function handleCompare(selection: SkillSelection) {
    if (!selection.skill.has_conflict) return;

    const existingPath = selection.skill.source_path.replace(/[^/]+$/, selection.skill.name);
    // For conflicts, the existing path is in the talent skills dir
    // We need to construct it properly - the backend will figure out the right paths

    try {
      // Open FileMerge - the backend will look up the existing path
      const talentSkillsDir = selection.skill.source_path.split('/').slice(0, -2).join('/').replace(/\.\w+$/, '.talent/skills');
      const existingSkillPath = `${talentSkillsDir.split('/').slice(0, -2).join('/')}/.talent/skills/${selection.skill.name}`;

      await launchFileMerge(existingSkillPath, selection.skill.source_path);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function handleClose() {
    if (!isImporting) {
      onclose();
    }
  }

  function toggleSelectAll() {
    const allSelected = selections.every(s => s.selected);
    selections = selections.map(s => ({ ...s, selected: !allSelected }));
  }

  // Scan when dialog opens
  $effect(() => {
    if (open) {
      scan();
    }
  });
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="dialog-overlay" onclick={handleClose}>
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <div class="dialog-header">
        <h2>Import Skills from Other Tools</h2>
        <button class="close-btn" onclick={handleClose} disabled={isImporting}>×</button>
      </div>

      <div class="dialog-content">
        {#if error}
          <div class="error-message">
            <span>{error}</span>
            <button onclick={() => error = null}>Dismiss</button>
          </div>
        {/if}

        {#if importResult}
          <div class="success-message">
            <p>
              Imported {importResult.imported} skill{importResult.imported !== 1 ? 's' : ''},
              synced to {importResult.synced_to} target{importResult.synced_to !== 1 ? 's' : ''}.
              {#if importResult.skipped > 0}
                Skipped {importResult.skipped}.
              {/if}
              {#if importResult.errors > 0}
                {importResult.errors} error{importResult.errors !== 1 ? 's' : ''}.
              {/if}
            </p>
            <button class="primary" onclick={handleClose}>Done</button>
          </div>
        {:else if isScanning}
          <div class="scanning">
            <div class="spinner"></div>
            <p>Scanning Codex, Claude Code, Gemini, Cursor, Amp, Goose...</p>
          </div>
        {:else if selections.length === 0}
          <div class="empty-state">
            <p>No importable skills found</p>
            <p class="hint">Skills must have a SKILL.md file with valid frontmatter</p>
          </div>
        {:else}
          <div class="skills-header">
            <label class="select-all">
              <input
                type="checkbox"
                checked={selections.every(s => s.selected)}
                indeterminate={selections.some(s => s.selected) && !selections.every(s => s.selected)}
                onchange={toggleSelectAll}
              />
              Select all
            </label>
            <span class="skills-count">
              Found {selections.length} skill{selections.length !== 1 ? 's' : ''}
            </span>
          </div>

          <div class="skills-list">
            {#each selections as selection, i}
              <div class="skill-row" class:has-conflict={selection.skill.has_conflict}>
                <label class="skill-checkbox">
                  <input
                    type="checkbox"
                    bind:checked={selections[i].selected}
                  />
                </label>

                <div class="skill-info">
                  <div class="skill-name-row">
                    <span class="skill-name">{selection.skill.name}</span>
                    <span class="skill-source">{selection.skill.source_target}</span>
                  </div>
                  <p class="skill-description">{selection.skill.description}</p>
                </div>

                <div class="skill-status">
                  {#if selection.skill.has_conflict}
                    <span class="badge exists">EXISTS</span>
                    <select
                      bind:value={selections[i].resolution}
                      disabled={!selection.selected}
                    >
                      <option value="skip">Skip</option>
                      <option value="overwrite">Overwrite</option>
                    </select>
                    {#if hasFileMerge}
                      <button
                        class="compare-btn"
                        onclick={() => handleCompare(selection)}
                        title="Compare in FileMerge"
                      >
                        Compare
                      </button>
                    {/if}
                  {:else}
                    <span class="badge new">NEW</span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      {#if !importResult && !isScanning && selections.length > 0}
        <div class="dialog-footer">
          <button onclick={handleClose} disabled={isImporting}>
            Cancel
          </button>
          <button
            class="primary"
            onclick={handleImport}
            disabled={isImporting || selectedCount === 0}
          >
            {isImporting ? 'Importing...' : `Import ${selectedCount}`}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: var(--color-surface, #1e293b);
    border-radius: 12px;
    width: 90%;
    max-width: 600px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--color-border, #334155);
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: var(--color-text-muted, #94a3b8);
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .close-btn:hover:not(:disabled) {
    color: var(--color-text, #f1f5f9);
  }

  .dialog-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.25rem;
    min-height: 200px;
  }

  .error-message {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid var(--color-error, #ef4444);
    border-radius: 8px;
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: var(--color-error, #ef4444);
    font-size: 0.85rem;
  }

  .error-message button {
    background: transparent;
    border: 1px solid currentColor;
    color: inherit;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
  }

  .success-message {
    text-align: center;
    padding: 2rem 1rem;
  }

  .success-message p {
    margin: 0 0 1rem;
    color: var(--color-success, #22c55e);
  }

  .scanning {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 1rem;
    color: var(--color-text-muted, #94a3b8);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--color-border, #334155);
    border-top-color: var(--color-primary, #3b82f6);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state {
    text-align: center;
    padding: 2rem 1rem;
  }

  .empty-state p {
    margin: 0;
  }

  .empty-state .hint {
    margin-top: 0.5rem;
    color: var(--color-text-muted, #94a3b8);
    font-size: 0.85rem;
  }

  .skills-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--color-border, #334155);
  }

  .select-all {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .skills-count {
    font-size: 0.8rem;
    color: var(--color-text-muted, #94a3b8);
  }

  .skills-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .skill-row {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--color-bg, #0f172a);
    border-radius: 8px;
    border: 1px solid var(--color-border, #334155);
  }

  .skill-row.has-conflict {
    border-color: var(--color-warning, #f59e0b);
    background: rgba(245, 158, 11, 0.05);
  }

  .skill-checkbox {
    display: flex;
    align-items: center;
    padding-top: 2px;
  }

  .skill-checkbox input {
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .skill-info {
    flex: 1;
    min-width: 0;
  }

  .skill-name-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .skill-name {
    font-weight: 500;
    font-size: 0.9rem;
  }

  .skill-source {
    font-size: 0.7rem;
    color: var(--color-text-muted, #94a3b8);
    background: var(--color-surface, #1e293b);
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
  }

  .skill-description {
    margin: 0;
    font-size: 0.8rem;
    color: var(--color-text-muted, #94a3b8);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .skill-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .badge {
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .badge.new {
    background: var(--color-success, #22c55e);
    color: white;
  }

  .badge.exists {
    background: var(--color-warning, #f59e0b);
    color: white;
  }

  .skill-status select {
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--color-border, #334155);
    border-radius: 4px;
    background: var(--color-surface, #1e293b);
    color: var(--color-text, #f1f5f9);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .skill-status select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .compare-btn {
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--color-border, #334155);
    border-radius: 4px;
    background: var(--color-surface, #1e293b);
    color: var(--color-text, #f1f5f9);
    font-size: 0.7rem;
    cursor: pointer;
  }

  .compare-btn:hover {
    background: var(--color-border, #334155);
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--color-border, #334155);
  }

  button {
    padding: 0.5rem 1rem;
    border: none;
    background: var(--color-surface, #1e293b);
    color: var(--color-text, #f1f5f9);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    transition: background 0.2s;
  }

  button:hover:not(:disabled) {
    background: var(--color-border, #334155);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.primary {
    background: var(--color-primary, #3b82f6);
    color: white;
  }

  button.primary:hover:not(:disabled) {
    background: var(--color-primary-hover, #2563eb);
  }
</style>
