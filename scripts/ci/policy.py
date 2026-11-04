#!/usr/bin/env python3
"""Dependency qualification and exact-tree CI reuse. Python standard library only."""

import argparse
import datetime as dt
import io
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib
from urllib.parse import quote, urlencode, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener, urlopen
import zipfile


WORKFLOW = ".github/workflows/ci.yml"
CRATES_INDEX = "registry+https://github.com/rust-lang/crates.io-index"
SHA = re.compile(r"[0-9a-f]{40}(?:[0-9a-f]{24})?\Z")
EXCLUDED_NPM_ROOTS = {"docs", ".agents", ".missions", "benchmarks"}


def git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root, stderr=subprocess.DEVNULL).decode().strip()


def timestamp(value):
    result = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    if result.tzinfo is None:
        raise ValueError("Publication time must include a timezone")
    return result


class SafeRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        if urlsplit(newurl).scheme != "https":
            raise ValueError("Refusing a non-HTTPS artifact redirect")
        result = super().redirect_request(req, fp, code, msg, headers, newurl)
        if urlsplit(req.full_url).netloc != urlsplit(newurl).netloc:
            result.remove_header("Authorization")
        return result


class GitHub:
    def __init__(self, token):
        self.token = token
        self.opener = build_opener(SafeRedirect())

    def request(self, route, limit=8 * 1024 * 1024):
        if not route.startswith("/repos/") or ".." in route:
            raise ValueError("Invalid GitHub API route")
        headers = {"Accept": "application/vnd.github+json", "User-Agent": "anti-slop-ci",
                   "X-GitHub-Api-Version": "2022-11-28"}
        if self.token:
            headers["Authorization"] = "Bearer " + self.token
        with self.opener.open(Request("https://api.github.com" + route, headers=headers), timeout=10) as response:
            data = response.read(limit + 1)
        if len(data) > limit:
            raise ValueError("GitHub response exceeds the lookup limit")
        return data

    def get(self, route):
        return json.loads(self.request(route))

    def pages(self, route, field=None):
        result = []
        separator = "&" if "?" in route else "?"
        for page in range(1, 6):
            response = self.get(f"{route}{separator}per_page=100&page={page}")
            items = response[field] if field else response
            if not isinstance(items, list):
                raise ValueError("Invalid GitHub list response")
            result.extend(items)
            if len(items) < 100:
                return result
        raise ValueError("Lookup pagination limit reached; full verification required")

    def download(self, route):
        return self.request(route, limit=1024 * 1024)


def read_receipt(data):
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        if archive.namelist() != ["receipt.json"]:
            raise ValueError("Unexpected receipt archive contents")
        entry = archive.getinfo("receipt.json")
        if entry.file_size > 16384:
            raise ValueError("Receipt is too large")
        result = json.loads(archive.read(entry))
    if not isinstance(result, dict):
        raise ValueError("Receipt must be an object")
    return result


def matches(receipt, run, artifact, pr, repository, repository_id, tree):
    expected = {
        "schemaVersion": 1, "repository": repository, "repositoryId": repository_id,
        "pullRequest": pr["number"], "headSha": pr["head"]["sha"], "treeSha": tree,
        "workflowPath": WORKFLOW, "runId": run["id"], "runAttempt": run["run_attempt"],
    }
    if any(type(receipt.get(key)) is not type(value) or receipt.get(key) != value
           for key, value in expected.items()):
        return False
    return (
        run.get("event") == "pull_request" and run.get("status") == "completed"
        and run.get("conclusion") == "success" and run.get("path") == WORKFLOW
        and run.get("head_sha") == pr["head"]["sha"]
        and run.get("repository", {}).get("id") == repository_id
        and run.get("repository", {}).get("full_name") == repository
        and artifact.get("workflow_run", {}).get("id") == run["id"]
        and (not run.get("pull_requests") or any(
            item.get("number") == pr["number"] for item in run["pull_requests"]))
    )


