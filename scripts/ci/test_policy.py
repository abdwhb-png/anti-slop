"""Regression tests for CI's public decision and dependency-policy boundaries."""

import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import policy


REPO = "example/project"
TREE = "a" * 40
HEAD = "b" * 40
COMMIT = "c" * 40
NOW = "2026-11-04T12:00:00+00:00"


def proof():
    return {
        "schemaVersion": 1, "repository": REPO, "repositoryId": 5,
        "pullRequest": 7, "headSha": HEAD, "treeSha": TREE,
        "workflowPath": policy.WORKFLOW, "runId": 10, "runAttempt": 2,
    }


def archive(value):
    result = io.BytesIO()
    with zipfile.ZipFile(result, "w") as out:
        out.writestr("receipt.json", json.dumps(value))
    return result.getvalue()


class GitHub:
    """External API fixture; the production decision logic is never copied."""

    def __init__(self):
        self.pr = {"number": 7, "merged_at": NOW, "merge_commit_sha": COMMIT,
                   "head": {"sha": HEAD}, "base": {"ref": "main"}}
        self.artifact = {"id": 11, "name": "ci-receipt-" + TREE,
                         "expired": False, "workflow_run": {"id": 10}}
        self.run = {"id": 10, "run_attempt": 2, "event": "pull_request",
                    "status": "completed", "conclusion": "success",
                    "path": policy.WORKFLOW, "head_sha": HEAD,
                    "repository": {"id": 5, "full_name": REPO},
                    "pull_requests": [{"number": 7}]}
        self.receipt = proof()
        self.artifacts = [self.artifact]

    def pages(self, route, field=None):
        if "/commits/" in route:
            return [self.pr]
        if "/artifacts?" in route:
            return self.artifacts
        raise AssertionError(route)

    def get(self, route):
        if route.endswith("/runs/10"):
            return self.run
        raise AssertionError(route)

    def download(self, route):
        return archive(self.receipt)


