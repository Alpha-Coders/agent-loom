<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { ask, open as openDialog } from '@tauri-apps/plugin-dialog';
  import { getSkills, getTargets, syncAll, validateAll, refreshSkills, createSkill, deleteSkill, getStats, getSkillContent, saveSkillContent, validateSkill, importAllSkills, toggleTarget, addFolderTarget, fixSkill, scanFolderForSkills, importFromFolder, revealInFinder } from './lib/api';
  import type { SkillInfo, TargetInfo, SyncResult, StatsInfo, ImportResultInfo, ScannedSkillInfo, FolderImportSelectionInfo } from './lib/types';
  import SkillEditor from './lib/SkillEditor.svelte';
  import ImportFromFolderModal from './lib/ImportFromFolderModal.svelte';
  import { Plus, RefreshCw, RotateCcw, Download, X, Sparkles, Trash2, FolderOpen, FilePenLine, Sun, Moon, Monitor } from 'lucide-svelte';

  // Theme state
  type ThemeMode = 'system' | 'light' | 'dark';
  let themeMode = $state<ThemeMode>('system');
  let systemPrefersDark = $state(true);
  let resolvedTheme = $derived(themeMode === 'system' ? (systemPrefersDark ? 'dark' : 'light') : themeMode);

  // Apply theme to document
  $effect(() => {
    document.documentElement.setAttribute('data-theme', resolvedTheme);
  });

  function cycleTheme() {
    const modes: ThemeMode[] = ['system', 'light', 'dark'];
    const currentIndex = modes.indexOf(themeMode);
    themeMode = modes[(currentIndex + 1) % modes.length];
    localStorage.setItem('theme', themeMode);
  }

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


  // Editor state
  let editingSkill = $state<SkillInfo | null>(null);
  let editorContent = $state('');
  let originalContent = $state('');
  let isSaving = $state(false);
  let isFixing = $state(false);

  // Context menu state
  interface ContextMenuItem {
    label: string;
    action: () => void;
    destructive?: boolean;
    disabled?: boolean;
  }
  interface ContextMenu {
    x: number;
    y: number;
    items: ContextMenuItem[];
  }
  let contextMenu = $state<ContextMenu | null>(null);

  function showContextMenu(event: MouseEvent, items: ContextMenuItem[]) {
    event.preventDefault();
    event.stopPropagation();

    // Calculate position, ensuring menu stays within viewport
    let x = event.clientX;
    let y = event.clientY;

    // Approximate menu dimensions for boundary check
    const menuWidth = 180;
    const menuHeight = items.length * 36 + 8;

    if (x + menuWidth > window.innerWidth) {
      x = window.innerWidth - menuWidth - 8;
    }
    if (y + menuHeight > window.innerHeight) {
      y = window.innerHeight - menuHeight - 8;
    }

    contextMenu = { x, y, items };
  }

  function hideContextMenu() {
    contextMenu = null;
  }

  // Snackbar/toast state
  interface Snackbar {
    id: number;
    message: string;
    type: 'success' | 'info' | 'warning' | 'error';
  }
  let snackbars = $state<Snackbar[]>([]);
  let snackbarId = 0;

  function showSnackbar(message: string, type: 'success' | 'info' | 'warning' | 'error' = 'success', duration = 3000) {
    const id = ++snackbarId;
    snackbars = [...snackbars, { id, message, type }];

    // Only errors require manual dismissal; success, info, and warnings auto-dismiss
    if (type !== 'error') {
      setTimeout(() => {
        snackbars = snackbars.filter(s => s.id !== id);
      }, duration);
    }
  }

  function dismissSnackbar(id: number) {
    snackbars = snackbars.filter(s => s.id !== id);
  }

  function formatSyncMessage(created: number, removed: number): string {
    if (created === 0 && removed === 0) {
      return 'All targets up to date';
    }
    if (created > 0 && removed === 0) {
      return `Added ${created} skill${created === 1 ? '' : 's'} to targets`;
    }
    if (created === 0 && removed > 0) {
      return `Removed ${removed} skill${removed === 1 ? '' : 's'} from targets`;
    }
    return `Added ${created}, removed ${removed} skill${created + removed === 1 ? '' : 's'}`;
  }

  let hasUnsavedChanges = $derived(editorContent !== originalContent);
  let hasNewSkillFormInput = $derived(newSkillName.trim() !== '' || newSkillDescription.trim() !== '');
  let hasAnyUnsavedWork = $derived(hasUnsavedChanges || (showNewSkillForm && hasNewSkillFormInput));

  // Show errors as snackbars
  $effect(() => {
    if (error) {
      showSnackbar(error, 'error');
      error = null;
    }
  });

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
        showSnackbar(formatSyncMessage(totalCreated, totalRemoved), 'success');
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

      // Show feedback based on validation state
      if (validatedSkill.validation_status === 'invalid') {
        const errorCount = validatedSkill.validation_errors.length;
        showSnackbar(`Saved with ${errorCount} validation error${errorCount === 1 ? '' : 's'}`, 'warning');
      } else {
        showSnackbar('Saved', 'success');
      }
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

  // === Drag and Drop Handlers (using Tauri native API) ===

  async function handleFileDrop(paths: string[]) {
    if (paths.length === 0) return;

    // Use the first dropped path
    const folderPath = paths[0];
    await scanAndShowModal(folderPath);
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

  // Context menu handlers
  function handleSkillContextMenu(skill: SkillInfo, event: MouseEvent) {
    const items: ContextMenuItem[] = [
      {
        label: 'Edit',
        action: () => handleEditSkill(skill),
      },
      {
        label: 'Reveal in Finder',
        action: async () => {
          try {
            await revealInFinder(skill.path);
          } catch (e) {
            error = e instanceof Error ? e.message : String(e);
          }
        },
      },
      {
        label: 'Delete',
        action: () => handleDeleteSkill(skill, event),
        destructive: true,
      },
    ];
    showContextMenu(event, items);
  }

  function handleTargetContextMenu(target: TargetInfo, event: MouseEvent) {
    const items: ContextMenuItem[] = [
      {
        label: target.enabled ? 'Disable' : 'Enable',
        action: () => handleToggleTarget(target.id),
      },
      {
        label: 'Reveal in Finder',
        action: async () => {
          try {
            await revealInFinder(target.skills_path);
          } catch (e) {
            error = e instanceof Error ? e.message : String(e);
          }
        },
      },
    ];
    showContextMenu(event, items);
  }

  async function handleAddFolderTarget() {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: 'Select folder to sync skills to',
      });

      if (selected && typeof selected === 'string') {
        error = null;
        const newTarget = await addFolderTarget(selected);
        targets = await getTargets();
        stats = await getStats();
        showSnackbar(`Added "${newTarget.name}"`, 'success');
      }
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
      if (contextMenu) {
        hideContextMenu();
      } else if (showNewSkillForm) {
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
    // Initialize theme from localStorage
    const savedTheme = localStorage.getItem('theme') as ThemeMode | null;
    if (savedTheme && ['system', 'light', 'dark'].includes(savedTheme)) {
      themeMode = savedTheme;
    }

    // Detect system preference
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    systemPrefersDark = mediaQuery.matches;
    const handleChange = (e: MediaQueryListEvent) => {
      systemPrefersDark = e.matches;
    };
    mediaQuery.addEventListener('change', handleChange);
    unlistenFns.push(() => mediaQuery.removeEventListener('change', handleChange));

    loadData();

    // Get window reference for close handling
    const currentWindow = getCurrentWindow();

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

    // Listen for native file drag-drop events from OS (Finder, etc.)
    const webview = getCurrentWebview();
    unlistenFns.push(await webview.onDragDropEvent((event) => {
      if (event.payload.type === 'enter' || event.payload.type === 'over') {
        isDragging = true;
      } else if (event.payload.type === 'leave') {
        isDragging = false;
      } else if (event.payload.type === 'drop') {
        isDragging = false;
        handleFileDrop(event.payload.paths);
      }
    }));

    // Handle window close with unsaved changes confirmation
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
  role="application"
>
  <!-- Pane 1: Targets -->
  <aside class="targets-pane">
    <div class="pane-header">
      <span class="pane-title">Targets</span>
      <div class="pane-actions">
        <button class="pane-action" onclick={handleAddFolderTarget} title="Add folder as target">
          <Plus class="icon" size={16} strokeWidth={1.5} />
        </button>
      </div>
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
      <div class="targets-list">
        {#each targets as target}
          {@const syncStatus = target.sync_status}
          {@const isOutOfSync = syncStatus && !syncStatus.is_synced}
          {@const syncIssues = isOutOfSync ? [
            ...syncStatus.missing_skills.length ? [`${syncStatus.missing_skills.length} missing`] : [],
            ...syncStatus.extra_items.length ? [`${syncStatus.extra_items.length} unmanaged`] : [],
            ...syncStatus.broken_links.length ? [`${syncStatus.broken_links.length} broken`] : []
          ].join(', ') : ''}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="target-item"
            class:disabled={!target.enabled}
            onclick={() => handleToggleTarget(target.id)}
            oncontextmenu={(e) => handleTargetContextMenu(target, e)}
            title={target.enabled
              ? isOutOfSync
                ? `Out of sync: ${syncIssues}\nClick to disable • Right-click for options`
                : 'Synced • Click to disable • Right-click for options'
              : 'Click to enable • Right-click for options'}
          >
            <span class="target-icon">{target.enabled ? '◉' : '○'}</span>
            <span class="target-name">{target.name}</span>
            {#if target.enabled && target.exists}
              {#if isOutOfSync}
                <span class="target-warning" title="Out of sync: {syncIssues}">●</span>
              {:else}
                <span class="target-ready">✓</span>
              {/if}
            {/if}
          </div>
        {/each}
        {#if targets.length === 0}
          <div class="targets-empty">No targets detected</div>
        {/if}
      </div>
    </div>

    <div class="targets-footer">
      <button class="theme-toggle" onclick={cycleTheme} title="Color scheme">
        {#if themeMode === 'system'}
          <Monitor class="icon" size={14} strokeWidth={1.5} />
          <span>System</span>
        {:else if themeMode === 'light'}
          <Sun class="icon" size={14} strokeWidth={1.5} />
          <span>Light</span>
        {:else}
          <Moon class="icon" size={14} strokeWidth={1.5} />
          <span>Dark</span>
        {/if}
      </button>
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
        <button class="pane-action" onclick={handleImport} disabled={isImporting} title="Import skills from AI tools">
          <Download class="icon" size={16} strokeWidth={1.5} />
        </button>
        <button class="pane-action" onclick={handleFolderPickerImport} disabled={isScanning} title="Import from folder">
          <FolderOpen class="icon" size={16} strokeWidth={1.5} />
        </button>
      </div>
    </div>

    <div class="skills-content">
      {#if lastSyncResults.length > 0}
        {@const totalCreated = lastSyncResults.reduce((sum, r) => sum + r.created.length, 0)}
        {@const totalRemoved = lastSyncResults.reduce((sum, r) => sum + r.removed.length, 0)}
        {@const allErrors = lastSyncResults.flatMap(r => r.errors.map(e => ({ target: r.target_name, ...e })))}
        {@const totalErrors = allErrors.length}
        <div class="banner" class:banner-warning={totalErrors > 0} class:banner-success={totalErrors === 0}>
          <div class="banner-content">
            <span>
              {#if totalErrors > 0}⚠{:else}✓{/if}
              {formatSyncMessage(totalCreated, totalRemoved)}
              {#if totalErrors > 0} ({totalErrors} error{totalErrors === 1 ? '' : 's'}){/if}
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
              oncontextmenu={(e) => handleSkillContextMenu(skill, e)}
              title="Click to edit • Right-click for options"
            >
              <div class="skill-status">
                <span class="status-dot" class:valid={skill.validation_status === 'valid'} class:invalid={skill.validation_status === 'invalid'}></span>
              </div>
              <div class="skill-info">
                <div class="skill-name">{skill.name}</div>
                <div class="skill-description">{skill.description}</div>
              </div>
              <button class="skill-delete" onclick={(e) => handleDeleteSkill(skill, e)} title="Delete skill">
                <Trash2 class="icon" size={16} strokeWidth={1.5} />
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
            <div class="field-hint">
              Lowercase letters, numbers, and hyphens only.
              {#if newSkillName.trim()}
                <br />Creates: <code>~/.agentloom/skills/{newSkillName.trim().toLowerCase().replace(/\s+/g, '-')}/</code>
              {/if}
            </div>
          </div>

          <div class="form-field">
            <label for="skill-description">Description</label>
            <textarea
              id="skill-description"
              placeholder="Describe what the skill does and when to use it. Include keywords that help agents identify relevant tasks."
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

<!-- Drop Zone Overlay (full window) -->
{#if isDragging}
  <div class="drop-zone-overlay">
    <div class="drop-zone-content">
      <FolderOpen class="drop-zone-icon" size={48} strokeWidth={1.5} />
      <p class="drop-zone-title">Drop Skill Folders</p>
      <p class="drop-zone-hint">Import skills from a folder</p>
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

<!-- Context Menu -->
{#if contextMenu}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="context-menu-backdrop" onclick={hideContextMenu}></div>
  <div
    class="context-menu"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
  >
    {#each contextMenu.items as item}
      <button
        class="context-menu-item"
        class:destructive={item.destructive}
        disabled={item.disabled}
        onclick={() => { item.action(); hideContextMenu(); }}
      >
        {item.label}
      </button>
    {/each}
  </div>
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

    /* Layout - 3 Pane */
    --targets-pane-width: 180px;
    --skills-pane-width: 280px;
    --editor-min-width: 720px;
  }

  /* ============================================
     THEME: DARK (Default)
     ============================================ */
  :root, :root[data-theme="dark"] {
    --color-bg: #1c1c1e;
    --color-sidebar: #2c2c2e;
    --color-surface: #3a3a3c;
    --color-surface-hover: #48484a;
    --color-border: #3d3d3f;

    --color-text: #ffffff;
    --color-text-secondary: rgba(255, 255, 255, 0.85);
    --color-text-muted: rgba(255, 255, 255, 0.55);
    --color-text-dim: rgba(255, 255, 255, 0.35);

    --color-primary: #0a84ff;
    --color-primary-hover: #409cff;
    --color-primary-muted: rgba(10, 132, 255, 0.18);

    --color-success: #30d158;
    --color-warning: #ff9f0a;
    --color-error: #ff453a;
  }

  /* ============================================
     THEME: LIGHT
     ============================================ */
  :root[data-theme="light"] {
    --color-bg: #f5f5f7;
    --color-sidebar: #ffffff;
    --color-surface: #e8e8ed;
    --color-surface-hover: #d8d8dd;
    --color-border: #d1d1d6;

    --color-text: #1d1d1f;
    --color-text-secondary: rgba(0, 0, 0, 0.85);
    --color-text-muted: rgba(0, 0, 0, 0.55);
    --color-text-dim: rgba(0, 0, 0, 0.35);

    --color-primary: #007aff;
    --color-primary-hover: #0051a8;
    --color-primary-muted: rgba(0, 122, 255, 0.12);

    --color-success: #34c759;
    --color-warning: #ff9500;
    --color-error: #ff3b30;
  }

  /* ============================================
     THEME TRANSITION
     ============================================ */
  :root {
    --theme-transition: 0.3s ease;
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
    transition:
      background-color var(--theme-transition),
      color var(--theme-transition);
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
    padding: 0 var(--space-4);
    padding-bottom: var(--space-2);
    border-bottom: 1px solid var(--color-border);
    -webkit-app-region: drag;
    flex-shrink: 0;
    gap: var(--space-3);
    transition: border-color var(--theme-transition);
  }

  .pane-title {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
    white-space: nowrap;
    line-height: 28px; /* Match button height for alignment */
    height: 28px;
    transition: color var(--theme-transition);
  }

  .pane-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    -webkit-app-region: no-drag;
    height: 28px;
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
    -webkit-app-region: no-drag;
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
    height: 26px;
    padding: 0 var(--space-3);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    border-radius: var(--radius-sm);
    transition: background 0.15s ease, box-shadow 0.2s ease, transform 0.1s ease;
  }

  .pane-action.primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
    box-shadow: 0 2px 12px rgba(10, 132, 255, 0.4);
  }

  .pane-action.primary:active:not(:disabled) {
    box-shadow: 0 1px 6px rgba(10, 132, 255, 0.3);
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
    transition:
      background-color var(--theme-transition),
      border-color var(--theme-transition);
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

  .target-warning {
    flex-shrink: 0;
    color: var(--color-warning);
    font-size: 10px;
    cursor: help;
    animation: warning-pulse 2s ease-in-out infinite;
  }

  @keyframes checkmark-pop {
    0% { transform: scale(0); opacity: 0; }
    50% { transform: scale(1.3); }
    100% { transform: scale(1); opacity: 1; }
  }

  @keyframes warning-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .targets-empty {
    padding: var(--space-4) var(--space-2);
    font-size: var(--font-xs);
    color: var(--color-text-dim);
    text-align: center;
  }

  .targets-footer {
    padding: var(--space-2) var(--space-3);
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .theme-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-2);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    font-size: var(--font-xs);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .theme-toggle:hover {
    background: rgba(255, 255, 255, 0.05);
    color: var(--color-text);
  }

  .theme-toggle:active {
    transform: scale(0.96);
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
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.2s ease;
  }

  .sync-button:hover:not(:disabled) {
    background: var(--color-primary-hover);
    box-shadow: 0 4px 16px rgba(10, 132, 255, 0.4);
  }

  .sync-button:active:not(:disabled) {
    transform: scale(0.97);
    box-shadow: 0 2px 8px rgba(10, 132, 255, 0.3);
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
    transition:
      background-color var(--theme-transition),
      border-color var(--theme-transition);
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
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
    font-size: var(--font-sm);
    flex-shrink: 0;
    line-height: 1.4;
    overflow: hidden;
    animation: banner-slide-in 0.25s ease-out;
  }

  @keyframes banner-slide-in {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
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
    flex: 1 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .banner-details {
    margin-top: var(--space-2);
    opacity: 0.9;
  }

  .banner > button:not(.fix-button) {
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

  .banner > button:not(.fix-button):hover {
    background: rgba(255, 255, 255, 0.1);
  }

  /* ============================================
     FORM UTILITIES
     ============================================ */
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
    animation: float 3s ease-in-out infinite;
  }

  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-6px); }
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
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    min-height: 64px;
    box-sizing: border-box;
    position: relative;
    transition:
      background 0.2s ease,
      transform 0.1s ease,
      border-color var(--theme-transition);
  }

  .skill-item::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--color-primary);
    opacity: 0;
    transform: scaleY(0);
    transition: opacity 0.2s ease, transform 0.2s ease;
  }

  .skill-item:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .skill-item.selected {
    background: var(--color-primary-muted);
  }

  .skill-item.selected::before {
    opacity: 1;
    transform: scaleY(1);
  }

  .skill-item:active {
    background: rgba(255, 255, 255, 0.06);
    transform: scale(0.995);
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
    transition: box-shadow 0.3s ease, background 0.2s ease;
  }

  .status-dot.valid {
    background: var(--color-success);
    box-shadow: 0 0 8px rgba(48, 209, 88, 0.5);
  }

  .status-dot.invalid {
    background: var(--color-error);
    box-shadow: 0 0 8px rgba(255, 69, 58, 0.5);
  }

  .skill-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .skill-name {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: color var(--theme-transition);
  }

  .skill-description {
    font-size: var(--font-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    transition: color var(--theme-transition);
    line-clamp: 2;
    -webkit-box-orient: vertical;
    line-height: 1.4;
  }

  .skill-delete {
    flex-shrink: 0;
    opacity: 0;
    width: 28px;
    height: 28px;
    padding: 0;
    background: none;
    border: none;
    color: var(--color-text-dim);
    font-size: 16px;
    cursor: pointer;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: opacity 0.15s ease, background 0.15s ease, color 0.15s ease, transform 0.1s ease;
  }

  .skill-item:hover .skill-delete {
    opacity: 1;
  }

  .skill-delete:hover {
    color: var(--color-error);
    background: rgba(255, 69, 58, 0.15);
  }

  .skill-delete:active {
    transform: scale(0.9);
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
    transition: background-color var(--theme-transition);
  }

  .editor-header {
    background: var(--color-sidebar);
    transition: background-color var(--theme-transition);
  }

  .editor-title {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .editor-title h2 {
    margin: 0;
    font-size: var(--font-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
    line-height: 1.2;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
    flex: 0 0 auto;
    height: 28px;
    padding: 0 var(--space-3);
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-bg);
    background: var(--color-warning);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
    white-space: nowrap;
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
    animation: editor-fade-in 0.2s ease-out;
  }

  @keyframes editor-fade-in {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
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
    animation: float 3s ease-in-out infinite;
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
    transition: background 0.15s ease, transform 0.1s ease, box-shadow 0.2s ease;
  }

  .placeholder-action :global(.icon) {
    flex-shrink: 0;
  }

  .placeholder-action:hover {
    background: var(--color-primary-hover);
    box-shadow: 0 4px 20px rgba(10, 132, 255, 0.4);
  }

  .placeholder-action:active {
    transform: scale(0.97);
    box-shadow: 0 2px 10px rgba(10, 132, 255, 0.3);
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
    transition:
      border-color 0.15s ease,
      box-shadow 0.15s ease,
      background-color var(--theme-transition),
      color var(--theme-transition);
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
    flex-direction: column-reverse;
    gap: var(--space-3);
    z-index: 1000;
    pointer-events: none;
  }

  .snackbar {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-6);
    background: var(--color-sidebar);
    border-radius: var(--radius-lg);
    box-shadow:
      0 4px 12px rgba(0, 0, 0, 0.3),
      0 12px 40px rgba(0, 0, 0, 0.4),
      inset 0 1px 0 rgba(255, 255, 255, 0.06);
    font-size: var(--font-base);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
    pointer-events: auto;
    animation: snackbar-slide-up 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
    min-width: 280px;
    max-width: 480px;
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  @keyframes snackbar-slide-up {
    from {
      opacity: 0;
      transform: translateY(16px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .snackbar span {
    flex: 1;
    line-height: 1.4;
  }

  .snackbar button {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    padding: 0;
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .snackbar button:hover {
    color: var(--color-text);
    background: rgba(255, 255, 255, 0.1);
  }

  .snackbar-success {
    border-left: 4px solid var(--color-success);
  }

  .snackbar-info {
    border-left: 4px solid var(--color-primary);
  }

  .snackbar-warning {
    border-left: 4px solid var(--color-warning);
  }

  .snackbar-error {
    border-left: 4px solid var(--color-error);
  }

  /* ============================================
     DROP ZONE OVERLAY (Full Window)
     ============================================ */
  .drop-zone-overlay {
    position: fixed;
    inset: 0;
    background: rgba(10, 132, 255, 0.12);
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
    text-align: center;
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
    margin-bottom: var(--space-4);
  }

  .drop-zone-title {
    margin: 0;
    font-size: var(--font-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
  }

  .drop-zone-hint {
    margin: var(--space-2) 0 0 0;
    font-size: var(--font-sm);
    color: var(--color-text-muted);
  }

  /* ============================================
     CONTEXT MENU
     ============================================ */
  .context-menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
  }

  .context-menu {
    position: fixed;
    z-index: 1001;
    min-width: 160px;
    padding: var(--space-1);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.05);
    animation: context-menu-appear 0.1s ease-out;
  }

  @keyframes context-menu-appear {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .context-menu-item {
    display: block;
    width: 100%;
    padding: var(--space-2) var(--space-3);
    background: none;
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-text);
    font-size: var(--font-sm);
    font-family: inherit;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s ease;
  }

  .context-menu-item:hover:not(:disabled) {
    background: var(--color-primary-muted);
  }

  .context-menu-item:active:not(:disabled) {
    background: var(--color-primary);
    color: white;
  }

  .context-menu-item:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .context-menu-item.destructive {
    color: var(--color-error);
  }

  .context-menu-item.destructive:hover:not(:disabled) {
    background: rgba(255, 69, 58, 0.15);
  }

  .context-menu-item.destructive:active:not(:disabled) {
    background: var(--color-error);
    color: white;
  }
</style>
