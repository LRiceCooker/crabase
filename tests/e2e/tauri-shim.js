// Tauri shim for E2E testing
// Injected via Playwright's addInitScript() BEFORE the WASM loads.
// Routes __TAURI__.core.invoke() calls to the test HTTP server on port 3001.

// serde_wasm_bindgen serializes Rust HashMaps as JS Map objects.
// JSON.stringify ignores Map entries, so we convert them to plain objects first.
function mapsToObjects(val) {
  if (val instanceof Map) {
    const obj = {};
    val.forEach((v, k) => {
      obj[k] = mapsToObjects(v);
    });
    return obj;
  }
  if (Array.isArray(val)) return val.map(mapsToObjects);
  if (val !== null && typeof val === "object") {
    const obj = {};
    for (const k of Object.keys(val)) {
      obj[k] = mapsToObjects(val[k]);
    }
    return obj;
  }
  return val;
}

window.__TAURI__ = {
  core: {
    async invoke(cmd, args = {}) {
      const response = await fetch(`http://localhost:3001/invoke/${cmd}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(mapsToObjects(args)),
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