class ReuseTests(unittest.TestCase):
    def test_merge_squash_and_rebase_reuse_identical_tested_trees(self):
        for method in ("merge", "squash", "rebase"):
            with self.subTest(method=method), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                def git(*args):
                    return subprocess.check_output(["git", *args], cwd=root,
                                                   stderr=subprocess.DEVNULL).decode().strip()
                git("init", "-q", "-b", "main")
                git("config", "user.email", "fixture@example.invalid")
                git("config", "user.name", "Fixture")
                (root / "base").write_text("base\n")
                git("add", ".")
                git("commit", "-qm", "base")
                git("checkout", "-qb", "feature")
                (root / "feature").write_text("feature\n")
                git("add", ".")
                git("commit", "-qm", "feature")
                original_head = git("rev-parse", "HEAD")
                git("checkout", "-q", "main")
                (root / "main-update").write_text("main update\n")
                git("add", ".")
                git("commit", "-qm", "update main")
                git("merge", "--no-commit", "--no-ff", "feature")
                tested_tree = git("write-tree")
                git("merge", "--abort")
                if method == "merge":
                    git("merge", "--no-ff", "-m", "merge PR", "feature")
                elif method == "squash":
                    git("merge", "--squash", "feature")
                    git("commit", "-qm", "squash PR")
                else:
                    git("checkout", "-q", "feature")
                    git("rebase", "main")
                    git("checkout", "-q", "main")
                    git("merge", "--ff-only", "feature")
                merged_commit = git("rev-parse", "HEAD")
                merged_tree = git("rev-parse", "HEAD^{tree}")
                self.assertEqual(tested_tree, merged_tree)
                api = GitHub()
                api.pr.update(merge_commit_sha=merged_commit, head={"sha": original_head})
                api.run["head_sha"] = original_head
                api.artifact["name"] = "ci-receipt-" + tested_tree
                api.receipt.update(headSha=original_head, treeSha=tested_tree)
                decision = policy.reuse(api, REPO, 5, merged_commit, merged_tree)
                self.assertFalse(decision["run_full"])
                self.assertIn("/actions/runs/10", decision["source"])

    def test_direct_push_and_non_tip_pr_associations_run_fully(self):
        for mutation in ({"merged_at": None}, {"merge_commit_sha": "d" * 40},
                         {"base": {"ref": "other"}}):
            api = GitHub()
            api.pr.update(mutation)
            self.assertTrue(policy.reuse(api, REPO, 5, COMMIT, TREE)["run_full"])

    def test_receipt_mismatches_cannot_claim_success(self):
        for key, value in (("treeSha", "d" * 40), ("headSha", "e" * 40),
                           ("repositoryId", 6), ("repository", "other/project"),
                           ("pullRequest", 8), ("runId", 12), ("runAttempt", 1),
                           ("workflowPath", ".github/workflows/unrelated.yml"),
                           ("schemaVersion", 2)):
            with self.subTest(key=key):
                api = GitHub()
                api.receipt[key] = value
                self.assertTrue(policy.reuse(api, REPO, 5, COMMIT, TREE)["run_full"])

    def test_server_metadata_must_confirm_successful_current_pr_attempt(self):
        for mutation in ({"conclusion": "failure"}, {"conclusion": "cancelled"},
                         {"status": "in_progress"}, {"event": "push"},
                         {"head_sha": "d" * 40}, {"run_attempt": 3},
                         {"path": ".github/workflows/other.yml"},
                         {"repository": {"id": 6, "full_name": REPO}},
                         {"pull_requests": [{"number": 8}]}):
            with self.subTest(mutation=mutation):
                api = GitHub()
                api.run.update(mutation)
                self.assertTrue(policy.reuse(api, REPO, 5, COMMIT, TREE)["run_full"])

    def test_missing_expired_or_malformed_proof_runs_fully(self):
        api = GitHub()
        api.artifacts = []
        self.assertTrue(policy.reuse(api, REPO, 5, COMMIT, TREE)["run_full"])
        api = GitHub()
        api.artifact["expired"] = True
        self.assertTrue(policy.reuse(api, REPO, 5, COMMIT, TREE)["run_full"])
        api = GitHub()
        api.receipt = []
        self.assertTrue(policy.reuse(api, REPO, 5, COMMIT, TREE)["run_full"])
        with patch.object(api, "download", return_value=b"not a ZIP"):
            self.assertTrue(policy.reuse(api, REPO, 5, COMMIT, TREE)["run_full"])

    def test_api_errors_fall_back_to_full_verification(self):
        api = GitHub()
        with patch.object(api, "pages", side_effect=OSError("unavailable")):
            self.assertTrue(policy.reuse(api, REPO, 5, COMMIT, TREE)["run_full"])

    def test_receipts_are_bounded_data_and_never_extracted(self):
        data = io.BytesIO()
        with zipfile.ZipFile(data, "w") as out:
            out.writestr("../receipt.json", json.dumps(proof()))
        with self.assertRaises(ValueError):
            policy.read_receipt(data.getvalue())
        with self.assertRaises(ValueError):
            policy.read_receipt(archive({"payload": "x" * 20000}))

    def test_cross_host_redirect_drops_authentication(self):
        from urllib.request import Request
        request = Request("https://api.github.com/artifact", headers={"Authorization": "Bearer fixture"})
        redirected = policy.SafeRedirect().redirect_request(
            request, None, 302, "Found", {}, "https://artifact.example/archive")
        self.assertFalse(any(k.lower() == "authorization" for k in redirected.headers))

    def test_cli_manual_dispatch_always_requests_full_verification(self):
        import os
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            (path / "event.json").write_text('{}')
            environment = os.environ | {
                "GITHUB_REPOSITORY": REPO, "GITHUB_REPOSITORY_ID": "5",
                "GITHUB_EVENT_NAME": "workflow_dispatch", "GITHUB_REF": "refs/heads/main",
                "GITHUB_EVENT_PATH": str(path / "event.json"),
                "GITHUB_OUTPUT": str(path / "output"), "GITHUB_STEP_SUMMARY": str(path / "summary"),
            }
            completed = subprocess.run(["python3", str(Path(policy.__file__).resolve()), "reuse"],
                                       env=environment, capture_output=True, text=True, check=True)
            self.assertTrue(json.loads(completed.stdout)["run_full"])
            self.assertIn("run_full=true", (path / "output").read_text())


class WhitespaceTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.git("init", "-q")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "user.name", "Fixture")
        self.git("config", "core.attributesFile", "/dev/null")
        self.git("config", "core.whitespace", "blank-at-eol,blank-at-eof,space-before-tab")
        candidate = Path(policy.__file__).resolve().parents[2]
        for name in (".gitattributes", ".missions/AS-M003/evidence/.gitattributes"):
            source = candidate / name
            target = self.root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(source.read_bytes() if source.exists() else b"")
        self.git("add", ".")
        self.git("commit", "-qm", "baseline attributes")
        self.base = self.git("rev-parse", "HEAD").decode().strip()

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, stderr=subprocess.DEVNULL)

    def whitespace(self):
        return subprocess.run(
            [sys.executable, str(Path(policy.__file__).resolve()), "whitespace", "--base", self.base],
            cwd=self.root, capture_output=True, text=True)

    def test_raw_evidence_is_publishable_without_rewriting_bytes(self):
        raw = {
            ".missions/AS-M003/evidence/P01/preparation.diff": b"diff --git a/a b/a\n \n context\n",
            ".missions/AS-M003/evidence/P02/07-hold-format-check.log": b"formatter output\n \ncontinued\n",
            ".missions/AS-M003/evidence/P03/03-capture-regressions.log": b"test result: ok\n\n",
            ".missions/AS-M003/evidence/P03/trial-staged-001/observation-staged-1.log": b"elapsed: 1\n\n",
        }
        for name, data in raw.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        self.git("add", ".")
        self.git("commit", "-qm", "byte-preserved raw evidence")
        result = self.whitespace()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for name, data in raw.items():
            with self.subTest(path=name):
                self.assertEqual((self.root / name).read_bytes(), data)
                self.assertEqual(self.git("show", "HEAD:" + name), data)
        self.assertEqual(self.git("status", "--porcelain"), b"")

    def test_maintained_files_and_unrelated_evidence_still_reject_whitespace(self):
        paths = (
            "src/lib.rs", "docs/README.md", "scripts/ci/example.py",
            ".missions/AS-M003/evidence/P03/09-final-audit.json",
            ".missions/AS-M003/evidence/P03/trial-staged-001/measurements-staged.json",
            ".missions/AS-M003/evidence/P04/unrelated.log",
            ".missions/AS-M003/evidence/P03/trial-staged-001/unrelated.log",
            ".missions/AS-M004/evidence/P01/preparation.diff",
            "docs/unrelated.diff",
        )
        for name in paths:
            with self.subTest(path=name):
                path = self.root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"maintained content \n\n")
                self.git("add", "--", name)
                result = self.whitespace()
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertIn(name + ":1: trailing whitespace.", result.stdout)
                self.assertIn(name + ":2: new blank line at EOF.", result.stdout)
                path.write_bytes(b"maintained content\n")
                self.git("add", "--", name)
        result = self.whitespace()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_raw_evidence_keeps_checks_for_other_whitespace_classes(self):
        cases = (
            (".missions/AS-M003/evidence/P01/preparation.diff", b"patch\n\n",
             "new blank line at EOF."),
            (".missions/AS-M003/evidence/P03/03-capture-regressions.log", b"result \n",
             "trailing whitespace."),
            (".missions/AS-M003/evidence/P02/07-hold-format-check.log", b" \tindent\n",
             "space before tab in indent."),
        )
        for name, data, diagnostic in cases:
            with self.subTest(path=name, diagnostic=diagnostic):
                path = self.root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
                self.git("add", "--", name)
                result = self.whitespace()
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertIn(name + ":", result.stdout)
                self.assertIn(diagnostic, result.stdout)
                path.write_bytes(b"raw output\n")
                self.git("add", "--", name)


