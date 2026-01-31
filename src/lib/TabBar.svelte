<script lang="ts">
  import { Puzzle, Palette, Sun, Moon } from 'lucide-svelte';

  export type Tab = 'skills';
  export type ThemeMode = 'system' | 'light' | 'dark';

  interface Props {
    activeTab: Tab;
    onTabChange: (tab: Tab) => void;
    themeMode: ThemeMode;
    onThemeToggle: () => void;
  }

  let { activeTab, onTabChange, themeMode, onThemeToggle }: Props = $props();

  // Tab definitions - easy to extend by adding more items
  const tabs: { id: Tab; label: string }[] = [
    { id: 'skills', label: 'Skills' },
  ];
</script>

<div class="app-section">
  <div class="app-header">
    <Puzzle class="app-icon" size={16} strokeWidth={1.5} />
    <span class="app-name">AgentLoom</span>
    <button class="theme-toggle" onclick={onThemeToggle} title="Color scheme: {themeMode}">
      {#if themeMode === 'system'}
        <Palette size={14} strokeWidth={1.5} />
      {:else if themeMode === 'light'}
        <Sun size={14} strokeWidth={1.5} />
      {:else}
        <Moon size={14} strokeWidth={1.5} />
      {/if}
    </button>
  </div>
  <div class="tab-list">
    {#each tabs as tab}
      <button
        class="tab-item"
        class:active={activeTab === tab.id}
        onclick={() => onTabChange(tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>
</div>

<style>
  .app-section {
    flex-shrink: 0;
  }

  .app-header {
    height: var(--titlebar-height);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--toolbar-padding-x);
    border-bottom: 1px solid var(--color-border);
    -webkit-app-region: drag;
    transition: border-color var(--theme-transition);
  }

  .app-header :global(.app-icon) {
    color: var(--color-primary);
    flex-shrink: 0;
  }

  .app-name {
    flex: 1;
    font-size: var(--font-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
    transition: color var(--theme-transition);
  }

  .theme-toggle {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    cursor: pointer;
    -webkit-app-region: no-drag;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }

  .theme-toggle:hover {
    background: var(--color-surface);
    border-color: var(--color-surface-hover);
    color: var(--color-text);
  }

  .theme-toggle:active {
    transform: scale(0.96);
  }

  .tab-list {
    display: flex;
    flex-direction: column;
  }

  .tab-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: var(--space-2) var(--space-4);
    background: transparent;
    border: none;
    border-left: 3px solid transparent;
    color: var(--color-text-secondary);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    font-family: inherit;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }

  .tab-item:hover:not(.active) {
    background: rgba(255, 255, 255, 0.04);
  }

  .tab-item:active:not(.active) {
    background: rgba(255, 255, 255, 0.06);
  }

  .tab-item.active {
    background: var(--color-primary-muted);
    border-left-color: var(--color-primary);
    color: var(--color-text);
  }

  /* Light theme adjustments */
  :global([data-theme="light"]) .tab-item:hover:not(.active) {
    background: rgba(0, 0, 0, 0.04);
  }

  :global([data-theme="light"]) .tab-item:active:not(.active) {
    background: rgba(0, 0, 0, 0.06);
  }
</style>
