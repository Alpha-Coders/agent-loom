<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { getSkills, getTargets, syncAll, validateAll, refreshSkills, createSkill, deleteSkill, getStats, getSkillContent, saveSkillContent, validateSkill } from './lib/api';
  import type { SkillInfo, TargetInfo, SyncResult, StatsInfo } from './lib/types';
  import SkillEditor from './lib/SkillEditor.svelte';
  import ImportDialog from './lib/ImportDialog.svelte';

  // State using Svelte 5 runes
  let skills = $state<SkillInfo[]>([]);
  let targets = $state<TargetInfo[]>([]);
  let stats = $state<StatsInfo | null>(null);
  let lastSyncResults = $state<SyncResult[]>([]);

  let isLoading = $state(true);
  let isSyncing = $state(false);
  let error = $state<string | null>(null);

  // New skill form
  let showNewSkillForm = $state(false);
  let newSkillName = $state('');
  let newSkillDescription = $state('');

  // Import dialog
  let showImportDialog = $state(false);

  // Active tab
  let activeTab = $state<'skills' | 'targets'>('skills');

  // Editor state
  let editingSkill = $state<SkillInfo | null>(null);
  let editorContent = $state('');
  let originalContent = $state('');
  let isSaving = $state(false);

  let hasUnsavedChanges = $derived(editorContent !== originalContent);

  async function loadData() {
    try {
      isLoading = true;
      error = null;

      const [skillsData, targetsData, statsData] = await Promise.all([
        getSkills(),
        getTargets(),
        getStats()
      ]);

      skills = skillsData;
      targets = targetsData;
      stats = statsData;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isLoading = false;
    }
  }

  async function handleSync() {
    try {
      isSyncing = true;
      error = null;

      // First validate all skills
      skills = await validateAll();

      // Then sync
      lastSyncResults = await syncAll();

      // Refresh data
      await loadData();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isSyncing = false;
    }
  }

  async function handleRefresh() {
    try {
      error = null;
      skills = await refreshSkills();
      stats = await getStats();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleCreateSkill() {
    if (!newSkillName.trim() || !newSkillDescription.trim()) return;

    try {
      error = null;
      await createSkill(newSkillName.trim(), newSkillDescription.trim());
      newSkillName = '';
      newSkillDescription = '';
      showNewSkillForm = false;
      await loadData();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleDeleteSkill(name: string, event: MouseEvent) {
    event.stopPropagation();
    if (!confirm(`Delete skill "${name}"?`)) return;

    try {
      error = null;
      await deleteSkill(name);

      // Close editor if we deleted the skill being edited
      if (editingSkill?.name === name) {
        editingSkill = null;
        editorContent = '';
        originalContent = '';
      }

      await loadData();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleEditSkill(skill: SkillInfo) {
    // Warn if unsaved changes
    if (hasUnsavedChanges && editingSkill) {
      if (!confirm('You have unsaved changes. Discard and open another skill?')) {
        return;
      }
    }

    try {
      error = null;
      const content = await getSkillContent(skill.name);
      editingSkill = skill;
      editorContent = content;
      originalContent = content;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleSaveSkill() {
    if (!editingSkill) return;

    try {
      isSaving = true;
      error = null;

      // Save the content
      const updatedSkill = await saveSkillContent(editingSkill.name, editorContent);

      // Re-validate the skill
      const validatedSkill = await validateSkill(editingSkill.name);

      // Update local state
      originalContent = editorContent;
      editingSkill = validatedSkill;

      // Update skills list
      skills = skills.map(s => s.name === validatedSkill.name ? validatedSkill : s);
      stats = await getStats();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isSaving = false;
    }
  }

  function handleCloseEditor() {
    if (hasUnsavedChanges) {
      if (!confirm('You have unsaved changes. Discard?')) {
        return;
      }
    }
    editingSkill = null;
    editorContent = '';
    originalContent = '';
  }

  function handleEditorChange(content: string) {
    editorContent = content;
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'valid': return 'var(--color-success)';
      case 'invalid': return 'var(--color-error)';
      default: return 'var(--color-warning)';
    }
  }

  async function handleImportComplete() {
    // Refresh skills list after import
    await loadData();
  }

  function getSyncSummary(result: SyncResult): string {
    const parts = [];
    if (result.created.length > 0) parts.push(`+${result.created.length}`);
    if (result.removed.length > 0) parts.push(`-${result.removed.length}`);
    if (result.unchanged.length > 0) parts.push(`=${result.unchanged.length}`);
    return parts.join(' ') || 'No changes';
  }

  // Event listener cleanup
  let unlistenTraySync: UnlistenFn | null = null;

  onMount(async () => {
    loadData();

    // Listen for tray "Sync All" menu item
    unlistenTraySync = await listen('tray-sync-all', () => {
      handleSync();
    });
  });

  onDestroy(() => {
    if (unlistenTraySync) {
      unlistenTraySync();
    }
  });
</script>

<div class="app-container" class:editor-open={editingSkill !== null}>
  <main>
    <header>
      <h1>Talent</h1>
      <p class="subtitle">Agent Skills Manager</p>
    </header>

    {#if error}
      <div class="error-banner">
        <span>{error}</span>
        <button onclick={() => error = null}>Dismiss</button>
      </div>
    {/if}

    {#if stats}
      <div class="stats-bar">
        <div class="stat">
          <span class="stat-value">{stats.total_skills}</span>
          <span class="stat-label">Skills</span>
        </div>
        <div class="stat">
          <span class="stat-value">{stats.valid_skills}</span>
          <span class="stat-label">Valid</span>
        </div>
        <div class="stat">
          <span class="stat-value">{stats.enabled_targets}</span>
          <span class="stat-label">Targets</span>
        </div>
        <div class="stat">
          <span class="stat-value">{stats.is_watching ? 'On' : 'Off'}</span>
          <span class="stat-label">Auto-sync</span>
        </div>
      </div>
    {/if}

    <div class="toolbar">
      <div class="tabs">
        <button
          class="tab"
          class:active={activeTab === 'skills'}
          onclick={() => activeTab = 'skills'}
        >
          Skills ({skills.length})
        </button>
        <button
          class="tab"
          class:active={activeTab === 'targets'}
          onclick={() => activeTab = 'targets'}
        >
          Targets ({targets.length})
        </button>
      </div>

      <div class="actions">
        <button onclick={handleRefresh} disabled={isLoading}>
          Refresh
        </button>
        <button onclick={handleSync} disabled={isSyncing || isLoading} class="primary">
          {isSyncing ? 'Syncing...' : 'Sync All'}
        </button>
        <button onclick={() => showImportDialog = true}>
          Import
        </button>
        <button onclick={() => showNewSkillForm = !showNewSkillForm}>
          {showNewSkillForm ? 'Cancel' : 'New Skill'}
        </button>
      </div>
    </div>

    {#if showNewSkillForm}
      <div class="new-skill-form">
        <input
          type="text"
          placeholder="skill-name (kebab-case)"
          bind:value={newSkillName}
        />
        <input
          type="text"
          placeholder="Description"
          bind:value={newSkillDescription}
        />
        <button onclick={handleCreateSkill} class="primary" disabled={!newSkillName.trim() || !newSkillDescription.trim()}>
          Create
        </button>
      </div>
    {/if}

    {#if lastSyncResults.length > 0}
      <div class="sync-results">
        <h3>Last Sync Results</h3>
        <div class="sync-list">
          {#each lastSyncResults as result}
            <div class="sync-item">
              <span class="target-name">{result.target_name}</span>
              <span class="sync-summary">{getSyncSummary(result)}</span>
              {#if result.errors.length > 0}
                <span class="sync-errors">{result.errors.length} errors</span>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if isLoading}
      <div class="loading">Loading...</div>
    {:else if activeTab === 'skills'}
      <div class="skill-list">
        {#if skills.length === 0}
          <div class="empty-state">
            <p>No skills found</p>
            <div class="empty-actions">
              <button class="primary" onclick={() => showNewSkillForm = true}>
                + Create New Skill
              </button>
              <button onclick={() => showImportDialog = true}>
                Import Existing
              </button>
            </div>
            <p class="hint">or scan Codex, Claude Code, Gemini, Cursor, Amp, Goose...</p>
          </div>
        {:else}
          {#each skills as skill}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="skill-card"
              class:selected={editingSkill?.name === skill.name}
              onclick={() => handleEditSkill(skill)}
            >
              <div class="skill-header">
                <h3>{skill.name}</h3>
                <span
                  class="status-badge"
                  style="background-color: {getStatusColor(skill.validation_status)}"
                >
                  {skill.validation_status}
                </span>
              </div>
              <p class="skill-description">{skill.description}</p>
              {#if skill.tags.length > 0}
                <div class="skill-tags">
                  {#each skill.tags as tag}
                    <span class="tag">{tag}</span>
                  {/each}
                </div>
              {/if}
              <div class="skill-meta">
                {#if skill.version}
                  <span>v{skill.version}</span>
                {/if}
                {#if skill.author}
                  <span>by {skill.author}</span>
                {/if}
              </div>
              {#if skill.validation_errors.length > 0}
                <div class="validation-errors">
                  {#each skill.validation_errors as validationError}
                    <p class="error-text">{validationError}</p>
                  {/each}
                </div>
              {/if}
              <div class="skill-actions">
                <button class="danger" onclick={(e) => handleDeleteSkill(skill.name, e)}>
                  Delete
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {:else}
      <div class="target-list">
        {#if targets.length === 0}
          <div class="empty-state">
            <p>No targets detected</p>
            <p class="hint">Install a supported AI CLI tool (Claude Code, Codex, Gemini, Cursor, Amp, or Goose)</p>
          </div>
        {:else}
          {#each targets as target}
            <div class="target-card" class:disabled={!target.enabled}>
              <div class="target-header">
                <h3>{target.name}</h3>
                <span class="target-status" class:exists={target.exists}>
                  {target.exists ? 'Ready' : 'Not initialized'}
                </span>
              </div>
              <p class="target-path">{target.skills_path}</p>
              <div class="target-badges">
                {#if target.auto_detected}
                  <span class="badge">Auto-detected</span>
                {/if}
                {#if target.enabled}
                  <span class="badge enabled">Enabled</span>
                {:else}
                  <span class="badge disabled">Disabled</span>
                {/if}
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </main>

  <ImportDialog
    open={showImportDialog}
    onclose={() => showImportDialog = false}
    onimported={handleImportComplete}
  />

  {#if editingSkill}
    <aside class="editor-panel">
      <div class="editor-header">
        <div class="editor-title">
          <h2>{editingSkill.name}</h2>
          {#if hasUnsavedChanges}
            <span class="unsaved-indicator">Unsaved</span>
          {/if}
        </div>
        <div class="editor-actions">
          <button onclick={handleSaveSkill} disabled={isSaving || !hasUnsavedChanges} class="primary">
            {isSaving ? 'Saving...' : 'Save'}
          </button>
          <button onclick={handleCloseEditor}>
            Close
          </button>
        </div>
      </div>
      <div class="editor-container">
        <SkillEditor content={editorContent} onchange={handleEditorChange} />
      </div>
    </aside>
  {/if}
</div>

<style>
  :root {
    --color-bg: #0f172a;
    --color-surface: #1e293b;
    --color-border: #334155;
    --color-text: #f1f5f9;
    --color-text-muted: #94a3b8;
    --color-primary: #3b82f6;
    --color-primary-hover: #2563eb;
    --color-success: #22c55e;
    --color-warning: #f59e0b;
    --color-error: #ef4444;
    --color-danger: #dc2626;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    background-color: var(--color-bg);
    color: var(--color-text);
    overflow: hidden;
  }

  :global(html), :global(body), :global(#app) {
    height: 100%;
  }

  .app-container {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  main {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 2rem;
  }

  .app-container.editor-open main {
    max-width: 500px;
  }

  header {
    text-align: center;
    margin-bottom: 2rem;
  }

  header h1 {
    margin: 0;
    font-size: 2rem;
    color: var(--color-primary);
  }

  .subtitle {
    margin: 0.5rem 0 0;
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }

  .error-banner {
    background-color: var(--color-error);
    color: white;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .error-banner button {
    background: transparent;
    border: 1px solid white;
    color: white;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
  }

  .stats-bar {
    display: flex;
    gap: 1rem;
    justify-content: center;
    margin-bottom: 2rem;
    padding: 1rem;
    background-color: var(--color-surface);
    border-radius: 12px;
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0 1rem;
    border-right: 1px solid var(--color-border);
  }

  .stat:last-child {
    border-right: none;
  }

  .stat-value {
    font-size: 1.25rem;
    font-weight: bold;
    color: var(--color-primary);
  }

  .stat-label {
    font-size: 0.65rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
  }

  .tab {
    padding: 0.5rem 0.75rem;
    border: none;
    background: var(--color-surface);
    color: var(--color-text-muted);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.8rem;
  }

  .tab:hover {
    background: var(--color-border);
  }

  .tab.active {
    background: var(--color-primary);
    color: white;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  button {
    padding: 0.5rem 0.75rem;
    border: none;
    background: var(--color-surface);
    color: var(--color-text);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.8rem;
  }

  button:hover:not(:disabled) {
    background: var(--color-border);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.primary {
    background: var(--color-primary);
    color: white;
  }

  button.primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  button.danger {
    background: var(--color-danger);
    color: white;
  }

  button.danger:hover:not(:disabled) {
    background: #b91c1c;
  }

  .new-skill-form {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    padding: 0.75rem;
    background: var(--color-surface);
    border-radius: 8px;
  }

  .new-skill-form input {
    flex: 1;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 0.8rem;
  }

  .new-skill-form input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .sync-results {
    background: var(--color-surface);
    border-radius: 8px;
    padding: 0.75rem;
    margin-bottom: 1rem;
  }

  .sync-results h3 {
    margin: 0 0 0.5rem;
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .sync-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .sync-item {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    padding: 0.25rem 0.5rem;
    background: var(--color-bg);
    border-radius: 4px;
    font-size: 0.7rem;
  }

  .target-name {
    font-weight: 500;
  }

  .sync-summary {
    color: var(--color-text-muted);
  }

  .sync-errors {
    color: var(--color-error);
  }

  .loading {
    text-align: center;
    padding: 3rem;
    color: var(--color-text-muted);
  }

  .empty-state {
    text-align: center;
    padding: 2rem;
    background: var(--color-surface);
    border-radius: 12px;
  }

  .empty-state p {
    margin: 0;
  }

  .empty-state .hint {
    margin-top: 0.5rem;
    color: var(--color-text-muted);
    font-size: 0.8rem;
  }

  .empty-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: center;
    margin-top: 1rem;
  }

  .skill-list, .target-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .skill-card {
    background: var(--color-surface);
    border-radius: 12px;
    padding: 1rem;
    border: 1px solid var(--color-border);
    text-align: left;
    width: 100%;
    cursor: pointer;
  }

  .skill-card:hover {
    border-color: var(--color-primary);
  }

  .skill-card.selected {
    border-color: var(--color-primary);
    background: rgba(59, 130, 246, 0.1);
  }

  .target-card {
    background: var(--color-surface);
    border-radius: 12px;
    padding: 1rem;
    border: 1px solid var(--color-border);
  }

  .skill-header, .target-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .skill-header h3, .target-header h3 {
    margin: 0;
    font-size: 1rem;
  }

  .status-badge {
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-size: 0.65rem;
    font-weight: 500;
    color: white;
    text-transform: capitalize;
  }

  .skill-description {
    margin: 0 0 0.5rem;
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }

  .skill-tags {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .tag {
    padding: 0.1rem 0.4rem;
    background: var(--color-bg);
    border-radius: 4px;
    font-size: 0.65rem;
    color: var(--color-text-muted);
  }

  .skill-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.65rem;
    color: var(--color-text-muted);
    margin-bottom: 0.5rem;
  }

  .validation-errors {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--color-error);
    border-radius: 6px;
    padding: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .error-text {
    margin: 0;
    font-size: 0.7rem;
    color: var(--color-error);
  }

  .skill-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  .target-card.disabled {
    opacity: 0.6;
  }

  .target-status {
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-size: 0.65rem;
    background: var(--color-warning);
    color: white;
  }

  .target-status.exists {
    background: var(--color-success);
  }

  .target-path {
    margin: 0 0 0.5rem;
    font-size: 0.65rem;
    color: var(--color-text-muted);
    font-family: monospace;
    word-break: break-all;
  }

  .target-badges {
    display: flex;
    gap: 0.5rem;
  }

  .badge {
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    font-size: 0.65rem;
    background: var(--color-bg);
    color: var(--color-text-muted);
  }

  .badge.enabled {
    background: rgba(34, 197, 94, 0.2);
    color: var(--color-success);
  }

  .badge.disabled {
    background: rgba(239, 68, 68, 0.2);
    color: var(--color-error);
  }

  /* Editor Panel */
  .editor-panel {
    width: 50%;
    min-width: 400px;
    max-width: 700px;
    background: var(--color-surface);
    border-left: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .editor-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .editor-title h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .unsaved-indicator {
    padding: 0.2rem 0.5rem;
    background: var(--color-warning);
    color: white;
    border-radius: 4px;
    font-size: 0.65rem;
    font-weight: 500;
  }

  .editor-actions {
    display: flex;
    gap: 0.5rem;
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
    padding: 0;
  }
</style>
