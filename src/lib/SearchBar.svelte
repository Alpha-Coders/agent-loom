<script lang="ts">
  import { tick } from 'svelte';
  import { Search, ChevronUp, ChevronDown, X } from 'lucide-svelte';

  interface Props {
    totalMatches: number;
    currentMatch: number;
    onsearch: (query: string) => void;
    onnext: () => void;
    onprev: () => void;
    onclose: () => void;
  }

  let { totalMatches, currentMatch, onsearch, onnext, onprev, onclose }: Props = $props();

  let inputEl: HTMLInputElement;
  let query = $state('');

  function handleInput(e: Event) {
    query = (e.target as HTMLInputElement).value;
    onsearch(query);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (e.shiftKey) {
        onprev();
      } else {
        onnext();
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      onclose();
    }
  }

  // Auto-focus input on mount
  $effect(() => {
    tick().then(() => {
      inputEl?.focus();
    });
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="find-bar" role="search" onkeydown={handleKeydown}>
  <Search class="find-bar-icon" size={14} strokeWidth={1.5} />
  <input
    bind:this={inputEl}
    class="find-bar-input"
    type="text"
    placeholder="Find in document..."
    value={query}
    oninput={handleInput}
    spellcheck="false"
    autocomplete="off"
  />
  <span class="find-bar-count">
    {#if query && totalMatches > 0}
      {currentMatch + 1} of {totalMatches}
    {:else if query}
      No matches
    {/if}
  </span>
  <button class="find-bar-btn" onclick={onprev} disabled={totalMatches === 0} title="Previous (Shift+Enter)">
    <ChevronUp size={14} strokeWidth={2} />
  </button>
  <button class="find-bar-btn" onclick={onnext} disabled={totalMatches === 0} title="Next (Enter)">
    <ChevronDown size={14} strokeWidth={2} />
  </button>
  <button class="find-bar-btn" onclick={onclose} title="Close (Esc)">
    <X size={14} strokeWidth={2} />
  </button>
</div>

<style>
  .find-bar {
    position: absolute;
    top: 0;
    right: 16px;
    z-index: 10;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    background: var(--color-sidebar);
    border: 1px solid var(--color-border);
    border-top: none;
    border-radius: 0 0 var(--radius-md) var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    animation: find-bar-slide-in 0.15s ease-out;
  }

  @keyframes find-bar-slide-in {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .find-bar :global(.find-bar-icon) {
    flex-shrink: 0;
    color: var(--color-text-dim);
  }

  .find-bar-input {
    width: 180px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 4px 8px;
    font-size: 13px;
    font-family: inherit;
    color: var(--color-text);
    outline: none;
    transition: border-color 0.15s ease;
  }

  .find-bar-input:focus {
    border-color: var(--color-primary);
  }

  .find-bar-input::placeholder {
    color: var(--color-text-dim);
  }

  .find-bar-count {
    font-size: 11px;
    color: var(--color-text-muted);
    white-space: nowrap;
    min-width: 60px;
    text-align: center;
  }

  .find-bar-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }

  .find-bar-btn:hover:not(:disabled) {
    background: var(--color-surface);
    border-color: var(--color-surface-hover);
    color: var(--color-text);
  }

  .find-bar-btn:active:not(:disabled) {
    transform: scale(0.92);
  }

  .find-bar-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
