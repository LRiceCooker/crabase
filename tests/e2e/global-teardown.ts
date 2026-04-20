import { execSync } from "child_process";
import { existsSync, readFileSync, unlinkSync } from "fs";
import path from "path";

const ROOT = path.resolve(__dirname, "../..");
const PID_FILE = path.resolve(__dirname, ".e2e-pids.json");

function killProcess(pid: number | undefined, name: string) {
  if (!pid) return;
  try {
    // Kill the process group (negative PID kills the group)
    process.kill(-pid, "SIGTERM");
    console.log(`[e2e] Stopped ${name} (pid ${pid})`);
  } catch {
    // Process may have already exited
    try {
      process.kill(pid, "SIGTERM");
      console.log(`[e2e] Stopped ${name} (pid ${pid})`);
    } catch {
      console.log(`[e2e] ${name} already stopped`);
    }
  }
}

export default async function globalTeardown() {
  // Kill spawned processes
  if (existsSync(PID_FILE)) {
    const pids = JSON.parse(readFileSync(PID_FILE, "utf-8"));
    killProcess(pids.trunkServer, "Trunk dev server");
    killProcess(pids.testServer, "Test HTTP server");
    unlinkSync(PID_FILE);
  }

  // Stop Docker Postgres
  console.log("[e2e] Stopping Docker Postgres...");
  try {
    execSync("just test-teardown", { cwd: ROOT, stdio: "inherit" });
  } catch {
    console.warn("[e2e] Warning: test-teardown failed (container may already be stopped)");
  }

  console.log("[e2e] Teardown complete.");
}