class DependencyTests(unittest.TestCase):
    def setUp(self):
        self.now = policy.timestamp(NOW)
        self.cargo = {"ecosystem": "cargo", "name": "example", "version": "1.0.0",
                      "checksum": "a" * 64, "source": policy.CRATES_INDEX}
        self.npm = {"ecosystem": "npm", "name": "@scope/example", "version": "1.0.0",
                    "integrity": "sha512-fixture", "resolved": "https://registry.npmjs.org/@scope/example/-/example-1.0.0.tgz"}

    def test_eligible_versions_match_locked_integrity(self):
        cargo = {"versions": [{"num": "1.0.0", "created_at": "2026-10-01T00:00:00Z",
                               "yanked": False, "checksum": "a" * 64}]}
        npm = {"time": {"1.0.0": "2026-10-01T00:00:00Z"},
               "versions": {"1.0.0": {"dist": {"integrity": "sha512-fixture",
                         "tarball": self.npm["resolved"]}}}}
        policy.verify_package(self.cargo, cargo, self.now)
        policy.verify_package(self.npm, npm, self.now)

    def test_new_yanked_and_checksum_mismatched_cargo_versions_fail(self):
        version = {"num": "1.0.0", "created_at": "2026-10-01T00:00:00Z",
                   "yanked": False, "checksum": "a" * 64}
        for change in ({"created_at": "2026-11-03T00:00:00Z"},
                       {"checksum": "b" * 64}, {"yanked": True}):
            with self.assertRaises(ValueError):
                policy.verify_package(self.cargo, {"versions": [version | change]}, self.now)

    def test_unavailable_or_wrong_npm_integrity_fails(self):
        for metadata in ({}, {"time": {"1.0.0": "2026-10-01T00:00:00Z"},
                             "versions": {"1.0.0": {"dist": {"integrity": "sha512-wrong"}}}}):
            with self.assertRaises(ValueError):
                policy.verify_package(self.npm, metadata, self.now)

    def test_young_npm_version_and_changed_tarball_fail(self):
        metadata = {"time": {"1.0.0": "2026-11-03T00:00:00Z"},
                    "versions": {"1.0.0": {"dist": {"integrity": self.npm["integrity"],
                                                       "tarball": self.npm["resolved"]}}}}
        with self.assertRaises(ValueError):
            policy.verify_package(self.npm, metadata, self.now)
        metadata["time"]["1.0.0"] = "2026-10-01T00:00:00Z"
        metadata["versions"]["1.0.0"]["dist"]["tarball"] += "?changed"
        with self.assertRaises(ValueError):
            policy.verify_package(self.npm, metadata, self.now)

    def test_metadata_network_failure_fails_dependency_qualification(self):
        with patch.object(policy, "changed_packages", return_value=[self.cargo]):
            with self.assertRaises(OSError):
                policy.qualify(Path.cwd(), "", fetch=lambda _: (_ for _ in ()).throw(OSError("offline")))

    def test_exact_seven_day_boundary_is_eligible(self):
        metadata = {"versions": [{"num": "1.0.0", "created_at": "2026-10-28T12:00:00Z",
                                  "yanked": False, "checksum": "a" * 64}]}
        policy.verify_package(self.cargo, metadata, self.now)

    def test_lock_comparison_checks_added_changed_and_missing_history(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def git(*args):
                return subprocess.check_output(["git", *args], cwd=root, stderr=subprocess.DEVNULL).decode().strip()
            git("init", "-q")
            git("config", "user.email", "fixture@example.invalid")
            git("config", "user.name", "Fixture")
            lock = 'version = 4\n[[package]]\nname = "example"\nversion = "1.0.0"\nsource = "' + policy.CRATES_INDEX + '"\nchecksum = "' + "a" * 64 + '"\n'
            (root / "Cargo.lock").write_text(lock)
            git("add", ".")
            git("commit", "-qm", "baseline")
            base = git("rev-parse", "HEAD")
            self.assertEqual(policy.changed_packages(root, base), [])
            (root / "Cargo.lock").write_text(lock.replace('1.0.0', '1.1.0'))
            self.assertEqual(policy.changed_packages(root, base)[0]["version"], "1.1.0")
            self.assertEqual(len(policy.changed_packages(root, "f" * 40)), 1)


if __name__ == "__main__":
    unittest.main()