def reuse(api, repository, repository_id, commit, tree):
    fallback = {"run_full": True, "reason": "no_validated_pr_tree", "source": ""}
    try:
        prs = api.pages(f"/repos/{repository}/commits/{commit}/pulls")
        prs = [pr for pr in prs if pr.get("merged_at")
               and pr.get("merge_commit_sha") == commit and pr.get("base", {}).get("ref") == "main"]
        if not prs:
            return fallback
        name = "ci-receipt-" + tree
        artifacts = api.pages(f"/repos/{repository}/actions/artifacts?{urlencode({'name': name})}", "artifacts")
        for artifact in artifacts:
            if artifact.get("expired") is not False or artifact.get("name") != name:
                continue
            run_id = artifact["workflow_run"]["id"]
            if type(run_id) is not int or type(artifact.get("id")) is not int:
                continue
            run = api.get(f"/repos/{repository}/actions/runs/{run_id}")
            if run.get("conclusion") != "success" or run.get("status") != "completed":
                continue
            try:
                receipt = read_receipt(api.download(
                    f"/repos/{repository}/actions/artifacts/{artifact['id']}/zip"))
            except (ValueError, KeyError, zipfile.BadZipFile, UnicodeError):
                continue
            for pr in prs:
                if matches(receipt, run, artifact, pr, repository, repository_id, tree):
                    return {"run_full": False, "reason": "matched_pr_tree",
                            "source": f"https://github.com/{repository}/actions/runs/{run_id}"}
        return fallback
    except Exception as error:
        # An unavailable proof is never a successful check. The full suite is the fallback.
        print(f"CI reuse unavailable ({type(error).__name__}); running full verification.", file=sys.stderr)
        return fallback | {"reason": "proof_lookup_unavailable"}


def packages(path, text):
    if path == "Cargo.lock":
        result = []
        for item in tomllib.loads(text).get("package", []):
            source = item.get("source", "")
            if not source.startswith("registry+"):
                continue
            if source != CRATES_INDEX:
                raise ValueError("Only the qualified crates.io registry is supported")
            result.append({"ecosystem": "cargo", "name": item["name"], "version": item["version"],
                           "source": source, "checksum": item["checksum"]})
        return result
    lock = json.loads(text)
    if lock.get("lockfileVersion") not in (2, 3):
        raise ValueError("Expected npm lockfile format 2 or 3")
    result = []
    for location, item in lock["packages"].items():
        if not location or item.get("link"):
            continue
        if "node_modules/" not in location:
            continue  # Local workspace package, rather than an installed registry dependency.
        name = item.get("name") or location.rsplit("node_modules/", 1)[1]
        resolved = item.get("resolved", "")
        parsed = urlsplit(resolved)
        if parsed.scheme != "https" or parsed.netloc != "registry.npmjs.org":
            raise ValueError("Only the qualified public npm registry is supported")
        result.append({"ecosystem": "npm", "name": name, "version": item["version"],
                       "resolved": resolved, "integrity": item["integrity"]})
    return result


def changed_packages(root, base):
    paths = git(root, "ls-files", "-z", "--", "Cargo.lock", "**/package-lock.json", "package-lock.json").split("\0")
    # git ls-files omits new untracked files; include them for local pre-commit verification too.
    paths += git(root, "ls-files", "--others", "--exclude-standard", "-z", "--",
                 "Cargo.lock", "**/package-lock.json", "package-lock.json").split("\0")
    baseline = bool(base and SHA.fullmatch(base))
    if baseline:
        try:
            git(root, "cat-file", "-e", base + "^{commit}")
        except subprocess.CalledProcessError:
            baseline = False
    selected = {}
    for path in sorted(set(paths) - {""}):
        if path != "Cargo.lock" and Path(path).parts[0] in EXCLUDED_NPM_ROOTS:
            continue
        candidate = root / path
        if not candidate.is_file():
            continue  # Deleted lockfile has no candidate downloads to qualify.
        current = packages(path, candidate.read_text())
        old = []
        if baseline:
            try:
                old = packages(path, git(root, "show", f"{base}:{path}"))
            except (subprocess.CalledProcessError, ValueError, KeyError):
                pass  # Missing/unreadable comparison lock requires the complete current graph.
        identities = {json.dumps(p, sort_keys=True) for p in old}
        for item in current:
            key = json.dumps(item, sort_keys=True)
            if key not in identities:
                selected[key] = item
    return list(selected.values())


