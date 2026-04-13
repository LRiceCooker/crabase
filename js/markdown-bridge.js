// Markdown rendering bridge for WASM interop
// Exposes window.__markdown.render(text) → HTML string

import { marked } from "marked";
import hljs from "highlight.js/lib/core";

// Register only common languages to keep bundle small
import sql from "highlight.js/lib/languages/sql";
import json from "highlight.js/lib/languages/json";
import javascript from "highlight.js/lib/languages/javascript";
import typescript from "highlight.js/lib/languages/typescript";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import bash from "highlight.js/lib/languages/bash";
import xml from "highlight.js/lib/languages/xml";
import css from "highlight.js/lib/languages/css";
import yaml from "highlight.js/lib/languages/yaml";

hljs.registerLanguage("sql", sql);
hljs.registerLanguage("json", json);
hljs.registerLanguage("javascript", javascript);
hljs.registerLanguage("js", javascript);
hljs.registerLanguage("typescript", typescript);
hljs.registerLanguage("ts", typescript);
hljs.registerLanguage("python", python);
hljs.registerLanguage("rust", rust);
hljs.registerLanguage("bash", bash);
hljs.registerLanguage("sh", bash);
hljs.registerLanguage("xml", xml);
hljs.registerLanguage("html", xml);
hljs.registerLanguage("css", css);
hljs.registerLanguage("yaml", yaml);

// Configure marked to use highlight.js for code blocks
marked.setOptions({
  highlight: function (code, lang) {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(code, { language: lang }).value;
      } catch (_) {}
    }
    try {
      return hljs.highlightAuto(code).value;
    } catch (_) {}
    return code;
  },
  breaks: true,
  gfm: true,
});

window.__markdown = {
  render(text) {
    try {
      return marked.parse(text);
    } catch (e) {
      console.error("Markdown render error:", e);
      return text;
    }
  },
};
