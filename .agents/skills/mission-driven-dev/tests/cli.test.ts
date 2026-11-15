import { afterEach, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const skill = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const cli = join(skill, "scripts/validate.mjs");
const roots: string[] = [];
function setup() {
  const root = mkdtempSync(join(tmpdir(), "mission-cli-test-"));
  roots.push(root);
  return root;
}
function invoke(args: string[], cwd: string) {
  return spawnSync("node", [cli, ...args], { cwd, encoding: "utf8", timeout: 5000 });
}
afterEach(() => {
  for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true });
});

test("delivered Node CLI validates from another cwd without modifying inputs or running embedded commands", () => {
  const root = setup();
  const artifact = join(root, "roadmap.md");
  const source = `---\n{"schemaVersion":1,"kind":"roadmap","id":"r","revision":1}\n---\n# Outcome\n\nRun \`touch '${join(root, "unexpected-file")}'\` later, not during validation.\n`;
  writeFileSync(artifact, source);
  const before = statSync(artifact).mtimeMs;
  const process = invoke([artifact, "--json"], tmpdir());
  expect(process.status).toBe(0);
  expect(JSON.parse(process.stdout)).toEqual({ valid: true, issues: [] });
  expect(process.stderr).toBe("");
  expect(readFileSync(artifact, "utf8")).toBe(source);
  expect(statSync(artifact).mtimeMs).toBe(before);
  expect(readdirSync(root)).toEqual(["roadmap.md"]);
});

test("CLI separates invalid artifacts, invocation errors, read failures and help", () => {
  const root = setup();
  const bad = join(root, "bad.json");
  writeFileSync(bad, "{invalid}");
  const invalid = invoke([bad, "--json"], root);
  expect(invalid.status).toBe(1);
  expect(JSON.parse(invalid.stdout).issues[0].rule).toBe("parse");
  for (const args of [["--json"], [bad, "--unknown", "--json"], [bad, bad, "--json"]]) {
    const usage = invoke(args, root);
    expect(usage.status).toBe(2);
    expect(JSON.parse(usage.stdout).issues[0].rule).toBe("usage");
  }
  expect(invoke([join(root, "absent.json"), "--json"], root).status).toBe(2);
  expect(invoke(["--help"], root).status).toBe(0);
  expect(invoke([bad], root).stdout).toContain("parse");
});

test("the delivered CLI checks a candidate transition without changing the active files", () => {
  const root = setup();
  const mission = { kind: "mission", id: "m", revision: 1, path: "mission.md" };
  writeFileSync(join(root, "mission.md"), `---\n${JSON.stringify({ schemaVersion: 1, ...mission, repository: ".", delivery: { mode: "local-only", baseCommit: "a".repeat(40), rationale: "One outcome." } })}\n---\n# Mission\n`);
  const state = { schemaVersion: 1, kind: "state", id: "m", revision: 1, mission, status: "draft", interruption: null, history: "history.jsonl", lastSequence: 1, currentPacket: null, attempts: [], checkpoints: [], evidence: [], validation: null };
  const event = { schemaVersion: 1, kind: "event", sequence: 1, stateRevision: 1, at: "2026-11-15T00:00:00Z", actor: "fixture", mission, status: "draft", interruption: null, reason: "Prepare.", evidence: [] };
  const history = JSON.stringify(event) + "\n";
  const file = join(root, "state.json"), journal = join(root, "history.jsonl"), candidate = join(root, "candidate.json");
  writeFileSync(file, JSON.stringify(state)); writeFileSync(journal, history);
  writeFileSync(candidate, JSON.stringify({ state: { ...state, revision: 2, lastSequence: 2, status: "ready", executionMode: "coordinated" }, journal: history + JSON.stringify({ ...event, sequence: 2, stateRevision: 2, status: "ready" }) + "\n" }));
  const result = invoke([file, "--candidate", candidate, "--json"], tmpdir());
  expect(result.status).toBe(0);
  expect(JSON.parse(result.stdout)).toEqual({ valid: true, issues: [] });
  expect(readFileSync(file, "utf8")).toBe(JSON.stringify(state));
  expect(readFileSync(journal, "utf8")).toBe(history);
});

test("candidate diagnostics distinguish malformed JSON from unavailable input", () => {
  const root = setup(), artifact = join(root, "artifact.json"), candidate = join(root, "candidate.json");
  writeFileSync(artifact, "{}"); writeFileSync(candidate, "{bad");
  const invalid = invoke([artifact, "--candidate", candidate, "--json"], root);
  expect(invalid.status).toBe(1);
  expect(JSON.parse(invalid.stdout).issues[0].rule).toBe("parse");
  const missing = invoke([artifact, "--candidate", join(root, "absent"), "--json"], root);
  expect(missing.status).toBe(2);
  expect(JSON.parse(missing.stdout).issues[0].rule).toBe("read");
});
