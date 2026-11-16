import { readFile } from "node:fs/promises";
import { validateArtifact, type ValidationReport } from "./validate.js";

const args = process.argv.slice(2);
const usage = "Usage: node scripts/validate.mjs <artifact-path> [--json] [--candidate <state-and-journal.json>]";
const json = args.includes("--json");
const candidateIndex = args.indexOf("--candidate");
const candidatePath = candidateIndex >= 0 ? args[candidateIndex + 1] : undefined;
const flags = candidateIndex >= 0 ? args.filter((_, index) => index !== candidateIndex && index !== candidateIndex + 1) : args;
const paths = flags.filter((arg) => !arg.startsWith("--"));

function print(report: ValidationReport): void {
  if (json)
    process.stdout.write(`${JSON.stringify({ valid: report.valid, issues: report.issues })}\n`);
  else if (report.valid)
    process.stdout.write(
      "Structurally valid; authorization and semantic acceptance are not verified.\n",
    );
  else
    process.stdout.write(
      `${report.issues.map((i) => `${i.path}: ${i.rule}: ${i.message}`).join("\n")}\n`,
    );
  process.exitCode = report.exitCode;
}

if (args.length === 1 && args[0] === "--help") {
  process.stdout.write(`${usage}\n`);
} else if (
  paths.length !== 1 ||
  flags.some((arg) => arg.startsWith("--") && arg !== "--json") ||
  (candidateIndex >= 0 && (!candidatePath || candidatePath.startsWith("--"))) ||
  args.filter((arg) => arg === "--candidate").length > 1 ||
  args.filter((arg) => arg === "--json").length > 1
) {
  print({
    valid: false,
    issues: [{ path: "<arguments>", rule: "usage", message: usage }],
    exitCode: 2,
  });
} else {
  if (candidatePath) {
    let text: string | undefined;
    try { text = await readFile(candidatePath, "utf8"); }
    catch { print({ valid: false, exitCode: 2, issues: [{ path: candidatePath, rule: "read", message: "Cannot read candidate JSON." }] }); }
    if (text !== undefined) {
      let candidate: unknown;
      let parsed = false;
      try { candidate = JSON.parse(text); parsed = true; }
      catch { print({ valid: false, exitCode: 1, issues: [{ path: candidatePath, rule: "parse", message: "Invalid candidate JSON." }] }); }
      if (parsed) print(await validateArtifact(paths[0], candidate));
    }
  } else print(await validateArtifact(paths[0]));
}
