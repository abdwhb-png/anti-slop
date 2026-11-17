set -u
REPO=/home/abdwhb/engineering/firstmate-orchestration-test/data/as-m004-durable/worktree
TMP=$(mktemp -d /tmp/opencode/p05/negctl.XXXXXX)
cd "$TMP"
git init -q
git config user.email fixture@example.invalid
git config user.name Fixture
git config core.attributesFile /dev/null
mkdir -p .missions/AS-M003/evidence .missions/AS-M004/evidence/P01 .missions/AS-M004/evidence/P03 .missions/AS-M004/evidence/P04 .missions/AS-M004/evidence/P05 src
cp "$REPO/.gitattributes" .gitattributes
cp "$REPO/.missions/AS-M003/evidence/.gitattributes" .missions/AS-M003/evidence/.gitattributes
git add -A
git commit -qm "baseline: real attribute files only"
BASE=$(git rev-parse HEAD)

echo "temporary repository: $TMP"
echo "baseline commit: $BASE"
echo "core.attributesFile: $(git config --get core.attributesFile)"
echo "core.whitespace: $(git config --get core.whitespace || echo '<unset: git default>')"
echo "attribute files copied as configuration (no policy logic copied):"
sha256sum .gitattributes .missions/AS-M003/evidence/.gitattributes
echo

run() {
  label=$1
  expect=$2
  out=$(python3 "$REPO/scripts/ci/policy.py" whitespace --base "$BASE" 2>&1)
  rc=$?
  echo "--- $label"
  echo "policy exit=$rc expected=$expect"
  printf '%s\n' "$out" | cat -A
  echo
}

clear_path() {
  git rm -q --cached --ignore-unmatch -- "$1" >/dev/null 2>&1
  rm -f -- "$1"
}

echo "=== case 1: scoped raw archive is permitted and byte-identical"
cp "$REPO/.missions/AS-M004/evidence/P03/tracked-source.diff" .missions/AS-M004/evidence/P03/tracked-source.diff
before=$(sha256sum .missions/AS-M004/evidence/P03/tracked-source.diff | cut -d' ' -f1)
git add -- .missions/AS-M004/evidence/P03/tracked-source.diff
run "case 1 scoped raw archive" 0
git commit -qm "case 1 scoped raw archive"
after_commit=$(git show HEAD:.missions/AS-M004/evidence/P03/tracked-source.diff | sha256sum | cut -d' ' -f1)
repo_copy=$(sha256sum "$REPO/.missions/AS-M004/evidence/P03/tracked-source.diff" | cut -d' ' -f1)
echo "sha256 working=$before committed=$after_commit repository=$repo_copy"
echo

echo "=== case 2: new source file with trailing space is rejected"
printf 'let value = 1; \n' > src/new_source.rs
echo "fixture bytes (cat -A, bytes=$(stat -c %s src/new_source.rs)):"; cat -A src/new_source.rs
git add -- src/new_source.rs
run "case 2 new source" 1
clear_path src/new_source.rs

echo "=== case 3: authored mission Markdown report with trailing space and blank EOF is rejected"
printf '# Authored report \n\n' > .missions/AS-M004/evidence/P05/authored-report.md
echo "fixture bytes (cat -A, bytes=$(stat -c %s .missions/AS-M004/evidence/P05/authored-report.md)):"; cat -A .missions/AS-M004/evidence/P05/authored-report.md
git add -- .missions/AS-M004/evidence/P05/authored-report.md
run "case 3 authored report" 1
clear_path .missions/AS-M004/evidence/P05/authored-report.md

echo "=== case 4: unlisted AS-M004 raw file is rejected"
printf 'diff --git a/a b/a\n \n' > .missions/AS-M004/evidence/P01/preparation.diff
echo "fixture bytes (cat -A, bytes=$(stat -c %s .missions/AS-M004/evidence/P01/preparation.diff)):"; cat -A .missions/AS-M004/evidence/P01/preparation.diff
git add -- .missions/AS-M004/evidence/P01/preparation.diff
run "case 4 unlisted raw file" 1
clear_path .missions/AS-M004/evidence/P01/preparation.diff

echo "=== case 5: exempted frozen conception file still rejects blank-at-eol"
printf 'conception line \n' > .missions/AS-M004/evidence/conception-accepted.md
echo "fixture bytes (cat -A, bytes=$(stat -c %s .missions/AS-M004/evidence/conception-accepted.md)):"; cat -A .missions/AS-M004/evidence/conception-accepted.md
git add -- .missions/AS-M004/evidence/conception-accepted.md
run "case 5 class narrowness" 1
clear_path .missions/AS-M004/evidence/conception-accepted.md

echo "=== case 6: two-class raw entry permits both observed classes"
printf 'measure \n\n' > .missions/AS-M004/evidence/P04/measurement-source.diff.txt
echo "fixture bytes (cat -A, bytes=$(stat -c %s .missions/AS-M004/evidence/P04/measurement-source.diff.txt)):"; cat -A .missions/AS-M004/evidence/P04/measurement-source.diff.txt
git add -- .missions/AS-M004/evidence/P04/measurement-source.diff.txt
run "case 6 two-class raw file" 0
clear_path .missions/AS-M004/evidence/P04/measurement-source.diff.txt

echo "=== final state"
git status --porcelain=v1 --untracked-files=all
echo "(empty status means every fixture was cleared)"
