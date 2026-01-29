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

  // Resizable pane state
  const MIN_MAIN_WIDTH = 320;
  const MIN_EDITOR_WIDTH = 400;
  const STORAGE_KEY_MAIN_WIDTH = 'talent-main-panel-width';

  let mainPanelWidth = $state<number | null>(null);
  let isResizing = $state(false);
  let containerRef: HTMLDivElement | null = null;

  function loadSavedWidth() {
    try {
      const saved = localStorage.getItem(STORAGE_KEY_MAIN_WIDTH);
      if (saved) {
        const width = parseInt(saved, 10);
        if (!isNaN(width) && width >= MIN_MAIN_WIDTH) {
          mainPanelWidth = width;
        }
      }
    } catch {
      // localStorage not available
    }
  }

  function savePanelWidth(width: number) {
    try {
      localStorage.setItem(STORAGE_KEY_MAIN_WIDTH, width.toString());
    } catch {
      // localStorage not available
    }
  }

  function handleResizeStart(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    document.addEventListener('mousemove', handleResizeMove);
    document.addEventListener('mouseup', handleResizeEnd);
  }

  function handleResizeMove(e: MouseEvent) {
    if (!isResizing || !containerRef) return;

    const containerRect = containerRef.getBoundingClientRect();
    const containerWidth = containerRect.width;
    let newWidth = e.clientX - containerRect.left;

    // Enforce minimum widths
    newWidth = Math.max(MIN_MAIN_WIDTH, newWidth);
    newWidth = Math.min(containerWidth - MIN_EDITOR_WIDTH - 8, newWidth); // 8px for handle

    mainPanelWidth = newWidth;
  }

  function handleResizeEnd() {
    isResizing = false;
    document.removeEventListener('mousemove', handleResizeMove);
    document.removeEventListener('mouseup', handleResizeEnd);

    if (mainPanelWidth !== null) {
      savePanelWidth(mainPanelWidth);
    }
  }

  async function loadData() {
    try {
      isLoading = true;
      error = null;

      // Load targets and stats in parallel, then validate skills
      const [targetsData, statsData] = await Promise.all([
        getTargets(),
        getStats()
      ]);

      // Validate all skills on load to get proper status
      const skillsData = await validateAll();

      skills = skillsData;
      targets = targetsData;
      stats = await getStats(); // Refresh stats after validation
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
      // Refresh skills from disk, then validate them
      await refreshSkills();
      skills = await validateAll();
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
    loadSavedWidth();
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

<div
  class="app-container"
  class:editor-open={editingSkill !== null}
  class:is-resizing={isResizing}
  bind:this={containerRef}
>
  <main style={editingSkill && mainPanelWidth ? `width: ${mainPanelWidth}px; flex: none;` : ''}>
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
      <div class="toolbar-divider"></div>
      <div class="actions">
        <button class="ghost" onclick={handleRefresh} disabled={isLoading}>
          Refresh
        </button>
        <button onclick={handleSync} disabled={isSyncing || isLoading} class="primary">
          {isSyncing ? 'Syncing...' : 'Sync All'}
        </button>
        <button class="ghost" onclick={() => showImportDialog = true}>
          Import
        </button>
        <button class="ghost" onclick={() => showNewSkillForm = !showNewSkillForm}>
          {showNewSkillForm ? 'Cancel' : 'New'}
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
      {@const totalCreated = lastSyncResults.reduce((sum, r) => sum + r.created.length, 0)}
      {@const totalRemoved = lastSyncResults.reduce((sum, r) => sum + r.removed.length, 0)}
      {@const totalErrors = lastSyncResults.reduce((sum, r) => sum + r.errors.length, 0)}
      {@const hasErrors = totalErrors > 0}
      <div class="sync-results" class:has-errors={hasErrors}>
        <div class="sync-results-header">
          <div class="sync-results-title">
            <span class="sync-icon">{hasErrors ? '⚠' : '✓'}</span>
            <h3>Sync {hasErrors ? 'Completed with Errors' : 'Complete'}</h3>
          </div>
          <div class="sync-results-summary">
            {#if totalCreated > 0}
              <span class="sync-stat created">+{totalCreated} created</span>
            {/if}
            {#if totalRemoved > 0}
              <span class="sync-stat removed">-{totalRemoved} removed</span>
            {/if}
            {#if totalErrors > 0}
              <span class="sync-stat errors">{totalErrors} errors</span>
            {/if}
            {#if totalCreated === 0 && totalRemoved === 0 && totalErrors === 0}
              <span class="sync-stat unchanged">All up to date</span>
            {/if}
          </div>
          <button class="sync-dismiss" onclick={() => lastSyncResults = []}>×</button>
        </div>
        <div class="sync-targets">
          {#each lastSyncResults as result}
            <div class="sync-target" class:has-errors={result.errors.length > 0}>
              <div class="sync-target-header">
                <span class="sync-target-name">{result.target_name}</span>
                <div class="sync-target-stats">
                  {#if result.created.length > 0}
                    <span class="stat-badge created">+{result.created.length}</span>
                  {/if}
                  {#if result.removed.length > 0}
                    <span class="stat-badge removed">-{result.removed.length}</span>
                  {/if}
                  {#if result.unchanged.length > 0}
                    <span class="stat-badge unchanged">={result.unchanged.length}</span>
                  {/if}
                  {#if result.errors.length > 0}
                    <span class="stat-badge errors">{result.errors.length} errors</span>
                  {/if}
                </div>
              </div>
              {#if result.errors.length > 0}
                <div class="sync-errors-list">
                  {#each result.errors as err}
                    <div class="sync-error-item">
                      {#if err.skill}<strong>{err.skill}:</strong> {/if}{err.message}
                    </div>
                  {/each}
                </div>
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
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resize-handle"
      onmousedown={handleResizeStart}
    ></div>
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
  /* ============================================
     DESIGN SYSTEM TOKENS
     ============================================ */
  :root {
    /* Spacing Scale (4px base) */
    --space-1: 4px;
    --space-2: 8px;
    --space-3: 12px;
    --space-4: 16px;
    --space-5: 20px;
    --space-6: 24px;
    --space-8: 32px;

    /* Typography Scale */
    --font-xs: 11px;
    --font-sm: 13px;
    --font-base: 14px;
    --font-lg: 16px;
    --font-xl: 20px;
    --font-2xl: 24px;

    --font-weight-normal: 400;
    --font-weight-medium: 500;
    --font-weight-semibold: 600;
    --font-weight-bold: 700;

    --line-height-tight: 1.25;
    --line-height-normal: 1.5;

    /* Border Radius Scale */
    --radius-sm: 4px;
    --radius-md: 6px;
    --radius-lg: 8px;
    --radius-xl: 12px;

    /* Colors - Slate palette (dark theme) */
    --color-bg: #0f172a;
    --color-surface: #1e293b;
    --color-surface-hover: #273548;
    --color-border: #334155;
    --color-border-strong: #475569;

    --color-text: #f1f5f9;
    --color-text-secondary: #cbd5e1;
    --color-text-muted: #94a3b8;
    --color-text-dim: #64748b;

    /* Accent Colors */
    --color-primary: #3b82f6;
    --color-primary-hover: #2563eb;
    --color-primary-muted: rgba(59, 130, 246, 0.15);

    --color-success: #22c55e;
    --color-success-muted: rgba(34, 197, 94, 0.15);

    --color-warning: #f59e0b;
    --color-warning-muted: rgba(245, 158, 11, 0.15);

    --color-error: #ef4444;
    --color-error-muted: rgba(239, 68, 68, 0.15);

    --color-danger: #dc2626;
    --color-danger-hover: #b91c1c;

    /* Shadows */
    --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
    --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.3);

    /* Transitions */
    --transition-fast: 0.1s ease;
    --transition-normal: 0.2s ease;

    /* Layout */
    --main-min-width: 320px;
    --editor-min-width: 400px;
  }

  /* ============================================
     GLOBAL STYLES
     ============================================ */
  :global(html), :global(body), :global(#app) {
    height: 100%;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: var(--font-base);
    line-height: var(--line-height-normal);
    background-color: var(--color-bg);
    color: var(--color-text);
    overflow: hidden;
    -webkit-font-smoothing: antialiased;
  }

  /* ============================================
     LAYOUT
     ============================================ */
  .app-container {
    display: flex;
    height: 100vh;
    min-width: 700px;
    overflow: hidden;
  }

  main {
    flex: 1 1 auto;
    min-width: var(--main-min-width);
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--space-4);
  }

  .app-container.editor-open main {
    flex: 0 0 auto;
    width: 360px;
    min-width: var(--main-min-width);
    max-width: 420px;
    padding: var(--space-3);
  }

  /* ============================================
     HEADER
     ============================================ */
  header {
    text-align: center;
    margin-bottom: var(--space-4);
  }

  header h1 {
    margin: 0;
    font-size: var(--font-xl);
    font-weight: var(--font-weight-bold);
    color: var(--color-primary);
    letter-spacing: -0.02em;
  }

  .subtitle {
    margin: var(--space-1) 0 0;
    color: var(--color-text-muted);
    font-size: var(--font-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* ============================================
     ERROR BANNER
     ============================================ */
  .error-banner {
    background-color: var(--color-error);
    color: white;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-lg);
    margin-bottom: var(--space-4);
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--font-sm);
  }

  .error-banner button {
    background: transparent;
    border: 1px solid rgba(255,255,255,0.5);
    color: white;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-xs);
  }

  .error-banner button:hover {
    background: rgba(255,255,255,0.1);
    border-color: white;
  }

  /* ============================================
     STATS BAR
     ============================================ */
  .stats-bar {
    display: flex;
    justify-content: center;
    margin-bottom: var(--space-4);
    padding: var(--space-3) var(--space-4);
    background-color: var(--color-surface);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0 var(--space-4);
    border-right: 1px solid var(--color-border);
  }

  .stat:last-child {
    border-right: none;
  }

  .stat-value {
    font-size: var(--font-lg);
    font-weight: var(--font-weight-bold);
    color: var(--color-primary);
  }

  .stat-label {
    font-size: var(--font-xs);
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  /* ============================================
     TOOLBAR
     ============================================ */
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
    padding: var(--space-2);
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border);
  }

  .tabs {
    display: flex;
    gap: var(--space-1);
    flex-shrink: 0;
  }

  .tab {
    padding: var(--space-2) var(--space-3);
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
  }

  .tab:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .tab.active {
    background: var(--color-primary);
    color: white;
  }

  .toolbar-divider {
    display: none;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin-left: auto;
  }

  /* Compact toolbar when editor is open */
  .app-container.editor-open .toolbar {
    flex-direction: column;
    align-items: stretch;
    padding: var(--space-2);
    gap: var(--space-2);
  }

  .app-container.editor-open .tabs {
    justify-content: center;
  }

  .app-container.editor-open .tab {
    padding: var(--space-2) var(--space-3);
    font-size: var(--font-xs);
    flex: 1;
    text-align: center;
  }

  .app-container.editor-open .actions {
    margin-left: 0;
    justify-content: center;
  }

  .app-container.editor-open .actions button {
    padding: var(--space-2) var(--space-3);
    font-size: var(--font-xs);
    flex: 1;
  }

  /* ============================================
     BUTTONS
     ============================================ */
  button {
    padding: var(--space-2) var(--space-3);
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    white-space: nowrap;
  }

  button:hover:not(:disabled) {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  button:disabled {
    opacity: 0.4;
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
    background: var(--color-danger-hover);
  }

  button.ghost {
    background: transparent;
    color: var(--color-text-muted);
  }

  button.ghost:hover:not(:disabled) {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  /* ============================================
     NEW SKILL FORM
     ============================================ */
  .new-skill-form {
    display: flex;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
    padding: var(--space-3);
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border);
  }

  .new-skill-form input {
    flex: 1;
    min-width: 0;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg);
    color: var(--color-text);
    font-size: var(--font-sm);
    transition: border-color var(--transition-fast);
  }

  .new-skill-form input::placeholder {
    color: var(--color-text-dim);
  }

  .new-skill-form input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  /* ============================================
     SYNC RESULTS
     ============================================ */
  .sync-results {
    background: var(--color-success-muted);
    border-radius: var(--radius-lg);
    padding: var(--space-3);
    margin-bottom: var(--space-4);
    border: 1px solid var(--color-success);
  }

  .sync-results.has-errors {
    background: var(--color-error-muted);
    border-color: var(--color-error);
  }

  .sync-results-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .sync-results-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .sync-icon {
    font-size: var(--font-lg);
  }

  .sync-results:not(.has-errors) .sync-icon {
    color: var(--color-success);
  }

  .sync-results.has-errors .sync-icon {
    color: var(--color-error);
  }

  .sync-results h3 {
    margin: 0;
    font-size: var(--font-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
  }

  .sync-results-summary {
    display: flex;
    gap: var(--space-2);
    margin-left: auto;
  }

  .sync-stat {
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    background: var(--color-bg);
  }

  .sync-stat.created {
    color: var(--color-success);
  }

  .sync-stat.removed {
    color: var(--color-warning);
  }

  .sync-stat.errors {
    color: var(--color-error);
  }

  .sync-stat.unchanged {
    color: var(--color-text-muted);
  }

  .sync-dismiss {
    padding: var(--space-1) var(--space-2);
    background: transparent;
    color: var(--color-text-muted);
    font-size: var(--font-lg);
    line-height: 1;
  }

  .sync-dismiss:hover {
    color: var(--color-text);
    background: var(--color-surface-hover);
  }

  .sync-targets {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .sync-target {
    background: var(--color-bg);
    border-radius: var(--radius-md);
    padding: var(--space-2) var(--space-3);
    border: 1px solid transparent;
  }

  .sync-target.has-errors {
    border-color: var(--color-error);
  }

  .sync-target-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .sync-target-name {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
  }

  .sync-target-stats {
    display: flex;
    gap: var(--space-1);
  }

  .stat-badge {
    font-size: var(--font-xs);
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    font-weight: var(--font-weight-medium);
  }

  .stat-badge.created {
    background: var(--color-success-muted);
    color: var(--color-success);
  }

  .stat-badge.removed {
    background: var(--color-warning-muted);
    color: var(--color-warning);
  }

  .stat-badge.unchanged {
    background: var(--color-surface);
    color: var(--color-text-muted);
  }

  .stat-badge.errors {
    background: var(--color-error-muted);
    color: var(--color-error);
  }

  .sync-errors-list {
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--color-border);
  }

  .sync-error-item {
    font-size: var(--font-xs);
    color: var(--color-error);
    padding: var(--space-1) 0;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, monospace;
    word-break: break-all;
  }

  .sync-error-item + .sync-error-item {
    border-top: 1px dashed var(--color-border);
  }

  .sync-error-item strong {
    color: var(--color-text-secondary);
  }

  /* ============================================
     LISTS
     ============================================ */
  .loading {
    text-align: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
    font-size: var(--font-sm);
  }

  .empty-state {
    text-align: center;
    padding: var(--space-8);
    background: var(--color-surface);
    border-radius: var(--radius-xl);
    border: 1px solid var(--color-border);
  }

  .empty-state p {
    margin: 0;
    color: var(--color-text-secondary);
  }

  .empty-state .hint {
    margin-top: var(--space-2);
    color: var(--color-text-muted);
    font-size: var(--font-sm);
  }

  .empty-actions {
    display: flex;
    gap: var(--space-3);
    justify-content: center;
    margin-top: var(--space-4);
  }

  .skill-list, .target-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  /* ============================================
     SKILL CARD
     ============================================ */
  .skill-card {
    background: var(--color-surface);
    border-radius: var(--radius-xl);
    padding: var(--space-4);
    border: 1px solid var(--color-border);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .skill-card:hover {
    border-color: var(--color-primary);
    background: var(--color-surface-hover);
  }

  .skill-card.selected {
    border-color: var(--color-primary);
    background: var(--color-primary-muted);
  }

  .skill-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .skill-header h3 {
    margin: 0;
    font-size: var(--font-base);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
  }

  .status-badge {
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    color: white;
    text-transform: capitalize;
    flex-shrink: 0;
  }

  .skill-description {
    margin: 0 0 var(--space-3);
    color: var(--color-text-muted);
    font-size: var(--font-sm);
    line-height: var(--line-height-normal);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .skill-tags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin-bottom: var(--space-2);
  }

  .tag {
    padding: var(--space-1) var(--space-2);
    background: var(--color-bg);
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    color: var(--color-text-muted);
  }

  .skill-meta {
    display: flex;
    gap: var(--space-3);
    font-size: var(--font-xs);
    color: var(--color-text-dim);
    margin-bottom: var(--space-2);
  }

  .validation-errors {
    background: var(--color-error-muted);
    border: 1px solid var(--color-error);
    border-radius: var(--radius-md);
    padding: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .error-text {
    margin: 0;
    font-size: var(--font-xs);
    color: var(--color-error);
  }

  .skill-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--color-border);
  }

  /* ============================================
     TARGET CARD
     ============================================ */
  .target-card {
    background: var(--color-surface);
    border-radius: var(--radius-xl);
    padding: var(--space-4);
    border: 1px solid var(--color-border);
  }

  .target-card.disabled {
    opacity: 0.5;
  }

  .target-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .target-header h3 {
    margin: 0;
    font-size: var(--font-base);
    font-weight: var(--font-weight-semibold);
  }

  .target-status {
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    background: var(--color-warning);
    color: white;
  }

  .target-status.exists {
    background: var(--color-success);
  }

  .target-path {
    margin: 0 0 var(--space-2);
    font-size: var(--font-xs);
    color: var(--color-text-dim);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, monospace;
    word-break: break-all;
  }

  .target-badges {
    display: flex;
    gap: var(--space-2);
  }

  .badge {
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    background: var(--color-bg);
    color: var(--color-text-muted);
  }

  .badge.enabled {
    background: var(--color-success-muted);
    color: var(--color-success);
  }

  .badge.disabled {
    background: var(--color-error-muted);
    color: var(--color-error);
  }

  /* ============================================
     RESIZE HANDLE
     ============================================ */
  .resize-handle {
    width: 4px;
    background: var(--color-border);
    cursor: col-resize;
    flex-shrink: 0;
    transition: background-color var(--transition-fast);
  }

  .resize-handle:hover {
    background: var(--color-primary);
  }

  .app-container.is-resizing {
    cursor: col-resize;
    user-select: none;
  }

  .app-container.is-resizing .resize-handle {
    background: var(--color-primary);
  }

  /* ============================================
     EDITOR PANEL
     ============================================ */
  .editor-panel {
    flex: 1 1 auto;
    min-width: var(--editor-min-width);
    background: var(--color-surface);
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-3) var(--space-4);
    background: var(--color-bg);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .editor-title {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  .editor-title h2 {
    margin: 0;
    font-size: var(--font-base);
    font-weight: var(--font-weight-semibold);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .unsaved-indicator {
    padding: var(--space-1) var(--space-2);
    background: var(--color-warning);
    color: white;
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    flex-shrink: 0;
  }

  .editor-actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
  }
</style>
