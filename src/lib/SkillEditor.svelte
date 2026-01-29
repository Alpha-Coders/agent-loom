<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightSpecialChars, drawSelection, dropCursor, highlightActiveLine } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { markdown } from '@codemirror/lang-markdown';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, foldGutter, indentOnInput } from '@codemirror/language';
  import { oneDark } from '@codemirror/theme-one-dark';

  interface Props {
    content: string;
    onchange?: (content: string) => void;
  }

  let { content, onchange }: Props = $props();

  let editorContainer: HTMLDivElement;
  let view: EditorView | null = null;

  function createEditor() {
    if (!editorContainer) return;

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged && onchange) {
        onchange(update.state.doc.toString());
      }
    });

    const state = EditorState.create({
      doc: content,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightSpecialChars(),
        history(),
        foldGutter(),
        drawSelection(),
        dropCursor(),
        indentOnInput(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        bracketMatching(),
        highlightActiveLine(),
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
        ]),
        markdown(),
        oneDark,
        updateListener,
        EditorView.theme({
          '&': {
            height: '100%',
            fontSize: '14px',
          },
          '.cm-scroller': {
            overflow: 'auto',
            fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
          },
          '.cm-content': {
            padding: '12px 0',
          },
          '.cm-line': {
            padding: '0 12px',
          },
          '.cm-gutters': {
            backgroundColor: '#1e293b',
            borderRight: '1px solid #334155',
          },
        }),
      ],
    });

    view = new EditorView({
      state,
      parent: editorContainer,
    });
  }

  function destroyEditor() {
    if (view) {
      view.destroy();
      view = null;
    }
  }

  onMount(() => {
    createEditor();
  });

  onDestroy(() => {
    destroyEditor();
  });

  // Update editor content when content prop changes externally
  $effect(() => {
    if (view && content !== view.state.doc.toString()) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: content,
        },
      });
    }
  });
</script>

<div class="editor-wrapper" bind:this={editorContainer}></div>

<style>
  .editor-wrapper {
    height: 100%;
    width: 100%;
    background-color: #0f172a;
    border-radius: 8px;
    overflow: hidden;
  }

  .editor-wrapper :global(.cm-editor) {
    height: 100%;
  }

  .editor-wrapper :global(.cm-focused) {
    outline: none;
  }
</style>
