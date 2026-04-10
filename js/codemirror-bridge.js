// CodeMirror 6 bridge for WASM interop
// Exposes CodeMirror editor lifecycle on window.__codemirror

import { EditorState, Compartment } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter, drawSelection, rectangularSelection, crosshairCursor, highlightSpecialChars, placeholder as cmPlaceholder } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { sql, PostgreSQL } from "@codemirror/lang-sql";
import { json } from "@codemirror/lang-json";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
import { oneDark } from "@codemirror/theme-one-dark";
import { syntaxHighlighting, defaultHighlightStyle, indentOnInput, bracketMatching, foldGutter, foldKeymap } from "@codemirror/language";

// Store active editor instances by ID
const editors = new Map();
let nextId = 1;

// Light theme (minimal overrides — uses default CodeMirror light look)
const lightTheme = EditorView.theme({
  "&": {
    backgroundColor: "#ffffff",
    color: "#111827",
    fontSize: "13px",
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
  },
  ".cm-content": {
    caretColor: "#6366F1",
    lineHeight: "1.625",
  },
  ".cm-cursor": {
    borderLeftColor: "#6366F1",
  },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
    backgroundColor: "#EEF2FF",
  },
  ".cm-activeLine": {
    backgroundColor: "#F9FAFB",
  },
  ".cm-activeLineGutter": {
    backgroundColor: "#F3F4F6",
  },
  ".cm-gutters": {
    backgroundColor: "#F9FAFB",
    color: "#9CA3AF",
    borderRight: "1px solid #F3F4F6",
  },
  ".cm-tooltip": {
    backgroundColor: "#ffffff",
    border: "1px solid #E5E7EB",
    borderRadius: "6px",
    boxShadow: "0 4px 6px -1px rgba(0,0,0,0.1)",
  },
  ".cm-tooltip-autocomplete": {
    "& > ul > li[aria-selected]": {
      backgroundColor: "#EEF2FF",
      color: "#4338CA",
    },
  },
}, { dark: false });

// Dark theme extension (supplements oneDark with crabase palette tweaks)
const darkThemeExt = EditorView.theme({
  "&": {
    backgroundColor: "#0D0D0F",
    fontSize: "13px",
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
  },
  ".cm-content": {
    caretColor: "#818CF8",
    lineHeight: "1.625",
  },
  ".cm-cursor": {
    borderLeftColor: "#818CF8",
  },
  ".cm-gutters": {
    backgroundColor: "#0F0F11",
    color: "#71717A",
    borderRight: "1px solid #1F1F23",
  },
  ".cm-activeLine": {
    backgroundColor: "rgba(255,255,255,0.03)",
  },
  ".cm-activeLineGutter": {
    backgroundColor: "rgba(255,255,255,0.05)",
  },
}, { dark: true });

function buildExtensions(opts, langCompartment) {
  const isDark = opts.isDark || false;
  const lang = opts.language || "sql"; // "sql" | "json"
  const readOnly = opts.readOnly || false;
  const onChangeCallback = opts.onChange || null;
  const placeholder = opts.placeholder || "";
  const schema = opts.schema || null;

  const exts = [
    lineNumbers(),
    highlightActiveLine(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    drawSelection(),
    rectangularSelection(),
    crosshairCursor(),
    indentOnInput(),
    bracketMatching(),
    closeBrackets(),
    highlightSelectionMatches(),
    foldGutter(),
    history(),
    keymap.of([
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...completionKeymap,
      indentWithTab,
    ]),
    autocompletion(),
    EditorView.lineWrapping,
  ];

  // Language (via compartment for dynamic reconfiguration)
  const langExt = buildLangExtension(lang, schema);
  if (langCompartment) {
    exts.push(langCompartment.of(langExt));
  } else {
    exts.push(langExt);
  }

  // Theme
  if (isDark) {
    exts.push(oneDark);
    exts.push(darkThemeExt);
  } else {
    exts.push(syntaxHighlighting(defaultHighlightStyle, { fallback: true }));
    exts.push(lightTheme);
  }

  // Placeholder
  if (placeholder) {
    exts.push(cmPlaceholder(placeholder));
  }

  // Read-only
  if (readOnly) {
    exts.push(EditorState.readOnly.of(true));
  }

  // Change listener
  if (onChangeCallback) {
    exts.push(EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        onChangeCallback(update.state.doc.toString());
      }
    }));
  }

  return exts;
}

