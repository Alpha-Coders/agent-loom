<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightSpecialChars, drawSelection, dropCursor, highlightActiveLine } from '@codemirror/view';
  import { EditorState, Compartment } from '@codemirror/state';
  import { markdown } from '@codemirror/lang-markdown';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, foldGutter, indentOnInput, HighlightStyle } from '@codemirror/language';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { tags } from '@lezer/highlight';

  interface Props {
    content: string;
    onchange?: (content: string) => void;
  }

  let { content, onchange }: Props = $props();

  let editorContainer: HTMLDivElement;
  let view: EditorView | null = null;
  let themeCompartment = new Compartment();
  let scrollTimeout: ReturnType<typeof setTimeout> | null = null;

  // Light theme for CodeMirror
  const lightTheme = EditorView.theme({
    '&': {
      backgroundColor: '#ffffff',
      color: '#1d1d1f',
    },
    '.cm-content': {
      caretColor: '#007aff',
    },
    '.cm-cursor': {
      borderLeftColor: '#007aff',
    },
    '.cm-selectionBackground, ::selection': {
      backgroundColor: 'rgba(0, 122, 255, 0.2)',
    },
    '.cm-activeLine': {
      backgroundColor: 'rgba(0, 0, 0, 0.04)',
    },
    '.cm-activeLineGutter': {
      backgroundColor: 'rgba(0, 0, 0, 0.06)',
    },
    '.cm-gutters': {
      backgroundColor: '#f5f5f7',
      color: 'rgba(0, 0, 0, 0.4)',
      borderRight: '1px solid #d1d1d6',
    },
    '.cm-lineNumbers .cm-gutterElement': {
      color: 'rgba(0, 0, 0, 0.35)',
    },
  }, { dark: false });

  const lightHighlightStyle = HighlightStyle.define([
    { tag: tags.keyword, color: '#af00db' },
    { tag: tags.comment, color: '#6a9955', fontStyle: 'italic' },
    { tag: tags.string, color: '#a31515' },
    { tag: tags.number, color: '#098658' },
    { tag: tags.heading, color: '#0550ae', fontWeight: 'bold' },
    { tag: tags.link, color: '#0969da' },
    { tag: tags.emphasis, fontStyle: 'italic' },
    { tag: tags.strong, fontWeight: 'bold' },
  ]);

  // Dark theme overrides
  const darkThemeOverrides = EditorView.theme({
    '&': {
      height: '100%',
      fontSize: '13px',
    },
    '.cm-scroller': {
      overflow: 'auto',
      fontFamily: 'ui-monospace, SF Mono, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
    },
    '.cm-content': {
      padding: '12px 0',
    },
    '.cm-line': {
      padding: '0 12px',
    },
    '.cm-gutters': {
      backgroundColor: '#1c1c1e',
      borderRight: '1px solid #38383a',
    },
  }, { dark: true });

  // Light theme with overrides
  const lightThemeOverrides = EditorView.theme({
    '&': {
      height: '100%',
      fontSize: '13px',
    },
    '.cm-scroller': {
      overflow: 'auto',
      fontFamily: 'ui-monospace, SF Mono, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
    },
    '.cm-content': {
      padding: '12px 0',
    },
    '.cm-line': {
      padding: '0 12px',
    },
  }, { dark: false });

  function getThemeExtensions(isDark: boolean) {
    if (isDark) {
      return [oneDark, darkThemeOverrides];
    } else {
      return [lightTheme, syntaxHighlighting(lightHighlightStyle), lightThemeOverrides];
    }
  }

  function getCurrentTheme(): boolean {
    return document.documentElement.getAttribute('data-theme') !== 'light';
  }

  function createEditor() {
    if (!editorContainer) return;

    const isDark = getCurrentTheme();

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
        themeCompartment.of(getThemeExtensions(isDark)),
        EditorView.lineWrapping,
        updateListener,
      ],
    });

    view = new EditorView({
      state,
      parent: editorContainer,
    });

    // Add scroll listener for auto-hide scrollbar effect
    const scroller = editorContainer.querySelector('.cm-scroller');
    if (scroller) {
      scroller.addEventListener('scroll', handleScroll);
    }
  }

  function handleScroll() {
    // Add scrolling class to show scrollbar
    editorContainer?.classList.add('is-scrolling');

    // Clear existing timeout
    if (scrollTimeout) {
      clearTimeout(scrollTimeout);
    }

    // Hide scrollbar after 1s of inactivity
    scrollTimeout = setTimeout(() => {
      editorContainer?.classList.remove('is-scrolling');
    }, 1000);
  }

  function destroyEditor() {
    if (scrollTimeout) {
      clearTimeout(scrollTimeout);
      scrollTimeout = null;
    }
    const scroller = editorContainer?.querySelector('.cm-scroller');
    if (scroller) {
      scroller.removeEventListener('scroll', handleScroll);
    }
    if (view) {
      view.destroy();
      view = null;
    }
  }

  function updateTheme() {
    if (view) {
      const isDark = getCurrentTheme();
      view.dispatch({
        effects: themeCompartment.reconfigure(getThemeExtensions(isDark)),
      });
    }
  }

  // Watch for theme changes
  let observer: MutationObserver | null = null;

  onMount(() => {
    createEditor();

    // Watch for theme attribute changes on html element
    observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.attributeName === 'data-theme') {
          updateTheme();
        }
      }
    });
    observer.observe(document.documentElement, { attributes: true });
  });

  onDestroy(() => {
    destroyEditor();
    observer?.disconnect();
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
    background-color: var(--color-sidebar);
    overflow: hidden;
  }

  .editor-wrapper :global(.cm-editor) {
    height: 100%;
  }

  .editor-wrapper :global(.cm-focused) {
    outline: none;
  }

  /* Native scrollbar auto-hide for CodeMirror */
  .editor-wrapper :global(.cm-scroller) {
    scrollbar-width: thin;
    scrollbar-color: transparent transparent;
  }

  .editor-wrapper :global(.cm-scroller::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }

  .editor-wrapper :global(.cm-scroller::-webkit-scrollbar-track) {
    background: transparent;
  }

  .editor-wrapper :global(.cm-scroller::-webkit-scrollbar-thumb) {
    background: transparent;
    border-radius: 4px;
  }

  /* Show scrollbar when scrolling or hovering */
  .editor-wrapper.is-scrolling :global(.cm-scroller::-webkit-scrollbar-thumb),
  .editor-wrapper:hover :global(.cm-scroller::-webkit-scrollbar-thumb) {
    background: rgba(128, 128, 128, 0.4);
  }

  .editor-wrapper.is-scrolling :global(.cm-scroller::-webkit-scrollbar-thumb:hover),
  .editor-wrapper:hover :global(.cm-scroller::-webkit-scrollbar-thumb:hover) {
    background: rgba(128, 128, 128, 0.6);
  }

  .editor-wrapper.is-scrolling :global(.cm-scroller),
  .editor-wrapper:hover :global(.cm-scroller) {
    scrollbar-color: rgba(128, 128, 128, 0.4) transparent;
  }

  /* Light theme */
  :global([data-theme="light"]) .editor-wrapper.is-scrolling :global(.cm-scroller::-webkit-scrollbar-thumb),
  :global([data-theme="light"]) .editor-wrapper:hover :global(.cm-scroller::-webkit-scrollbar-thumb) {
    background: rgba(0, 0, 0, 0.25);
  }

  :global([data-theme="light"]) .editor-wrapper.is-scrolling :global(.cm-scroller::-webkit-scrollbar-thumb:hover),
  :global([data-theme="light"]) .editor-wrapper:hover :global(.cm-scroller::-webkit-scrollbar-thumb:hover) {
    background: rgba(0, 0, 0, 0.4);
  }

  :global([data-theme="light"]) .editor-wrapper.is-scrolling :global(.cm-scroller),
  :global([data-theme="light"]) .editor-wrapper:hover :global(.cm-scroller) {
    scrollbar-color: rgba(0, 0, 0, 0.25) transparent;
  }
</style>
