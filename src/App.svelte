<script lang="ts">
  import { onMount } from 'svelte';
  import { getSkills, getTargets, syncAll, validateAll, refreshSkills, createSkill, deleteSkill, getStats } from './lib/api';
  import type { SkillInfo, TargetInfo, SyncResult, StatsInfo } from './lib/types';

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

  // Active tab
  let activeTab = $state<'skills' | 'targets'>('skills');

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

  async function handleDeleteSkill(name: string) {
    if (!confirm(`Delete skill "${name}"?`)) return;

    try {
      error = null;
      await deleteSkill(name);
      await loadData();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'valid': return 'var(--color-success)';
      case 'invalid': return 'var(--color-error)';
      default: return 'var(--color-warning)';
    }
  }

  function getSyncSummary(result: SyncResult): string {
    const parts = [];
    if (result.created.length > 0) parts.push(`+${result.created.length}`);
    if (result.removed.length > 0) parts.push(`-${result.removed.length}`);
    if (result.unchanged.length > 0) parts.push(`=${result.unchanged.length}`);
    return parts.join(' ') || 'No changes';
  }

  onMount(() => {
    loadData();
  });
</script>

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
          <p class="hint">Create your first skill to get started</p>
        </div>
      {:else}
        {#each skills as skill}
          <div class="skill-card">
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
              <button class="danger" onclick={() => handleDeleteSkill(skill.name)}>
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
  }

  main {
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem;
  }

  header {
    text-align: center;
    margin-bottom: 2rem;
  }

  header h1 {
    margin: 0;
    font-size: 2.5rem;
    color: var(--color-primary);
  }

  .subtitle {
    margin: 0.5rem 0 0;
    color: var(--color-text-muted);
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
    padding: 0 1.5rem;
    border-right: 1px solid var(--color-border);
  }

  .stat:last-child {
    border-right: none;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: bold;
    color: var(--color-primary);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
  }

  .tab {
    padding: 0.5rem 1rem;
    border: none;
    background: var(--color-surface);
    color: var(--color-text-muted);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
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
    padding: 0.5rem 1rem;
    border: none;
    background: var(--color-surface);
    color: var(--color-text);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.875rem;
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
    padding: 1rem;
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
    font-size: 0.875rem;
  }

  .new-skill-form input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .sync-results {
    background: var(--color-surface);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .sync-results h3 {
    margin: 0 0 0.75rem;
    font-size: 0.875rem;
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
    font-size: 0.75rem;
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
    padding: 3rem;
    background: var(--color-surface);
    border-radius: 12px;
  }

  .empty-state p {
    margin: 0;
  }

  .empty-state .hint {
    margin-top: 0.5rem;
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }

  .skill-list, .target-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .skill-card, .target-card {
    background: var(--color-surface);
    border-radius: 12px;
    padding: 1.25rem;
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
    font-size: 1.125rem;
  }

  .status-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
    color: white;
    text-transform: capitalize;
  }

  .skill-description {
    margin: 0 0 0.75rem;
    color: var(--color-text-muted);
  }

  .skill-tags {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .tag {
    padding: 0.125rem 0.5rem;
    background: var(--color-bg);
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .skill-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.75rem;
    color: var(--color-text-muted);
    margin-bottom: 0.75rem;
  }

  .validation-errors {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--color-error);
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.75rem;
  }

  .error-text {
    margin: 0;
    font-size: 0.75rem;
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
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    background: var(--color-warning);
    color: white;
  }

  .target-status.exists {
    background: var(--color-success);
  }

  .target-path {
    margin: 0 0 0.75rem;
    font-size: 0.75rem;
    color: var(--color-text-muted);
    font-family: monospace;
    word-break: break-all;
  }

  .target-badges {
    display: flex;
    gap: 0.5rem;
  }

  .badge {
    padding: 0.125rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
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
</style>