function buildLangExtension(lang, schema) {
  if (lang === "sql") {
    const sqlOpts = { dialect: PostgreSQL };
    if (schema) {
      sqlOpts.schema = schema;
    }
    return sql(sqlOpts);
  } else if (lang === "json") {
    return json();
  }
  return [];
}

window.__codemirror = {
  /**
   * Create a CodeMirror editor inside the given DOM element.
   * @param {HTMLElement} parent - The container element
   * @param {Object} opts - { content, isDark, language, readOnly, placeholder }
   * @returns {number} Editor ID
   */
  create(parent, opts) {
    const id = nextId++;
    const content = opts.content || "";
    const onChangeCallbacks = [];
    const langCompartment = new Compartment();

    const state = EditorState.create({
      doc: content,
      extensions: buildExtensions({
        ...opts,
        onChange: (newContent) => {
          for (const cb of onChangeCallbacks) {
            cb(newContent);
          }
        },
      }, langCompartment),
    });

    const view = new EditorView({
      state,
      parent,
    });

    editors.set(id, {
      view,
      parent,
      opts,
      onChangeCallbacks,
      langCompartment,
      cleanContent: content, // for dirty tracking
    });

    return id;
  },

  /**
   * Destroy an editor instance.
   * @param {number} id
   */
  destroy(id) {
    const entry = editors.get(id);
    if (entry) {
      entry.view.destroy();
      editors.delete(id);
    }
  },

  /**
   * Get the current editor content.
   * @param {number} id
   * @returns {string}
   */
  getContent(id) {
    const entry = editors.get(id);
    if (!entry) return "";
    return entry.view.state.doc.toString();
  },

  /**
   * Set the editor content (replaces everything).
   * @param {number} id
   * @param {string} content
   */
  setContent(id, content) {
    const entry = editors.get(id);
    if (!entry) return;
    const view = entry.view;
    view.dispatch({
      changes: {
        from: 0,
        to: view.state.doc.length,
        insert: content,
      },
    });
  },

  /**
   * Focus the editor.
   * @param {number} id
   */
  focus(id) {
    const entry = editors.get(id);
    if (entry) {
      entry.view.focus();
    }
  },

  /**
   * Check if the editor content has changed since the last markClean().
   * @param {number} id
   * @returns {boolean}
   */
  isDirty(id) {
    const entry = editors.get(id);
    if (!entry) return false;
    return entry.view.state.doc.toString() !== entry.cleanContent;
  },

  /**
   * Mark the current content as "clean" (dirty tracking baseline).
   * @param {number} id
   */
  markClean(id) {
    const entry = editors.get(id);
    if (entry) {
      entry.cleanContent = entry.view.state.doc.toString();
    }
  },

  /**
   * Register a callback for content changes.
   * @param {number} id
   * @param {Function} callback - receives (newContent: string)
   */
  onChange(id, callback) {
    const entry = editors.get(id);
    if (entry) {
      entry.onChangeCallbacks.push(callback);
    }
  },

  /**
   * Switch the editor theme (recreates with new extensions).
   * @param {number} id
   * @param {boolean} isDark
   */
  setTheme(id, isDark) {
    const entry = editors.get(id);
    if (!entry) return;

    const content = entry.view.state.doc.toString();
    const cursorPos = entry.view.state.selection.main.head;
    const onChangeCallbacks = entry.onChangeCallbacks;
    const langCompartment = new Compartment();

    entry.view.destroy();

    const newOpts = { ...entry.opts, isDark };
    const state = EditorState.create({
      doc: content,
      extensions: buildExtensions({
        ...newOpts,
        onChange: (newContent) => {
          for (const cb of onChangeCallbacks) {
            cb(newContent);
          }
        },
      }, langCompartment),
      selection: { anchor: Math.min(cursorPos, content.length) },
    });

    const view = new EditorView({
      state,
      parent: entry.parent,
    });

    entry.view = view;
    entry.opts = newOpts;
    entry.langCompartment = langCompartment;
  },

  /**
   * Set schema for SQL autocompletion (table names + columns).
   * @param {number} id
   * @param {Object} schema - e.g. { users: ["id", "name"], posts: ["id", "title"] }
   */
  setSchema(id, schema) {
    const entry = editors.get(id);
    if (!entry) return;

    const lang = entry.opts.language || "sql";
    entry.opts.schema = schema;
    const langExt = buildLangExtension(lang, schema);
    entry.view.dispatch({
      effects: entry.langCompartment.reconfigure(langExt),
    });
  },
};
