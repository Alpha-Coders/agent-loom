<script lang="ts">
  import { X, AlertTriangle } from 'lucide-svelte';

  interface Props {
    skillName: string;
    onconfirm: () => void;
    oncancel: () => void;
  }

  let { skillName, onconfirm, oncancel }: Props = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      oncancel();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={oncancel}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <div class="modal-icon">
        <AlertTriangle size={24} strokeWidth={1.5} />
      </div>
      <h2>Delete Skill</h2>
      <button class="close-button" onclick={oncancel}>
        <X size={18} strokeWidth={1.5} />
      </button>
    </div>

    <div class="modal-body">
      <p class="confirm-question">Delete "{skillName}"?</p>
      <p class="confirm-description">
        This will permanently remove the skill and its symlinks from all targets.
      </p>
    </div>

    <div class="modal-footer">
      <button class="cancel-button" onclick={oncancel}>
        Cancel
      </button>
      <button class="delete-button" onclick={onconfirm}>
        Delete
      </button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
    animation: fade-in 0.15s ease-out;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal {
    width: 340px;
    max-width: 90vw;
    background: var(--color-sidebar);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    animation: slide-up 0.2s ease-out;
  }

  @keyframes slide-up {
    from {
      opacity: 0;
      transform: translateY(10px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--color-border);
  }

  .modal-icon {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 69, 58, 0.15);
    border-radius: var(--radius-md);
    color: var(--color-error);
  }

  .modal-header h2 {
    flex: 1;
    margin: 0;
    font-size: var(--font-base);
    font-weight: var(--font-weight-semibold);
  }

  .close-button {
    width: 28px;
    height: 28px;
    padding: 0;
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .close-button:hover {
    color: var(--color-text);
    background: var(--color-surface);
  }

  .modal-body {
    padding: var(--space-5);
  }

  .confirm-question {
    margin: 0 0 var(--space-2) 0;
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
  }

  .confirm-description {
    margin: 0;
    font-size: var(--font-xs);
    color: var(--color-text-muted);
    line-height: 1.5;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-5);
    border-top: 1px solid var(--color-border);
    background: var(--color-bg);
    border-radius: 0 0 var(--radius-lg) var(--radius-lg);
  }

  .cancel-button,
  .delete-button {
    height: 32px;
    padding: 0 var(--space-4);
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .cancel-button {
    background: var(--color-surface);
    color: var(--color-text-secondary);
  }

  .cancel-button:hover {
    background: var(--color-surface-hover);
  }

  .cancel-button:active {
    transform: scale(0.97);
  }

  .delete-button {
    background: var(--color-error);
    color: white;
    min-width: 80px;
  }

  .delete-button:hover {
    background: #ff6961;
    box-shadow: 0 2px 8px rgba(255, 69, 58, 0.4);
  }

  .delete-button:active {
    transform: scale(0.97);
  }
</style>
