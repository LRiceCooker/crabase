// Tauri shim for E2E testing
// Injected via Playwright's addInitScript() BEFORE the WASM loads.
// Routes __TAURI__.core.invoke() calls to the test HTTP server on port 3001.

window.__TAURI__ = {
  core: {
    async invoke(cmd, args = {}) {
      const response = await fetch(`http://localhost:3001/invoke/${cmd}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(args),
      });
      const data = await response.json();
      if (!response.ok) {
        throw data.error || "Unknown error";
      }
      return data;
    },
  },
  event: {
    async listen(_event, _handler) {
      // No-op: return an unlisten function
      return () => {};
    },
  },
  dialog: {
    async open(_options) {
      return null;
    },
    async save(_options) {
      return null;
    },
  },
};
