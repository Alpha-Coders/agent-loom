<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { ask, open as openDialog } from '@tauri-apps/plugin-dialog';
  import { getSkills, getTargets, syncAll, validateAll, refreshSkills, createSkill, deleteSkill, renameSkill, getStats, getSkillContent, saveSkillContent, validateSkill, importAllSkills, toggleTarget, getAvailableTargetTypes, addCustomTarget, fixSkill, scanFolderForSkills, importFromFolder, revealInFinder } from './lib/api';
  import type { SkillInfo, TargetInfo, SyncResult, StatsInfo, ImportResultInfo, ScannedSkillInfo, FolderImportSelectionInfo } from './lib/types';
  import SkillEditor from './lib/SkillEditor.svelte';
  import ImportFromFolderModal from './lib/ImportFromFolderModal.svelte';
  import { Plus, RefreshCw, RotateCcw, Download, X, Sparkles, Trash2, FolderOpen, FilePenLine } from 'lucide-svelte';

  // State using Svelte 5 runes
  let skills = $state<SkillInfo[]>([]);
  let targets = $state<TargetInfo[]>([]);
  let stats = $state<StatsInfo | null>(null);
  let lastSyncResults = $state<SyncResult[]>([]);

  let isLoading = $state(true);
  let isSyncing = $state(false);
  let isRefreshing = $state(false);
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
  let hasNewSkillFormInput = $derived(newSkillName.trim() !== '' || newSkillDescription.trim() !== '');
  let hasAnyUnsavedWork = $derived(hasUnsavedChanges || (showNewSkillForm && hasNewSkillFormInput));

  // Menu event handler - always enabled, checks state when called
  function handleMenuSave() {
    if (editingSkill && hasUnsavedChanges) {
      handleSaveSkill();
    }
  }

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
    const minDuration = 800; // Minimum time for one full icon rotation
    const startTime = Date.now();

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
      // Ensure minimum duration for smooth animation
      const elapsed = Date.now() - startTime;
      if (elapsed < minDuration) {
        await new Promise(resolve => setTimeout(resolve, minDuration - elapsed));
      }
      isSyncing = false;
    }
  }

  async function handleRefresh() {
    const minDuration = 800;
    const startTime = Date.now();

    try {
      isRefreshing = true;
      error = null;
      await refreshSkills();
      skills = await validateAll();
      stats = await getStats();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      const elapsed = Date.now() - startTime;
      if (elapsed < minDuration) {
        await new Promise(resolve => setTimeout(resolve, minDuration - elapsed));
      }
      isRefreshing = false;
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

      // Load the new skill in the editor
      await handleEditSkill(newSkill);
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

  async function handleDeleteSkill(skill: SkillInfo, event: MouseEvent) {
    event.stopPropagation();

    const confirmed = await ask(
      `This will permanently remove the skill and its symlinks from all targets.`,
      {
        title: `Delete "${skill.name}"?`,
        kind: 'warning',
        okLabel: 'Delete',
        cancelLabel: 'Cancel',
      }
    );

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

  // Finder integration
  async function handleRevealSkill(skill: SkillInfo, event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    try {
      await revealInFinder(skill.path);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleRevealTarget(target: TargetInfo, event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    try {
      await revealInFinder(target.skills_path);
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

    unlistenFns.push(await listen('menu-save', handleMenuSave));

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
  ondragenter={handleDragEnter}
  ondragleave={handleDragLeave}
  ondragover={handleDragOver}
  ondrop={handleDrop}
  role="application"
>
  <!-- Pane 1: Targets -->
  <aside class="targets-pane">
    <div class="pane-header">
      <span class="pane-title">Targets</span>
      <button class="pane-action" onclick={handleShowAddTarget} title="Add target">
        <Plus class="icon" size={16} strokeWidth={1.5} />
      </button>
    </div>

    <div class="targets-toolbar">
      <button class="sync-button" onclick={handleSync} disabled={isSyncing}>
        <span class="sync-icon" class:spinning={isSyncing}>
          <RefreshCw class="icon" size={14} strokeWidth={2} />
        </span>
        {isSyncing ? 'Syncing...' : 'Sync'}
      </button>
    </div>

    <div class="targets-content">
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

      <div class="targets-list">
        {#each targets as target}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="target-item"
            class:disabled={!target.enabled}
            onclick={() => handleToggleTarget(target.id)}
            oncontextmenu={(e) => handleRevealTarget(target, e)}
            title={target.enabled ? 'Click to disable • Right-click to reveal' : 'Click to enable • Right-click to reveal'}
          >
            <span class="target-icon">{target.enabled ? '◉' : '○'}</span>
            <span class="target-name">{target.name}</span>
            {#if target.enabled && target.exists}
              <span class="target-ready">✓</span>
            {/if}
          </div>
        {/each}
        {#if targets.length === 0}
          <div class="targets-empty">No targets detected</div>
        {/if}
      </div>
    </div>
  </aside>

  <!-- Pane 2: Skills -->
  <div class="skills-pane">
    <div class="pane-header">
      <span class="pane-title">{skills.length} Skills</span>
      <div class="pane-actions">
        <button class="pane-action" onclick={handleRefresh} disabled={isRefreshing} title="Refresh">
          <span class="refresh-icon" class:spinning={isRefreshing}>
            <RotateCcw class="icon" size={16} strokeWidth={1.5} />
          </span>
        </button>
        <button class="pane-action" onclick={handleImport} disabled={isImporting} title="Import from targets">
          <Download class="icon" size={16} strokeWidth={1.5} />
        </button>
        <button class="pane-action" onclick={handleFolderPickerImport} disabled={isScanning} title="Import from folder">
          <FolderOpen class="icon" size={16} strokeWidth={1.5} />
        </button>
      </div>
    </div>

    <div class="skills-content">
      {#if error}
        <div class="banner banner-error">
          <span>{error}</span>
          <button onclick={() => error = null} aria-label="Dismiss error">
            <X class="icon" size={16} strokeWidth={1.5} />
          </button>
        </div>
      {/if}

      {#if lastSyncResults.length > 0}
        {@const totalCreated = lastSyncResults.reduce((sum, r) => sum + r.created.length, 0)}
        {@const totalRemoved = lastSyncResults.reduce((sum, r) => sum + r.removed.length, 0)}
        {@const allErrors = lastSyncResults.flatMap(r => r.errors.map(e => ({ target: r.target_name, ...e })))}
        {@const totalErrors = allErrors.length}
        <div class="banner" class:banner-warning={totalErrors > 0} class:banner-success={totalErrors === 0}>
          <div class="banner-content">
            <span>
              {#if totalErrors > 0}⚠{:else}✓{/if}
              Synced: +{totalCreated} -{totalRemoved}
              {#if totalErrors > 0}({totalErrors} errors){/if}
            </span>
            {#if totalErrors > 0}
              <div class="banner-details">
                {#each allErrors as err}
                  <div>{err.target}: {err.message}</div>
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
        <div class="banner" class:banner-warning={lastImportResult.errors.length > 0} class:banner-success={lastImportResult.errors.length === 0}>
          <span>
            {#if lastImportResult.errors.length > 0}⚠{:else}✓{/if}
            Imported: {lastImportResult.imported.length} skills
          </span>
          <button onclick={() => lastImportResult = null} aria-label="Dismiss import results">
            <X class="icon" size={16} strokeWidth={1.5} />
          </button>
        </div>
      {/if}

      <div class="skills-list">
        {#if isLoading}
          <div class="loading">Loading...</div>
        {:else if skills.length === 0}
          <div class="empty-state">
            <div class="empty-icon">
              <Sparkles class="empty-icon-sparkle" size={32} strokeWidth={1.5} />
            </div>
            <p class="empty-title">No skills yet</p>
            <p class="empty-subtitle">Create your first skill to get started</p>
          </div>
        {:else}
          {#each skills as skill}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="skill-item"
              class:selected={editingSkill?.folder_name === skill.folder_name}
              onclick={() => handleEditSkill(skill)}
              oncontextmenu={(e) => handleRevealSkill(skill, e)}
              title="Click to edit • Right-click to reveal in Finder"
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
  </div>

  <!-- Pane 3: Editor -->
  <div class="editor-pane">
    {#if editingSkill}
      <div class="pane-header editor-header">
        <div class="pane-actions">
          <button class="pane-action" onclick={handleCloseEditor} title="Close">
            <X class="icon" size={16} strokeWidth={1.5} />
          </button>
        </div>
        <div class="editor-title">
          <h2>{editingSkill.name}</h2>
          {#if hasUnsavedChanges}
            <span class="unsaved-dot"></span>
          {/if}
        </div>
        <div class="pane-actions">
          <button class="pane-action primary" onclick={handleSaveSkill} disabled={isSaving || !hasUnsavedChanges} title="Save">
            {isSaving ? '...' : 'Save'}
          </button>
        </div>
      </div>
      {#if editingSkill.validation_errors.length > 0}
        <div class="banner banner-error">
          <div class="banner-content">
            {#each editingSkill.validation_errors as err}
              <div>{err}</div>
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
    {:else if showNewSkillForm}
      <div class="pane-header editor-header">
        <div class="pane-actions">
          <button class="pane-action" onclick={handleCloseNewSkillForm} title="Cancel">
            <X class="icon" size={16} strokeWidth={1.5} />
          </button>
        </div>
        <div class="editor-title">
          <h2>New Skill</h2>
        </div>
        <div class="pane-actions">
          <button class="pane-action primary" onclick={handleCreateSkill} disabled={!newSkillName.trim() || !newSkillDescription.trim()} title="Create">
            Create
          </button>
        </div>
      </div>
      <div class="new-skill-container">
        <div class="new-skill-form-centered">
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
              rows="3"
              autocapitalize="off"
              spellcheck="false"
            ></textarea>
          </div>
        </div>
      </div>
    {:else}
      <div class="editor-placeholder-full">
        <div class="placeholder-content">
          <FilePenLine class="placeholder-icon" size={48} strokeWidth={1} />
          <p class="placeholder-title">No Selection</p>
          <p class="placeholder-hint">Select a skill to edit, or create a new one</p>
          <button class="placeholder-action" onclick={() => showNewSkillForm = true}>
            <Plus class="icon" size={16} strokeWidth={2} />
            New Skill
          </button>
        </div>
      </div>
    {/if}
  </div>
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

    /* Layout - 3 Pane */
    --targets-pane-width: 180px;
    --skills-pane-width: 280px;
    --editor-min-width: 720px;
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
     LAYOUT - 3 PANE
     ============================================ */
  .app-container {
    display: flex;
    height: 100vh;
    overflow: hidden;
    min-width: calc(var(--targets-pane-width) + var(--skills-pane-width) + var(--editor-min-width));
  }

  /* ============================================
     SHARED PANE STYLES
     ============================================ */
  .pane-header {
    height: var(--titlebar-height);
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    padding: 0 var(--space-3);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--color-border);
    -webkit-app-region: drag;
    flex-shrink: 0;
    gap: var(--space-2);
  }

  .pane-title {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
    white-space: nowrap;
  }

  .pane-actions {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    -webkit-app-region: no-drag;
  }

  .pane-action {
    width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s ease, color 0.15s ease, transform 0.1s ease;
  }

  .pane-action:hover:not(:disabled) {
    background: var(--color-surface);
    color: var(--color-text);
  }

  .pane-action:active:not(:disabled) {
    transform: scale(0.92);
  }

  .pane-action:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .pane-action.primary {
    background: var(--color-primary);
    color: white;
    width: auto;
    padding: 0 var(--space-3);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
  }

  .pane-action.primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  /* ============================================
     PANE 1: TARGETS
     ============================================ */
  .targets-pane {
    width: var(--targets-pane-width);
    background: var(--color-sidebar);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .targets-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--space-2) var(--space-3);
  }

  .targets-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .target-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--font-xs);
    color: var(--color-text-muted);
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .target-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .target-item:active {
    transform: scale(0.98);
  }

  .target-item.disabled {
    opacity: 0.5;
  }

  .target-icon {
    flex-shrink: 0;
    width: 14px;
    text-align: center;
    font-size: 10px;
  }

  .target-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .target-ready {
    flex-shrink: 0;
    color: var(--color-success);
    font-size: 11px;
    animation: checkmark-pop 0.3s ease;
  }

  @keyframes checkmark-pop {
    0% { transform: scale(0); opacity: 0; }
    50% { transform: scale(1.3); }
    100% { transform: scale(1); opacity: 1; }
  }

  .targets-empty {
    padding: var(--space-4) var(--space-2);
    font-size: var(--font-xs);
    color: var(--color-text-dim);
    text-align: center;
  }

  .targets-toolbar {
    padding: var(--space-3);
    flex-shrink: 0;
  }

  .sync-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .sync-button:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .sync-button:active:not(:disabled) {
    transform: scale(0.97);
  }

  .sync-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .sync-icon,
  .refresh-icon {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sync-icon.spinning {
    animation: spin-cw 0.8s linear infinite;
  }

  .refresh-icon.spinning {
    animation: spin-ccw 0.8s linear infinite;
  }

  @keyframes spin-cw {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @keyframes spin-ccw {
    from { transform: rotate(360deg); }
    to { transform: rotate(0deg); }
  }

  /* Add Target Form */
  .add-target-form {
    padding: var(--space-2);
    margin-bottom: var(--space-2);
    background: var(--color-surface);
    border-radius: var(--radius-md);
  }

  .add-target-form select,
  .add-target-form input {
    width: 100%;
    padding: var(--space-2);
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
    height: 24px;
    padding: 0 var(--space-2);
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

  /* ============================================
     PANE 2: SKILLS
     ============================================ */
  .skills-pane {
    width: var(--skills-pane-width);
    background: var(--color-bg);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .skills-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .skills-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  /* ============================================
     BANNERS (Shared)
     ============================================ */
  .banner {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3);
    font-size: var(--font-xs);
    flex-shrink: 0;
  }

  .banner-error {
    background: rgba(255, 69, 58, 0.12);
    color: var(--color-error);
  }

  .banner-warning {
    background: rgba(255, 159, 10, 0.12);
    color: var(--color-warning);
  }

  .banner-success {
    background: rgba(48, 209, 88, 0.12);
    color: var(--color-success);
  }

  .banner-content {
    flex: 1;
    min-width: 0;
  }

  .banner-details {
    margin-top: var(--space-2);
    opacity: 0.9;
  }

  .banner button {
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    padding: 0;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
  }

  .banner button:hover {
    background: rgba(255, 255, 255, 0.1);
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

  /* ============================================
     SKILL LIST ITEMS
     ============================================ */
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

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8) var(--space-4);
    flex: 1;
  }

  .empty-icon {
    margin-bottom: var(--space-4);
  }

  .empty-icon :global(.empty-icon-sparkle) {
    color: var(--color-text-dim);
    opacity: 0.6;
  }

  .empty-title {
    margin: 0;
    font-size: var(--font-base);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-secondary);
  }

  .empty-subtitle {
    margin: var(--space-2) 0 0 0;
    font-size: var(--font-xs);
    color: var(--color-text-dim);
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
  }

  .skill-item:hover {
    background: rgba(255, 255, 255, 0.04);
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
  }

  .status-dot.valid {
    background: var(--color-success);
  }

  .status-dot.invalid {
    background: var(--color-error);
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
    visibility: hidden;
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
  }

  .skill-item:hover .skill-delete {
    visibility: visible;
  }

  .skill-delete:hover {
    color: var(--color-error);
    background: rgba(255, 69, 58, 0.15);
  }

  /* ============================================
     PANE 3: EDITOR
     ============================================ */
  .editor-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: var(--editor-min-width);
    background: var(--color-bg);
  }

  .editor-header {
    justify-content: space-between;
    background: var(--color-sidebar);
  }

  .editor-header-empty {
    background: var(--color-sidebar);
  }

  .editor-title {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
  }

  .editor-title h2 {
    margin: 0;
    font-size: var(--font-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
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

  .fix-button {
    flex-shrink: 0;
    height: 24px;
    padding: 0 var(--space-3);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-bg);
    background: var(--color-warning);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .fix-button:hover:not(:disabled) {
    background: #ffa340;
  }

  .fix-button:active:not(:disabled) {
    transform: scale(0.96);
  }

  .fix-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
  }

  /* Editor Placeholder */
  .editor-placeholder {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .editor-placeholder-full {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    -webkit-app-region: drag;
  }

  .editor-placeholder-full .placeholder-content {
    -webkit-app-region: no-drag;
  }

  .placeholder-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .placeholder-content :global(.placeholder-icon) {
    color: var(--color-text-dim);
    opacity: 0.4;
    margin-bottom: var(--space-4);
  }

  .placeholder-title {
    margin: 0;
    font-size: var(--font-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-muted);
  }

  .placeholder-hint {
    margin: var(--space-1) 0 0 0;
    font-size: var(--font-sm);
    color: var(--color-text-dim);
  }

  .placeholder-action {
    margin-top: var(--space-5);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-5);
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .placeholder-action :global(.icon) {
    flex-shrink: 0;
  }

  .placeholder-action:hover {
    background: var(--color-primary-hover);
  }

  .placeholder-action:active {
    transform: scale(0.97);
  }

  /* New Skill Form in Editor Pane */
  .new-skill-container {
    flex: 1;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: var(--space-8);
    overflow-y: auto;
  }

  .new-skill-form-centered {
    width: 100%;
    max-width: 480px;
  }

  .new-skill-form-centered .form-field {
    margin-bottom: var(--space-5);
  }

  .new-skill-form-centered label {
    display: block;
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-muted);
    margin-bottom: var(--space-2);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .new-skill-form-centered input,
  .new-skill-form-centered textarea {
    width: 100%;
    padding: var(--space-3);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text);
    font-size: var(--font-base);
    font-family: inherit;
    box-sizing: border-box;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  .new-skill-form-centered textarea {
    resize: vertical;
    min-height: 80px;
    line-height: 1.5;
  }

  .new-skill-form-centered input:focus,
  .new-skill-form-centered textarea:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-muted);
  }

  .new-skill-form-centered input::placeholder,
  .new-skill-form-centered textarea::placeholder {
    color: var(--color-text-dim);
  }

  .new-skill-form-centered .field-hint {
    margin-top: var(--space-2);
    font-size: var(--font-xs);
    color: var(--color-text-muted);
  }

  .new-skill-form-centered .field-hint code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    font-size: 10px;
    padding: 2px 6px;
    background: var(--color-bg);
    border-radius: var(--radius-sm);
    color: var(--color-primary);
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
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.05);
    font-size: var(--font-base);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
    pointer-events: auto;
    animation: snackbar-slide-in 0.2s ease-out;
    min-width: 200px;
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
