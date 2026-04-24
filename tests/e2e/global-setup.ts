import { execSync, spawn, ChildProcess } from "child_process";
import { existsSync, writeFileSync, readFileSync } from "fs";
import path from "path";

const ROOT = path.resolve(__dirname, "../..");
const PID_FILE = path.resolve(__dirname, ".e2e-pids.json");

async function waitForServer(
  url: string,
  name: string,
  timeoutMs = 60_000
): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url);
      if (res.ok || res.status < 500) return;
    } catch {
      // not ready yet
    }
    await new Promise((r) => setTimeout(r, 1000));
  }
  throw new Error(`${name} did not become ready within ${timeoutMs}ms`);
}

export default async function globalSetup() {
  console.log("[e2e] Starting Docker Postgres...");
  execSync("just test-setup", { cwd: ROOT, stdio: "inherit" });

  console.log("[e2e] Building test HTTP server...");
  execSync("cargo build --manifest-path tests/test_server/Cargo.toml", {
    cwd: ROOT,
    stdio: "inherit",
  });

  console.log("[e2e] Starting test HTTP server on port 3001...");
  const testServer = spawn(
    "cargo",
    ["run", "--manifest-path", "tests/test_server/Cargo.toml"],
    {
      cwd: ROOT,
      stdio: "pipe",
      detached: true,
    }
  );
  testServer.unref();

  console.log("[e2e] Starting Trunk dev server on port 8080...");
  const trunkServer = spawn("trunk", ["serve", "--port", "8080"], {
    cwd: ROOT,
    stdio: "pipe",
    detached: true,
  });
  trunkServer.unref();

  // Save PIDs for teardown
  const pids = {
    testServer: testServer.pid,
    trunkServer: trunkServer.pid,
  };
  writeFileSync(PID_FILE, JSON.stringify(pids));

  console.log("[e2e] Waiting for test server...");
  await waitForServer(
    "http://localhost:3001/invoke/load_settings",
    "Test HTTP server"
  );

  console.log("[e2e] Waiting for Trunk dev server...");
  await waitForServer("http://localhost:8080", "Trunk dev server", 120_000);

  console.log("[e2e] All services ready.");
}
