<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { ask, open as openDialog } from '@tauri-apps/plugin-dialog';
  import ConfirmDeleteModal from './lib/ConfirmDeleteModal.svelte';
  import { getSkills, getTargets, syncAll, validateAll, refreshSkills, createSkill, deleteSkill, renameSkill, getStats, getSkillContent, saveSkillContent, validateSkill, importAllSkills, toggleTarget, getAvailableTargetTypes, addCustomTarget, fixSkill, setSaveMenuEnabled, scanFolderForSkills, importFromFolder } from './lib/api';
  import type { SkillInfo, TargetInfo, SyncResult, StatsInfo, ImportResultInfo, ScannedSkillInfo, FolderImportSelectionInfo } from './lib/types';
  import SkillEditor from './lib/SkillEditor.svelte';
  import ImportFromFolderModal from './lib/ImportFromFolderModal.svelte';
  import { Plus, RefreshCw, RotateCcw, Download, X, Sparkles, Trash2, FolderOpen } from 'lucide-svelte';

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

  // Folder import state
  let isDragging = $state(false);
  let isScanning = $state(false);
  let showFolderImportModal = $state(false);
  let scannedSkills = $state<ScannedSkillInfo[]>([]);
  let isFolderImporting = $state(false);

  // Sidebar filter
  let activeFilter = $state<'all' | 'valid' | 'invalid'>('all');

  // Add target state
  let showAddTargetForm = $state(false);
  let availableTargetTypes = $state<[string, string][]>([]);
  let selectedTargetType = $state('');
  let customTargetPath = $state('');

  // Delete confirmation state
  let showDeleteModal = $state(false);
  let skillToDelete = $state<SkillInfo | null>(null);

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
  let hasNewSkillFormInput = $derived(newSkillName.trim() !== '' || newSkillDescription.trim() !== '');
  let hasAnyUnsavedWork = $derived(hasUnsavedChanges || (showNewSkillForm && hasNewSkillFormInput));

  // Update Save menu enabled state when editing state changes
  $effect(() => {
    setSaveMenuEnabled(editingSkill !== null && hasUnsavedChanges);
  });

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

  async function handleCloseNewSkillForm() {
    if (hasNewSkillFormInput) {
      const confirmed = await ask('You have unsaved changes. Discard new skill?', {
        title: 'Unsaved Changes',
        kind: 'warning',
      });
      if (!confirmed) return;
    }
    newSkillName = '';
    newSkillDescription = '';
    showNewSkillForm = false;
  }

  function handleDeleteSkill(skill: SkillInfo, event: MouseEvent) {
    event.stopPropagation();
    skillToDelete = skill;
    showDeleteModal = true;
  }

  async function confirmDeleteSkill() {
    if (!skillToDelete) return;

    const skill = skillToDelete;
    const folderName = skill.folder_name;
    const wasEditing = editingSkill?.folder_name === folderName;

    // Close modal first
    showDeleteModal = false;
    skillToDelete = null;

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

  function cancelDeleteSkill() {
    showDeleteModal = false;
    skillToDelete = null;
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

  // === Folder Import Handlers ===

  async function handleFolderPickerImport() {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: 'Select folder containing skills',
      });

      if (selected && typeof selected === 'string') {
        await scanAndShowModal(selected);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function scanAndShowModal(folderPath: string) {
    isScanning = true;
    error = null;

    try {
      const results = await scanFolderForSkills(folderPath);
      scannedSkills = results;

      if (results.length === 0) {
        showSnackbar('No skills found in folder', 'warning');
      } else {
        showFolderImportModal = true;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isScanning = false;
    }
  }

  async function handleFolderImport(selections: FolderImportSelectionInfo[]) {
    isFolderImporting = true;
    error = null;

    try {
      const result = await importFromFolder(selections);
      showFolderImportModal = false;
      await loadData();

      if (result.errors.length > 0) {
        lastImportResult = result;
      } else if (result.imported.length > 0) {
        showSnackbar(`Imported ${result.imported.length} skill${result.imported.length === 1 ? '' : 's'}`, 'success');
      } else {
        showSnackbar('No skills imported', 'info');
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isFolderImporting = false;
    }
  }

  function closeFolderImportModal() {
    showFolderImportModal = false;
    scannedSkills = [];
  }

  // === Drag and Drop Handlers ===

  let dragCounter = 0;

  function handleDragEnter(event: DragEvent) {
    event.preventDefault();
    dragCounter++;
    if (event.dataTransfer?.types.includes('Files')) {
      isDragging = true;
    }
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    dragCounter--;
    if (dragCounter === 0) {
      isDragging = false;
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
    dragCounter = 0;

    const items = event.dataTransfer?.items;
    if (!items || items.length === 0) return;

    // Get the first dropped item
    const item = items[0];
    if (item.kind !== 'file') return;

    const entry = item.webkitGetAsEntry?.();
    if (!entry || !entry.isDirectory) {
      showSnackbar('Please drop a folder, not a file', 'warning');
      return;
    }

    // For Tauri, we need to get the path from the file
    // The drag event in Tauri gives us the paths differently
    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      // In Tauri, the path is available via the webkitRelativePath or we need to use the Tauri API
      // For now, use the folder picker as fallback
      const filePath = (files[0] as any).path;
      if (filePath) {
        await scanAndShowModal(filePath);
      } else {
        // Fallback: open folder picker
        handleFolderPickerImport();
      }
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

    // Let ALL Cmd/Ctrl+key combos pass through to native menu handlers
    // (Cmd+S, Cmd+W, Cmd+N, Cmd+R, Cmd+Q, Cmd+H, Cmd+M, Cmd+Z, Cmd+C, Cmd+V, etc.)
    if (event.metaKey || event.ctrlKey) {
      return;
    }

    if (event.key === 'Escape') {
      event.preventDefault();
      if (showNewSkillForm) {
        handleCloseNewSkillForm();
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

    unlistenFns.push(await listen('menu-save', () => {
      if (editingSkill && hasUnsavedChanges) {
        handleSaveSkill();
      }
    }));

    // Handle window close with unsaved changes confirmation
    const currentWindow = getCurrentWindow();
    unlistenFns.push(await currentWindow.onCloseRequested(async (event) => {
      if (hasAnyUnsavedWork) {
        event.preventDefault();
        const confirmed = await ask('You have unsaved changes. Are you sure you want to quit?', {
          title: 'Unsaved Changes',
          kind: 'warning',
          okLabel: 'Quit',
          cancelLabel: 'Cancel',
        });
        if (confirmed) {
          await currentWindow.destroy();
        }
      }
    }));

    document.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    unlistenFns.forEach(fn => fn());
    document.removeEventListener('keydown', handleKeydown);
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="app-container"
  class:editor-open={editingSkill !== null}
  ondragenter={handleDragEnter}
  ondragleave={handleDragLeave}
  ondragover={handleDragOver}
  ondrop={handleDrop}
  role="application"
>
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
            <Plus class="icon" size={16} strokeWidth={1.5} />
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
              autocorrect="off"
              autocapitalize="off"
            />
            <div class="add-target-actions">
              <button class="add-target-btn" onclick={() => showAddTargetForm = false}>Cancel</button>
              <button class="add-target-btn primary" onclick={handleAddTarget} disabled={!selectedTargetType || !customTargetPath.trim()}>
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
      <!-- Primary Action -->
      <button class="sidebar-action-primary" onclick={handleSync} disabled={isSyncing}>
        <RefreshCw class="icon" size={14} strokeWidth={2} />
        {isSyncing ? 'Syncing...' : 'Sync All'}
      </button>

      <!-- Action Toolbar -->
      <div class="sidebar-toolbar">
        <!-- Import Group -->
        <div class="toolbar-group">
          <button class="toolbar-button" onclick={handleRefresh} disabled={isLoading} title="Refresh skill list">
            <RotateCcw class="icon" size={14} strokeWidth={1.5} />
          </button>
          <button class="toolbar-button" onclick={handleImport} disabled={isImporting} title="Import from targets">
            <Download class="icon" size={14} strokeWidth={1.5} />
          </button>
          <button class="toolbar-button" onclick={handleFolderPickerImport} disabled={isScanning} title="Import from folder">
            <FolderOpen class="icon" size={14} strokeWidth={1.5} />
          </button>
        </div>

        <!-- Divider -->
        <div class="toolbar-divider"></div>

        <!-- Create Action -->
        <button class="toolbar-button toolbar-button-create" onclick={() => showNewSkillForm = !showNewSkillForm} title="New skill (⌘N)">
          <Plus class="icon" size={14} strokeWidth={2} />
          <span>New</span>
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
        <button onclick={() => error = null} aria-label="Dismiss error">
          <X class="icon" size={16} strokeWidth={1.5} />
        </button>
      </div>
    {/if}

    {#if showNewSkillForm}
      <div class="new-skill-form">
        <div class="form-header">
          <div class="form-title">
            <Sparkles class="icon" size={16} strokeWidth={1.5} />
            <span>New Skill</span>
          </div>
          <button class="form-close" onclick={handleCloseNewSkillForm} title="Cancel (Esc)">
            <X class="icon" size={16} strokeWidth={1.5} />
          </button>
        </div>

        <div class="form-field">
          <label for="skill-name">Name</label>
          <!-- svelte-ignore a11y_autofocus -->
          <input
            id="skill-name"
            type="text"
            placeholder="my-awesome-skill"
            bind:value={newSkillName}
            autofocus
            autocapitalize="off"
            spellcheck="false"
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
            autocapitalize="off"
            spellcheck="false"
          ></textarea>
        </div>

        <div class="form-actions">
          <button class="form-btn" onclick={handleCloseNewSkillForm}>Cancel</button>
          <button class="form-btn primary" onclick={handleCreateSkill} disabled={!newSkillName.trim() || !newSkillDescription.trim()}>
            <Plus class="icon-sm" size={14} strokeWidth={2} />
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
        <button onclick={() => lastSyncResults = []} aria-label="Dismiss sync results">
          <X class="icon" size={16} strokeWidth={1.5} />
        </button>
      </div>
    {/if}

    {#if lastImportResult}
      <div class="sync-banner" class:has-errors={lastImportResult.errors.length > 0}>
        <span>
          {#if lastImportResult.errors.length > 0}⚠{:else}✓{/if}
          Imported: {lastImportResult.imported.length} skills
        </span>
        <button onclick={() => lastImportResult = null} aria-label="Dismiss import results">
          <X class="icon" size={16} strokeWidth={1.5} />
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
              <Trash2 class="icon" size={14} strokeWidth={1.5} />
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
          <button class="editor-btn" onclick={handleCloseEditor}>
            <X class="icon" size={14} strokeWidth={1.5} />
            <span>Close</span>
          </button>
          <button class="editor-btn primary" onclick={handleSaveSkill} disabled={isSaving || !hasUnsavedChanges}>
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
        <button class="placeholder-action" onclick={() => showNewSkillForm = true}>
          <Plus class="icon" size={14} strokeWidth={2} />
          <span>New Skill</span>
        </button>
      </div>
    </div>
  {/if}
</div>

<!-- Drop Zone Overlay -->
{#if isDragging}
  <div class="drop-zone-overlay">
    <div class="drop-zone-content">
      <FolderOpen class="drop-zone-icon" size={48} strokeWidth={1.5} />
      <p>Drop folder to import skills</p>
    </div>
  </div>
{/if}

<!-- Folder Import Modal -->
{#if showFolderImportModal}
  <ImportFromFolderModal
    skills={scannedSkills}
    isImporting={isFolderImporting}
    onimport={handleFolderImport}
    onclose={closeFolderImportModal}
  />
{/if}

<!-- Delete Confirmation Modal -->
{#if showDeleteModal && skillToDelete}
  <ConfirmDeleteModal
    skillName={skillToDelete.name}
    onconfirm={confirmDeleteSkill}
    oncancel={cancelDeleteSkill}
  />
{/if}

<!-- Snackbar container -->
{#if snackbars.length > 0}
  <div class="snackbar-container">
    {#each snackbars as snackbar (snackbar.id)}
      <div class="snackbar snackbar-{snackbar.type}">
        <span>{snackbar.message}</span>
        <button onclick={() => dismissSnackbar(snackbar.id)} aria-label="Dismiss notification">
          <X class="icon" size={16} strokeWidth={1.5} />
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
    /* Min-width = sidebar (220) + list (300) + editor (720) */
    min-width: 1240px;
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

  .add-target-actions {
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
  }

  .add-target-btn {
    height: 26px;
    padding: 0 var(--space-3);
    background: var(--color-surface-hover);
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .add-target-btn:hover:not(:disabled) {
    background: var(--color-bg);
  }

  .add-target-btn:active:not(:disabled) {
    transform: scale(0.96);
  }

  .add-target-btn.primary {
    background: var(--color-primary);
    color: white;
  }

  .add-target-btn.primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .add-target-btn:disabled {
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
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  /* Primary Sync Button */
  .sidebar-action-primary {
    width: 100%;
    padding: 10px var(--space-4);
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.15s ease;
  }

  .sidebar-action-primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
    box-shadow: 0 2px 8px rgba(10, 132, 255, 0.3);
  }

  .sidebar-action-primary:active:not(:disabled) {
    transform: scale(0.98);
  }

  .sidebar-action-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Toolbar Container */
  .sidebar-toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1);
    background: var(--color-surface);
    border-radius: var(--radius-md);
  }

  .toolbar-group {
    display: flex;
    gap: 2px;
  }

  .toolbar-divider {
    width: 1px;
    height: 20px;
    background: var(--color-border);
    margin: 0 var(--space-1);
  }

  .toolbar-button {
    height: 32px;
    min-width: 32px;
    padding: 0 var(--space-2);
    background: transparent;
    color: var(--color-text-muted);
    border: none;
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
    transition: background 0.15s ease, color 0.15s ease, transform 0.1s ease;
  }

  .toolbar-button:hover:not(:disabled) {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .toolbar-button:active:not(:disabled) {
    transform: scale(0.95);
  }

  .toolbar-button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* Create button with label */
  .toolbar-button-create {
    flex: 1;
    justify-content: center;
    color: var(--color-text-secondary);
  }

  .toolbar-button-create:hover:not(:disabled) {
    background: var(--color-primary-muted);
    color: var(--color-primary);
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

  .form-title :global(.icon) {
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

  .form-btn {
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: 0 var(--space-4);
    background: var(--color-surface-hover);
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.15s ease;
  }

  .form-btn:hover:not(:disabled) {
    background: var(--color-bg);
  }

  .form-btn:active:not(:disabled) {
    transform: scale(0.97);
  }

  .form-btn.primary {
    background: var(--color-primary);
    color: white;
  }

  .form-btn.primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
    box-shadow: 0 2px 8px rgba(10, 132, 255, 0.3);
  }

  .form-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .form-btn :global(.icon-sm) {
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
    /* Min-width ensures ~80 char line length: 80ch × ~8px + 50px gutter + 30px padding */
    min-width: 720px;
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
    align-items: center;
    gap: var(--space-2);
    -webkit-app-region: no-drag;
  }

  .editor-btn {
    height: 28px;
    padding: 0 var(--space-3);
    background: var(--color-surface);
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: var(--space-1);
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.15s ease;
  }

  .editor-btn.primary {
    background: var(--color-primary);
    color: white;
    padding: 0 var(--space-4);
  }

  .editor-btn:hover:not(:disabled) {
    background: var(--color-surface-hover);
  }

  .editor-btn.primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
    box-shadow: 0 2px 8px rgba(10, 132, 255, 0.3);
  }

  .editor-btn:active:not(:disabled) {
    transform: scale(0.96);
  }

  .editor-btn:disabled {
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

  .placeholder-action {
    margin-top: var(--space-5);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: 10px var(--space-5);
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.15s ease;
  }

  .placeholder-action:hover {
    background: var(--color-primary-hover);
    box-shadow: 0 2px 8px rgba(10, 132, 255, 0.3);
  }

  .placeholder-action:active {
    transform: scale(0.97);
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

  /* ============================================
     DROP ZONE OVERLAY
     ============================================ */
  .drop-zone-overlay {
    position: fixed;
    inset: 0;
    background: rgba(10, 132, 255, 0.15);
    border: 3px dashed var(--color-primary);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: drop-zone-fade-in 0.15s ease-out;
    pointer-events: none;
  }

  @keyframes drop-zone-fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .drop-zone-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-8);
    background: var(--color-sidebar);
    border-radius: var(--radius-lg);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    animation: drop-zone-bounce 0.3s ease-out;
  }

  @keyframes drop-zone-bounce {
    0% {
      transform: scale(0.9);
      opacity: 0;
    }
    50% {
      transform: scale(1.02);
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }

  .drop-zone-content :global(.drop-zone-icon) {
    color: var(--color-primary);
  }

  .drop-zone-content p {
    margin: 0;
    font-size: var(--font-lg);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
  }
</style>