def verify_package(package, metadata, now):
    try:
        version = package["version"]
        if package["ecosystem"] == "cargo":
            record = next(v for v in metadata["versions"] if v["num"] == version)
            eligible = record.get("yanked") is False and record["checksum"] == package["checksum"]
            published = timestamp(record["created_at"])
        else:
            record = metadata["versions"][version]["dist"]
            eligible = record["integrity"] == package["integrity"] and record["tarball"] == package["resolved"]
            published = timestamp(metadata["time"][version])
        if not eligible or published > now - dt.timedelta(days=7):
            raise ValueError("Release age, yank status or locked integrity did not qualify")
    except (KeyError, StopIteration, TypeError) as error:
        raise ValueError("Official release metadata is missing or invalid") from error


def qualify(root, base, fetch=None, now=None):
    now = now or dt.datetime.now(dt.timezone.utc)
    def default_fetch(url):
        with urlopen(Request(url, headers={"User-Agent": "Anti-Slop-CI/1"}), timeout=30) as response:
            return json.load(response)
    fetch = fetch or default_fetch
    selected = changed_packages(root, base)
    metadata = {}
    for package in selected:
        name = package["name"]
        key = (package["ecosystem"], name)
        if key not in metadata:
            url = ("https://crates.io/api/v1/crates/" if key[0] == "cargo"
                   else "https://registry.npmjs.org/") + quote(name, safe="")
            metadata[key] = fetch(url)
        verify_package(package, metadata[key], now)
        print(f"Qualified {key[0]} {name}@{package['version']}")
    print(f"Qualified {len(selected)} added/changed locked registry packages.")


def event_context():
    repository = os.environ["GITHUB_REPOSITORY"]
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Invalid repository identity")
    return repository, int(os.environ["GITHUB_REPOSITORY_ID"]), json.loads(
        Path(os.environ["GITHUB_EVENT_PATH"]).read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("reuse", "receipt", "dependencies", "whitespace"))
    parser.add_argument("--base", default="")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    root = Path(git(Path.cwd(), "rev-parse", "--show-toplevel"))
    if args.command == "dependencies":
        qualify(root, args.base)
    elif args.command == "whitespace":
        base = args.base
        try:
            if not SHA.fullmatch(base):
                raise ValueError("No comparison history")
            git(root, "cat-file", "-e", base + "^{commit}")
        except (ValueError, subprocess.CalledProcessError):
            # A complete tree check when the previous revision is unavailable.
            base = git(root, "hash-object", "-w", "-t", "tree", "/dev/null")
        subprocess.run(["git", "diff", "--check", base], cwd=root, check=True)
    else:
        repository, repository_id, event = event_context()
        tree = git(root, "rev-parse", "HEAD^{tree}")
        if args.command == "receipt":
            if os.environ["GITHUB_EVENT_NAME"] != "pull_request":
                raise ValueError("Only PR verification publishes a reuse receipt")
            receipt = {"schemaVersion": 1, "repository": repository, "repositoryId": repository_id,
                       "pullRequest": event["number"], "headSha": event["pull_request"]["head"]["sha"],
                       "treeSha": tree, "workflowPath": WORKFLOW,
                       "runId": int(os.environ["GITHUB_RUN_ID"]), "runAttempt": int(os.environ["GITHUB_RUN_ATTEMPT"])}
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(receipt, indent=2) + "\n")
        else:
            decision = {"run_full": True, "reason": "pr_or_manual_verification", "source": ""}
            if os.environ["GITHUB_EVENT_NAME"] == "push" and os.environ.get("GITHUB_REF") == "refs/heads/main":
                decision = reuse(GitHub(os.environ.get("GITHUB_TOKEN", "")), repository, repository_id,
                                 git(root, "rev-parse", "HEAD"), tree)
            print(json.dumps(decision))
            with Path(os.environ["GITHUB_OUTPUT"]).open("a") as output:
                output.write(f"run_full={str(decision['run_full']).lower()}\n")
                output.write(f"tree={tree}\n")
            with Path(os.environ["GITHUB_STEP_SUMMARY"]).open("a") as summary:
                summary.write("## Linux CI\n\n")
                summary.write("Full verification required.\n" if decision["run_full"] else
                              f"Reused successful PR verification: {decision['source']}\n\n"
                              f"Exact tested tree: `{tree}`. Setup, downloads, build and tests skipped.\n")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"CI policy failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
