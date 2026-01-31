<script lang="ts">
  export type Tab = 'skills';

  interface Props {
    activeTab: Tab;
    onTabChange: (tab: Tab) => void;
  }

  let { activeTab, onTabChange }: Props = $props();

  // Tab definitions - easy to extend by adding more items
  const tabs: { id: Tab; label: string }[] = [
    { id: 'skills', label: 'Skills' },
  ];
</script>

<div class="app-section">
  <div class="app-header">
    <span class="app-name">AgentLoom</span>
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
    padding: 20px var(--toolbar-padding-x) 0;
    border-bottom: 1px solid var(--color-border);
    -webkit-app-region: drag;
    transition: border-color var(--theme-transition);
  }

  .app-name {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
    transition: color var(--theme-transition);
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
