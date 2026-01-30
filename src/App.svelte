<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { getSkills, getTargets, syncAll, validateAll, refreshSkills, createSkill, deleteSkill, renameSkill, getStats, getSkillContent, saveSkillContent, validateSkill, importAllSkills, toggleTarget, getAvailableTargetTypes, addCustomTarget, fixSkill } from './lib/api';
  import type { SkillInfo, TargetInfo, SyncResult, StatsInfo, ImportResultInfo } from './lib/types';
  import SkillEditor from './lib/SkillEditor.svelte';

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

  // Import state
  let isImporting = $state(false);
  let lastImportResult = $state<ImportResultInfo | null>(null);

  // Sidebar filter
  let activeFilter = $state<'all' | 'valid' | 'invalid'>('all');

  // Add target state
  let showAddTargetForm = $state(false);
  let availableTargetTypes = $state<[string, string][]>([]);
  let selectedTargetType = $state('');
  let customTargetPath = $state('');

  // Editor state
  let editingSkill = $state<SkillInfo | null>(null);
  let editorContent = $state('');
  let originalContent = $state('');
  let isSaving = $state(false);
  let isFixing = $state(false);

  // Snackbar/toast state
  interface Snackbar {
    id: number;
    message: string;
    type: 'success' | 'info' | 'warning';
  }
  let snackbars = $state<Snackbar[]>([]);
  let snackbarId = 0;

  function showSnackbar(message: string, type: 'success' | 'info' | 'warning' = 'success', duration = 3000) {
    const id = ++snackbarId;
    snackbars = [...snackbars, { id, message, type }];

    setTimeout(() => {
      snackbars = snackbars.filter(s => s.id !== id);
    }, duration);
  }

  function dismissSnackbar(id: number) {
    snackbars = snackbars.filter(s => s.id !== id);
  }

  let hasUnsavedChanges = $derived(editorContent !== originalContent);

  // Filtered skills based on sidebar selection
  let filteredSkills = $derived(() => {
    switch (activeFilter) {
      case 'valid':
        return skills.filter(s => s.validation_status === 'valid');
      case 'invalid':
        return skills.filter(s => s.validation_status === 'invalid');
      default:
        return skills;
    }
  });

  // Counts for sidebar
  let validCount = $derived(skills.filter(s => s.validation_status === 'valid').length);
  let invalidCount = $derived(skills.filter(s => s.validation_status === 'invalid').length);
  let enabledTargetsCount = $derived(targets.filter(t => t.enabled).length);

  async function loadData() {
    try {
      isLoading = true;
      error = null;

      const [targetsData, statsData] = await Promise.all([
        getTargets(),
        getStats()
      ]);

      const skillsData = await validateAll();

      skills = skillsData;
      targets = targetsData;
      stats = await getStats();
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

      skills = await validateAll();
      const results = await syncAll();
      await loadData();

      // Calculate totals
      const totalCreated = results.reduce((sum, r) => sum + r.created.length, 0);
      const totalRemoved = results.reduce((sum, r) => sum + r.removed.length, 0);
      const totalErrors = results.reduce((sum, r) => sum + r.errors.length, 0);

      if (totalErrors > 0) {
        // Show persistent banner for errors
        lastSyncResults = results;
      } else {
        // Show auto-dismissing snackbar for success
        lastSyncResults = [];
        showSnackbar(`Synced: +${totalCreated} -${totalRemoved}`, 'success');
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isSyncing = false;
    }
  }

  async function handleRefresh() {
    try {
      error = null;
      await refreshSkills();
      skills = await validateAll();
      stats = await getStats();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleCreateSkill() {
    const name = newSkillName.trim();
    const description = newSkillDescription.trim();

    if (!name || !description) return;

    try {
      error = null;
      const newSkill = await createSkill(name, description);
      skills = [...skills, newSkill];
      newSkillName = '';
      newSkillDescription = '';
      showNewSkillForm = false;
      stats = await getStats();
      showSnackbar(`Created "${name}"`, 'success');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleDeleteSkill(skill: SkillInfo, event: MouseEvent) {
    event.stopPropagation();

    // Confirmation dialog
    const confirmed = await ask(`Delete "${skill.name}"?\n\nThis will permanently remove the skill and its symlinks from all targets.`, {
      title: 'Delete Skill',
      kind: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Cancel',
    });

    if (!confirmed) return;

    const folderName = skill.folder_name;
    const wasEditing = editingSkill?.folder_name === folderName;

    if (wasEditing) {
      editingSkill = null;
      editorContent = '';
      originalContent = '';
    }

    const previousSkills = skills;
    skills = skills.filter(s => s.folder_name !== folderName);

    try {
      error = null;
      await deleteSkill(folderName);
      stats = await getStats();
      showSnackbar(`Deleted "${skill.name}"`, 'success');
    } catch (e) {
      skills = previousSkills;
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleEditSkill(skill: SkillInfo) {
    if (hasUnsavedChanges && editingSkill) {
      const confirmed = await ask('You have unsaved changes. Discard and open another skill?', {
        title: 'Unsaved Changes',
        kind: 'warning',
      });
      if (!confirmed) return;
    }

    try {
      error = null;
      const content = await getSkillContent(skill.folder_name);
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

      const currentFolderName = editingSkill.folder_name;
      const savedSkill = await saveSkillContent(currentFolderName, editorContent);
      const validatedSkill = await validateSkill(savedSkill.folder_name);

      originalContent = editorContent;
      editingSkill = validatedSkill;

      if (savedSkill.folder_name !== currentFolderName) {
        skills = skills.map(s => s.folder_name === currentFolderName ? validatedSkill : s);
      } else {
        skills = skills.map(s => s.folder_name === savedSkill.folder_name ? validatedSkill : s);
      }

      stats = await getStats();
      showSnackbar('Saved', 'success');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      await loadData();
    } finally {
      isSaving = false;
    }
  }

  async function handleCloseEditor() {
    if (hasUnsavedChanges) {
      const confirmed = await ask('You have unsaved changes. Discard?', {
        title: 'Unsaved Changes',
        kind: 'warning',
      });
      if (!confirmed) return;
    }
    editingSkill = null;
    editorContent = '';
    originalContent = '';
  }

  function handleEditorChange(content: string) {
    editorContent = content;
  }

  async function handleFixSkill() {
    if (!editingSkill) return;

    try {
      isFixing = true;
      error = null;

      const fixedSkill = await fixSkill(editingSkill.folder_name);

      // Reload the content to show fixes
      const newContent = await getSkillContent(fixedSkill.folder_name);
      editorContent = newContent;
      originalContent = newContent;

      // Validate to update status
      const validatedSkill = await validateSkill(fixedSkill.folder_name);
      editingSkill = validatedSkill;

      // Update skills list
      skills = skills.map(s => s.folder_name === fixedSkill.folder_name ? validatedSkill : s);
      stats = await getStats();
      showSnackbar('Fixed frontmatter', 'success');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isFixing = false;
    }
  }

  // Check if editing skill has fixable errors
  let hasFixableErrors = $derived(
    editingSkill?.validation_errors.some(e => e.includes('can be auto-fixed')) ?? false
  );

  async function handleImport() {
    isImporting = true;
    error = null;
    lastImportResult = null;

    try {
      const result = await importAllSkills();
      await loadData();

      if (result.errors.length > 0) {
        // Show persistent banner for errors
        lastImportResult = result;
      } else if (result.imported.length > 0) {
        // Show auto-dismissing snackbar for success
        showSnackbar(`Imported ${result.imported.length} skill${result.imported.length === 1 ? '' : 's'}`, 'success');
      } else {
        showSnackbar('No new skills to import', 'info');
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isImporting = false;
    }
  }

  // Target management
  async function handleToggleTarget(targetId: string) {
    try {
      error = null;
      await toggleTarget(targetId);
      // Refresh targets to get updated state
      targets = await getTargets();
      stats = await getStats();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleShowAddTarget() {
    try {
      availableTargetTypes = await getAvailableTargetTypes();
      if (availableTargetTypes.length > 0) {
        selectedTargetType = availableTargetTypes[0][0];
      }
      customTargetPath = '';
      showAddTargetForm = true;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleAddTarget() {
    if (!selectedTargetType || !customTargetPath.trim()) return;

    try {
      error = null;
      await addCustomTarget(selectedTargetType, customTargetPath.trim());
      targets = await getTargets();
      stats = await getStats();
      showAddTargetForm = false;
      selectedTargetType = '';
      customTargetPath = '';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  // Event listener cleanup
  let unlistenFns: UnlistenFn[] = [];

  // Prevent macOS beep on non-input keystrokes + additional keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    const target = event.composedPath()[0];
    const isInput = target instanceof HTMLInputElement ||
                    target instanceof HTMLTextAreaElement ||
                    (target instanceof HTMLElement && target.closest('.cm-editor'));

    // Keyboard shortcuts (Cmd+S for save - native menu handles Cmd+N, Cmd+R, etc.)
    if (event.metaKey || event.ctrlKey) {
      switch (event.key) {
        case 's':
          // Cmd+S saves current skill (Cmd+Shift+S is sync all via menu)
          if (!event.shiftKey && editingSkill && hasUnsavedChanges) {
            event.preventDefault();
            handleSaveSkill();
          }
          return;
      }
    }

    if (event.key === 'Escape') {
      event.preventDefault();
      if (showNewSkillForm) {
        showNewSkillForm = false;
      } else if (editingSkill) {
        handleCloseEditor();
      }
      return;
    }

    // Allow default behavior in input fields
    if (isInput) return;

    // Prevent beep for non-input keystrokes
    event.preventDefault();
  }

  onMount(async () => {
    loadData();

    // Listen for tray events
    unlistenFns.push(await listen('tray-sync-all', () => {
      handleSync();
    }));

    // Listen for menu events
    unlistenFns.push(await listen('menu-new-skill', () => {
      showNewSkillForm = true;
    }));

    unlistenFns.push(await listen('menu-sync-all', () => {
      handleSync();
    }));

    unlistenFns.push(await listen('menu-refresh', () => {
      handleRefresh();
    }));

    document.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    unlistenFns.forEach(fn => fn());
    document.removeEventListener('keydown', handleKeydown);
  });
</script>

<div class="app-container" class:editor-open={editingSkill !== null}>
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <h1>Talent</h1>
    </div>

    <nav class="sidebar-nav">
      <div class="nav-section">
        <button
          class="nav-item"
          class:active={activeFilter === 'all'}
          onclick={() => activeFilter = 'all'}
        >
          <span class="nav-icon">◈</span>
          <span class="nav-label">All Skills</span>
          <span class="nav-count">{skills.length}</span>
        </button>
        <button
          class="nav-item"
          class:active={activeFilter === 'valid'}
          onclick={() => activeFilter = 'valid'}
        >
          <span class="nav-icon">●</span>
          <span class="nav-label">Valid</span>
          <span class="nav-count">{validCount}</span>
        </button>
        <button
          class="nav-item"
          class:active={activeFilter === 'invalid'}
          onclick={() => activeFilter = 'invalid'}
        >
          <span class="nav-icon">○</span>
          <span class="nav-label">Invalid</span>
          <span class="nav-count">{invalidCount}</span>
        </button>
      </div>

      <div class="nav-section">
        <div class="nav-section-header">
          <span class="nav-section-title">Targets</span>
          <button class="nav-section-action" onclick={handleShowAddTarget} title="Add target">
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
            </svg>
          </button>
        </div>
        {#if showAddTargetForm}
          <div class="add-target-form">
            <select bind:value={selectedTargetType}>
              {#each availableTargetTypes as [id, name]}
                <option value={id}>{name}</option>
              {/each}
            </select>
            <input
              type="text"
              placeholder="Skills path..."
              bind:value={customTargetPath}
            />
            <div class="form-actions">
              <button onclick={() => showAddTargetForm = false}>Cancel</button>
              <button class="primary" onclick={handleAddTarget} disabled={!selectedTargetType || !customTargetPath.trim()}>
                Add
              </button>
            </div>
          </div>
        {/if}
        {#each targets as target}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="nav-item target-item"
            class:disabled={!target.enabled}
            onclick={() => handleToggleTarget(target.id)}
            title={target.enabled ? 'Click to disable' : 'Click to enable'}
          >
            <span class="nav-icon">{target.enabled ? '◉' : '○'}</span>
            <span class="nav-label">{target.name}</span>
            {#if target.exists}
              <span class="target-ready">✓</span>
            {/if}
          </div>
        {/each}
        {#if targets.length === 0}
          <div class="nav-empty">No targets detected</div>
        {/if}
      </div>
    </nav>

    <div class="sidebar-footer">
      <button class="sidebar-action" onclick={handleSync} disabled={isSyncing}>
        {isSyncing ? 'Syncing...' : 'Sync All'}
      </button>
      <div class="sidebar-actions-row">
        <button class="sidebar-action-small" onclick={handleRefresh} disabled={isLoading} title="Refresh skills">
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99" />
          </svg>
        </button>
        <button class="sidebar-action-small" onclick={handleImport} disabled={isImporting} title="Import from targets">
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3" />
          </svg>
        </button>
        <button class="sidebar-action-small" onclick={() => showNewSkillForm = !showNewSkillForm} title="New skill">
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
        </button>
      </div>
    </div>
  </aside>

  <!-- Skill List -->
  <div class="list-panel">
    <div class="list-header">
      <span class="list-title">
        {activeFilter === 'all' ? 'All Skills' : activeFilter === 'valid' ? 'Valid Skills' : 'Invalid Skills'}
      </span>
      <span class="list-count">{filteredSkills().length}</span>
    </div>

    {#if error}
      <div class="error-banner">
        <span>{error}</span>
        <button onclick={() => error = null}>
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    {/if}

    {#if showNewSkillForm}
      <div class="new-skill-form">
        <div class="form-header">
          <div class="form-title">
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M9.813 15.904 9 18.75l-.813-2.846a4.5 4.5 0 0 0-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 0 0 3.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 0 0 3.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 0 0-3.09 3.09ZM18.259 8.715 18 9.75l-.259-1.035a3.375 3.375 0 0 0-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 0 0 2.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 0 0 2.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 0 0-2.456 2.456Z" />
            </svg>
            <span>New Skill</span>
          </div>
          <button class="form-close" onclick={() => showNewSkillForm = false} title="Cancel (Esc)">
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="form-field">
          <label for="skill-name">Name</label>
          <input
            id="skill-name"
            type="text"
            placeholder="my-awesome-skill"
            bind:value={newSkillName}
            autofocus
          />
          {#if newSkillName.trim()}
            <div class="field-hint">
              Will create: <code>~/.talent/skills/{newSkillName.trim().toLowerCase().replace(/\s+/g, '-')}/</code>
            </div>
          {/if}
        </div>

        <div class="form-field">
          <label for="skill-description">Description</label>
          <textarea
            id="skill-description"
            placeholder="What does this skill do?"
            bind:value={newSkillDescription}
            rows="2"
          ></textarea>
        </div>

        <div class="form-actions">
          <button onclick={() => showNewSkillForm = false}>Cancel</button>
          <button class="primary" onclick={handleCreateSkill} disabled={!newSkillName.trim() || !newSkillDescription.trim()}>
            <svg class="icon-sm" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
            </svg>
            Create Skill
          </button>
        </div>
      </div>
    {/if}

    {#if lastSyncResults.length > 0}
      {@const totalCreated = lastSyncResults.reduce((sum, r) => sum + r.created.length, 0)}
      {@const totalRemoved = lastSyncResults.reduce((sum, r) => sum + r.removed.length, 0)}
      {@const allErrors = lastSyncResults.flatMap(r => r.errors.map(e => ({ target: r.target_name, ...e })))}
      {@const totalErrors = allErrors.length}
      <div class="sync-banner" class:has-errors={totalErrors > 0}>
        <div class="sync-info">
          <span>
            {#if totalErrors > 0}⚠{:else}✓{/if}
            Synced: +{totalCreated} -{totalRemoved}
            {#if totalErrors > 0}({totalErrors} errors){/if}
          </span>
          {#if totalErrors > 0}
            <div class="sync-errors">
              {#each allErrors as err}
                <div class="sync-error">{err.target}: {err.message}</div>
              {/each}
            </div>
          {/if}
        </div>
        <button onclick={() => lastSyncResults = []}>
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    {/if}

    {#if lastImportResult}
      <div class="sync-banner" class:has-errors={lastImportResult.errors.length > 0}>
        <span>
          {#if lastImportResult.errors.length > 0}⚠{:else}✓{/if}
          Imported: {lastImportResult.imported.length} skills
        </span>
        <button onclick={() => lastImportResult = null}>
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    {/if}

    <div class="skill-list">
      {#if isLoading}
        <div class="loading">Loading...</div>
      {:else if filteredSkills().length === 0}
        <div class="empty-state">
          <p>No skills found</p>
          {#if activeFilter !== 'all'}
            <button onclick={() => activeFilter = 'all'}>Show all</button>
          {:else}
            <button onclick={() => showNewSkillForm = true}>Create one</button>
          {/if}
        </div>
      {:else}
        {#each filteredSkills() as skill}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="skill-item"
            class:selected={editingSkill?.folder_name === skill.folder_name}
            onclick={() => handleEditSkill(skill)}
          >
            <div class="skill-status">
              <span class="status-dot" class:valid={skill.validation_status === 'valid'} class:invalid={skill.validation_status === 'invalid'}></span>
            </div>
            <div class="skill-info">
              <div class="skill-name">{skill.name}</div>
              <div class="skill-description">{skill.description}</div>
            </div>
            <button class="skill-delete" onclick={(e) => handleDeleteSkill(skill, e)} title="Delete skill">
              <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Editor Panel -->
  {#if editingSkill}
    <div class="editor-panel">
      <div class="editor-header">
        <div class="editor-title">
          <h2>{editingSkill.name}</h2>
          {#if hasUnsavedChanges}
            <span class="unsaved-dot"></span>
          {/if}
        </div>
        <div class="editor-actions">
          <button onclick={handleCloseEditor}>Close</button>
          <button class="primary" onclick={handleSaveSkill} disabled={isSaving || !hasUnsavedChanges}>
            {isSaving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>
      {#if editingSkill.validation_errors.length > 0}
        <div class="validation-banner">
          <div class="validation-errors">
            {#each editingSkill.validation_errors as err}
              <div class="validation-error">{err}</div>
            {/each}
          </div>
          {#if hasFixableErrors}
            <button class="fix-button" onclick={handleFixSkill} disabled={isFixing || hasUnsavedChanges}>
              {isFixing ? 'Fixing...' : 'Auto-Fix'}
            </button>
          {/if}
        </div>
      {/if}
      <div class="editor-container">
        <SkillEditor content={editorContent} onchange={handleEditorChange} />
      </div>
    </div>
  {:else}
    <div class="editor-placeholder">
      <div class="placeholder-content">
        <div class="placeholder-icon">◇</div>
        <p>Select a skill to edit</p>
        <p class="placeholder-hint">or press <kbd>⌘N</kbd> to create new</p>
      </div>
    </div>
  {/if}
</div>

<!-- Snackbar container -->
{#if snackbars.length > 0}
  <div class="snackbar-container">
    {#each snackbars as snackbar (snackbar.id)}
      <div class="snackbar snackbar-{snackbar.type}">
        <span>{snackbar.message}</span>
        <button onclick={() => dismissSnackbar(snackbar.id)}>
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  /* ============================================
     DESIGN TOKENS
     ============================================ */
  :root {
    /* Spacing */
    --space-1: 4px;
    --space-2: 8px;
    --space-3: 12px;
    --space-4: 16px;
    --space-5: 20px;
    --space-6: 24px;
    --space-8: 32px;

    /* macOS traffic light area */
    --titlebar-height: 52px;

    /* Typography */
    --font-xs: 11px;
    --font-sm: 13px;
    --font-base: 14px;
    --font-lg: 16px;
    --font-xl: 18px;

    --font-weight-normal: 400;
    --font-weight-medium: 500;
    --font-weight-semibold: 600;

    /* Border Radius */
    --radius-sm: 4px;
    --radius-md: 6px;
    --radius-lg: 8px;

    /* Colors - Native macOS dark theme */
    --color-bg: #1c1c1e;
    --color-sidebar: #2c2c2e;
    --color-surface: #3a3a3c;
    --color-surface-hover: #48484a;
    --color-border: #3d3d3f;

    --color-text: #ffffff;
    --color-text-secondary: rgba(255, 255, 255, 0.85);
    --color-text-muted: rgba(255, 255, 255, 0.55);
    --color-text-dim: rgba(255, 255, 255, 0.35);

    /* Accent */
    --color-primary: #0a84ff;
    --color-primary-hover: #409cff;
    --color-primary-muted: rgba(10, 132, 255, 0.18);

    --color-success: #30d158;
    --color-warning: #ff9f0a;
    --color-error: #ff453a;

    /* Layout */
    --sidebar-width: 220px;
    --list-width: 300px;
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
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', sans-serif;
    font-size: var(--font-base);
    background-color: var(--color-bg);
    color: var(--color-text);
    overflow: hidden;
    -webkit-font-smoothing: antialiased;
    user-select: none;
    -webkit-user-select: none;
    cursor: default;
  }

  :global(input), :global(textarea), :global(.cm-editor) {
    user-select: text;
    -webkit-user-select: text;
    cursor: text;
  }

  /* ============================================
     ICONS (SF Symbols style)
     ============================================ */
  .icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  .icon-sm {
    width: 12px;
    height: 12px;
  }

  .icon-lg {
    width: 20px;
    height: 20px;
  }

  /* ============================================
     LAYOUT
     ============================================ */
  .app-container {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  /* ============================================
     SIDEBAR
     ============================================ */
  .sidebar {
    width: var(--sidebar-width);
    background: var(--color-sidebar);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar-header {
    height: var(--titlebar-height);
    padding: 0 var(--space-4);
    display: flex;
    align-items: flex-end;
    padding-bottom: var(--space-3);
    -webkit-app-region: drag;
    flex-shrink: 0;
  }

  .sidebar-header h1 {
    margin: 0;
    font-size: var(--font-xl);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
    letter-spacing: -0.02em;
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--space-2) var(--space-3);
  }

  .nav-section {
    margin-bottom: var(--space-5);
  }

  .nav-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-1) var(--space-2);
    margin-bottom: var(--space-1);
  }

  .nav-section-title {
    font-size: 11px;
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .nav-section-action {
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--color-text-dim);
    font-size: 14px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .nav-section-action:hover {
    background: var(--color-surface);
    color: var(--color-text-muted);
    transform: scale(1.1);
  }

  .nav-section-action:active {
    transform: scale(0.95);
  }

  .nav-empty {
    padding: var(--space-2);
    font-size: var(--font-xs);
    color: var(--color-text-dim);
    text-align: center;
  }

  .add-target-form {
    padding: var(--space-2);
    margin-bottom: var(--space-2);
    background: var(--color-surface);
    border-radius: var(--radius-md);
  }

  .add-target-form select,
  .add-target-form input {
    width: 100%;
    padding: 8px var(--space-2);
    margin-bottom: var(--space-2);
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text);
    font-size: var(--font-xs);
    box-sizing: border-box;
  }

  .add-target-form select:focus,
  .add-target-form input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .add-target-form input::placeholder {
    color: var(--color-text-dim);
  }

  .add-target-form .form-actions {
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
  }

  .add-target-form .form-actions button {
    padding: 6px var(--space-3);
    background: var(--color-surface-hover);
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: var(--font-xs);
    cursor: pointer;
  }

  .add-target-form .form-actions button.primary {
    background: var(--color-primary);
    color: white;
  }

  .add-target-form .form-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-2) var(--space-2);
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--font-sm);
    text-align: left;
    min-height: 28px;
    box-sizing: border-box;
    transition: background 0.15s ease, color 0.15s ease, transform 0.1s ease;
  }

  .nav-item:hover:not(.active) {
    background: var(--color-surface);
  }

  .nav-item:active {
    transform: scale(0.98);
  }

  .nav-item.active {
    background: var(--color-primary-muted);
    color: var(--color-primary);
  }

  .nav-item.disabled {
    opacity: 0.5;
  }

  .nav-item.target-item {
    cursor: pointer;
    padding: var(--space-1) var(--space-2);
    min-height: 24px;
    font-size: var(--font-xs);
    color: var(--color-text-muted);
  }

  .nav-item.target-item:hover:not(.active) {
    background: rgba(255, 255, 255, 0.05);
  }

  .nav-icon {
    flex-shrink: 0;
    width: 16px;
    text-align: center;
    font-size: 10px;
  }

  .nav-label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nav-count {
    flex-shrink: 0;
    font-size: 11px;
    color: var(--color-text-muted);
    background: var(--color-surface);
    padding: 2px 8px;
    border-radius: 10px;
    min-width: 20px;
    text-align: center;
    transition: transform 0.2s ease, background 0.15s ease;
  }

  .nav-item:hover .nav-count {
    background: var(--color-surface-hover);
  }

  .nav-item.active .nav-count {
    background: var(--color-primary);
    color: white;
  }

  .target-ready {
    flex-shrink: 0;
    color: var(--color-success);
    font-size: 12px;
    animation: checkmark-pop 0.3s ease;
  }

  @keyframes checkmark-pop {
    0% { transform: scale(0); opacity: 0; }
    50% { transform: scale(1.3); }
    100% { transform: scale(1); opacity: 1; }
  }

  .sidebar-footer {
    padding: var(--space-3);
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .sidebar-action {
    width: 100%;
    padding: 10px var(--space-4);
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.15s ease;
  }

  .sidebar-action:hover:not(:disabled) {
    background: var(--color-primary-hover);
    box-shadow: 0 2px 8px rgba(10, 132, 255, 0.3);
  }

  .sidebar-action:active:not(:disabled) {
    transform: scale(0.98);
  }

  .sidebar-action:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .sidebar-actions-row {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }

  .sidebar-action-small {
    flex: 1;
    padding: 8px;
    background: var(--color-surface);
    color: var(--color-text-secondary);
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s ease, transform 0.1s ease, color 0.15s ease;
  }

  .sidebar-action-small:hover:not(:disabled) {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .sidebar-action-small:active:not(:disabled) {
    transform: scale(0.92);
  }

  .sidebar-action-small:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ============================================
     LIST PANEL
     ============================================ */
  .list-panel {
    width: var(--list-width);
    background: var(--color-bg);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .list-header {
    height: var(--titlebar-height);
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    padding: 0 var(--space-4);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--color-border);
    -webkit-app-region: drag;
    flex-shrink: 0;
  }

  .list-title {
    font-size: var(--font-base);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
  }

  .list-count {
    font-size: var(--font-xs);
    color: var(--color-text-dim);
    background: var(--color-surface);
    padding: 2px 8px;
    border-radius: 10px;
  }

  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: rgba(255, 69, 58, 0.12);
    color: var(--color-error);
    font-size: var(--font-xs);
  }

  .error-banner span {
    flex: 1;
    min-width: 0;
  }

  .error-banner button {
    flex-shrink: 0;
    background: none;
    border: none;
    color: var(--color-error);
    cursor: pointer;
    padding: 0;
    font-size: 16px;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sync-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: rgba(48, 209, 88, 0.12);
    color: var(--color-success);
    font-size: var(--font-xs);
  }

  .sync-banner.has-errors {
    background: rgba(255, 159, 10, 0.12);
    color: var(--color-warning);
  }

  .sync-info {
    flex: 1;
    min-width: 0;
  }

  .sync-errors {
    margin-top: var(--space-2);
    font-size: var(--font-xs);
    opacity: 0.9;
  }

  .sync-error {
    padding: 2px 0;
    word-break: break-word;
  }

  .sync-banner button {
    flex-shrink: 0;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 0;
    font-size: 16px;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  /* ============================================
     NEW SKILL FORM
     ============================================ */
  .new-skill-form {
    margin: var(--space-3);
    padding: var(--space-4);
    background: linear-gradient(
      to bottom,
      color-mix(in srgb, var(--color-surface) 100%, transparent),
      color-mix(in srgb, var(--color-surface) 85%, var(--color-bg))
    );
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow:
      0 2px 8px rgba(0, 0, 0, 0.2),
      0 0 0 1px rgba(255, 255, 255, 0.03) inset;
    animation: form-slide-in 0.2s ease-out;
  }

  @keyframes form-slide-in {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .form-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-4);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--color-border);
  }

  .form-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
  }

  .form-title .icon {
    color: var(--color-primary);
  }

  .form-close {
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

  .form-close:hover {
    color: var(--color-text);
    background: var(--color-surface-hover);
  }

  .form-field {
    margin-bottom: var(--space-4);
  }

  .form-field label {
    display: block;
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-muted);
    margin-bottom: var(--space-2);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .new-skill-form input,
  .new-skill-form textarea {
    width: 100%;
    padding: var(--space-3);
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text);
    font-size: var(--font-sm);
    font-family: inherit;
    box-sizing: border-box;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  .new-skill-form textarea {
    resize: vertical;
    min-height: 60px;
    line-height: 1.5;
  }

  .new-skill-form input:focus,
  .new-skill-form textarea:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-muted);
  }

  .new-skill-form input::placeholder,
  .new-skill-form textarea::placeholder {
    color: var(--color-text-dim);
  }

  .field-hint {
    margin-top: var(--space-2);
    font-size: var(--font-xs);
    color: var(--color-text-muted);
  }

  .field-hint code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    font-size: 10px;
    padding: 2px 6px;
    background: var(--color-bg);
    border-radius: var(--radius-sm);
    color: var(--color-primary);
  }

  .form-actions {
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
    padding-top: var(--space-3);
    border-top: 1px solid var(--color-border);
  }

  .form-actions button {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    background: var(--color-surface-hover);
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .form-actions button:hover:not(:disabled) {
    background: var(--color-bg);
  }

  .form-actions button.primary {
    background: var(--color-primary);
    color: white;
  }

  .form-actions button.primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .form-actions button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .form-actions .icon-sm {
    width: 14px;
    height: 14px;
  }

  .skill-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .loading, .empty-state {
    padding: var(--space-8) var(--space-4);
    text-align: center;
    color: var(--color-text-muted);
    font-size: var(--font-sm);
    animation: fade-in 0.3s ease;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
  }

  .loading::before {
    content: '';
    width: 20px;
    height: 20px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state p {
    margin: 0;
  }

  .empty-state button {
    margin-top: var(--space-4);
    padding: 10px var(--space-5);
    background: var(--color-surface);
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: var(--font-sm);
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .empty-state button:hover {
    background: var(--color-surface-hover);
  }

  .empty-state button:active {
    transform: scale(0.97);
  }

  .skill-item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    min-height: 56px;
    box-sizing: border-box;
    transition: background 0.15s ease;
  }

  .skill-item:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .skill-item.selected {
    background: var(--color-primary-muted);
  }

  .skill-item:active {
    background: rgba(255, 255, 255, 0.06);
  }

  .skill-item.selected:active {
    background: var(--color-primary-muted);
  }

  .skill-status {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
  }

  .status-dot {
    display: block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-text-dim);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }

  .skill-item:hover .status-dot {
    transform: scale(1.2);
  }

  .status-dot.valid {
    background: var(--color-success);
    box-shadow: 0 0 0 0 rgba(48, 209, 88, 0);
  }

  .skill-item:hover .status-dot.valid {
    box-shadow: 0 0 6px 2px rgba(48, 209, 88, 0.4);
  }

  .status-dot.invalid {
    background: var(--color-error);
    animation: pulse-error 2s ease-in-out infinite;
  }

  @keyframes pulse-error {
    0%, 100% {
      box-shadow: 0 0 0 0 rgba(255, 69, 58, 0);
    }
    50% {
      box-shadow: 0 0 6px 2px rgba(255, 69, 58, 0.4);
    }
  }

  .skill-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .skill-name {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .skill-description {
    font-size: var(--font-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 1.3;
  }

  .skill-delete {
    flex-shrink: 0;
    opacity: 0;
    width: 24px;
    height: 24px;
    padding: 0;
    background: none;
    border: none;
    color: var(--color-text-dim);
    font-size: 16px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: opacity 0.15s ease, color 0.15s ease, background 0.15s ease, transform 0.1s ease;
  }

  .skill-item:hover .skill-delete {
    opacity: 1;
  }

  .skill-delete:hover {
    color: var(--color-error);
    background: rgba(255, 69, 58, 0.15);
    transform: scale(1.1);
  }

  .skill-delete:active {
    transform: scale(0.9);
  }

  /* ============================================
     EDITOR PANEL
     ============================================ */
  .editor-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 400px;
    background: var(--color-sidebar);
    animation: editor-slide-in 0.2s ease-out;
  }

  @keyframes editor-slide-in {
    from {
      opacity: 0;
      transform: translateX(10px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .editor-header {
    height: var(--titlebar-height);
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    padding: 0 var(--space-4);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg);
    -webkit-app-region: drag;
    flex-shrink: 0;
  }

  .editor-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .editor-title h2 {
    margin: 0;
    font-size: var(--font-sm);
    font-weight: var(--font-weight-semibold);
  }

  .unsaved-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-warning);
    animation: unsaved-pulse 1.5s ease-in-out infinite;
  }

  @keyframes unsaved-pulse {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.6;
      transform: scale(0.85);
    }
  }

  .editor-actions {
    display: flex;
    gap: var(--space-2);
    -webkit-app-region: no-drag;
  }

  .editor-actions button {
    padding: var(--space-1) var(--space-3);
    background: var(--color-surface);
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: var(--font-xs);
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.15s ease;
  }

  .editor-actions button.primary {
    background: var(--color-primary);
    color: white;
  }

  .editor-actions button:hover:not(:disabled) {
    background: var(--color-surface-hover);
  }

  .editor-actions button.primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
    box-shadow: 0 2px 8px rgba(10, 132, 255, 0.3);
  }

  .editor-actions button:active:not(:disabled) {
    transform: scale(0.96);
  }

  .editor-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .validation-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4);
    background: rgba(255, 69, 58, 0.15);
    border-bottom: 1px solid var(--color-border);
  }

  .validation-errors {
    flex: 1;
  }

  .validation-error {
    font-size: var(--font-xs);
    color: var(--color-error);
    padding: var(--space-1) 0;
  }

  .fix-button {
    flex-shrink: 0;
    padding: var(--space-1) var(--space-3);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
    background: var(--color-warning);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .fix-button:hover:not(:disabled) {
    background: #ffa340;
  }

  .fix-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
  }

  /* ============================================
     EDITOR PLACEHOLDER
     ============================================ */
  .editor-placeholder {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg);
  }

  .placeholder-content {
    text-align: center;
    color: var(--color-text-dim);
  }

  .placeholder-icon {
    font-size: 48px;
    margin-bottom: var(--space-4);
    opacity: 0.3;
  }

  .placeholder-content p {
    margin: 0;
    font-size: var(--font-sm);
  }

  .placeholder-hint {
    margin-top: var(--space-2) !important;
    font-size: var(--font-xs) !important;
    color: var(--color-text-dim);
  }

  .placeholder-hint kbd {
    display: inline-block;
    padding: 2px 6px;
    background: var(--color-surface);
    border-radius: var(--radius-sm);
    font-family: inherit;
    font-size: var(--font-xs);
  }

  /* ============================================
     SNACKBAR / TOAST NOTIFICATIONS
     ============================================ */
  .snackbar-container {
    position: fixed;
    bottom: var(--space-6);
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    z-index: 1000;
    pointer-events: none;
  }

  .snackbar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3), 0 0 0 1px rgba(255, 255, 255, 0.05);
    font-size: var(--font-sm);
    color: var(--color-text);
    pointer-events: auto;
    animation: snackbar-slide-in 0.2s ease-out;
  }

  @keyframes snackbar-slide-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .snackbar span {
    flex: 1;
  }

  .snackbar button {
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    padding: 0;
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .snackbar button:hover {
    color: var(--color-text);
    background: var(--color-surface-hover);
  }

  .snackbar-success {
    border-left: 3px solid var(--color-success);
  }

  .snackbar-info {
    border-left: 3px solid var(--color-primary);
  }

  .snackbar-warning {
    border-left: 3px solid var(--color-warning);
  }
</style>
