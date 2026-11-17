"""Verify the bounded AS-M003 path repair from retained bytes, without Git or the old home."""

import hashlib
import json
from pathlib import Path
import tarfile


ROOT = Path(__file__).resolve().parents[5]
RECOVERY = Path(__file__).resolve().parents[1]


def read_json(path):
    return json.loads(path.read_bytes())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


originals = read_json(RECOVERY / 'originals-manifest.json')
for name, entry in originals.items():
    archived = RECOVERY / entry['archivePath']
    assert archived.stat().st_size == entry['bytes'] and digest(archived) == entry['sha256'], name
assert len(originals) == 14

mapping = read_json(RECOVERY / 'path-map.json')
assert len(mapping['references']) == mapping['referenceOccurrences'] == 48
assert len(mapping['artifacts']) == mapping['affectedArtifacts'] == 11
assert len({entry['retainedTarget'] for entry in mapping['references']}) == mapping['requiredTargets'] == 40
repaired_prefix = None
for name, entry in mapping['artifacts'].items():
    content = (RECOVERY / originals[name]['archivePath']).read_bytes()
    assert hashlib.sha256(content).hexdigest() == entry['sha256']
    for change in mapping['references']:
        if change['artifact'] == name:
            before, after = json.dumps(change['before']).encode(), json.dumps(change['after']).encode()
            assert content.count(before) == 1
            content = content.replace(before, after, 1)
            target = ROOT / '.missions/AS-M003/evidence/recovery/retained' / change['retainedTarget']
            assert (ROOT / name).parent.joinpath(change['after']).resolve() == target
            assert target.is_file() and target.resolve().is_relative_to(ROOT)
    assert hashlib.sha256(content).hexdigest() == entry['repairedSha256']
    if name == '.missions/AS-M003/history.jsonl':
        repaired_prefix = content
        assert (ROOT / name).read_bytes().startswith(content)
    else:
        assert content == (ROOT / name).read_bytes(), name + ': change beyond authorized path strings'
assert repaired_prefix is not None and len(repaired_prefix.splitlines()) == 20

retained = read_json(RECOVERY / 'retained-manifest.json')
for name, entry in retained.items():
    path = RECOVERY / 'retained' / name
    assert path.stat().st_size == entry['bytes'] and digest(path) == entry['sha256'], name
assert len(retained) == 118 and sum(entry['role'] == 'required target' for entry in retained.values()) == 40
for name, entry in read_json(RECOVERY / 'parent-evidence-manifest.json').items():
    assert digest(RECOVERY / name) == entry['sha256']

changed = set(mapping['artifacts']) | {'.missions/AS-M003/state.json', 'README.md', 'docs/IMPLEMENTATION_ROADMAP.md'}
baseline = read_json(RECOVERY / 'checks/pre-repair-tracked-hashes.json')
for name, expected in baseline.items():
    if name not in changed:
        assert digest(ROOT / name) == expected, name + ': unrelated tracked file changed'
for path in RECOVERY.rglob('*'):
    assert not path.is_symlink() and path.resolve().is_relative_to(RECOVERY)

archive_checks = [
    ('coordinator/P01-C01/accepted-source.tar.gz', '90b38d8702f9a5618fec834f317643a3bc4e62c6bec8b5e97ac91b1564e398ea'),
    ('coordinator/final-r1/rejected-source.tar.gz', 'fd6add9533bcecd4daf5f22eccf059c4aa8ec8ab12efb6fe6e90afbc7c0d0646'),
]
for name, expected in archive_checks:
    assert digest(RECOVERY / 'retained' / name) == expected
rejected = read_json(RECOVERY / 'retained/coordinator/final-r1/rejected-source-archive.json')
with tarfile.open(RECOVERY / 'retained/coordinator/final-r1/rejected-source.tar.gz', 'r:gz') as archive:
    members = {entry.name.removeprefix('./'): entry for entry in archive.getmembers() if entry.isfile()}
    for entry in rejected['sourceFiles']:
        assert hashlib.sha256(archive.extractfile(members[entry['path']]).read()).hexdigest() == entry['sha256'], entry['path']

ledger = read_json(ROOT / '.missions/AS-M003/evidence/P04/02-validation-ledger.json')
old_prefix = '/home/abdwhb/engineering/firstmate-anti-slop-test/data/AS-M003-capture/'
log_checks = 0
for entry in ledger['commandsAndAttempts']:
    for path_key, hash_key in [('record', 'recordSha256'), ('log', 'logSha256')]:
        if entry.get(path_key):
            assert entry[path_key].startswith(old_prefix)
            assert digest(RECOVERY / 'retained' / entry[path_key].removeprefix(old_prefix)) == entry[hash_key]
            log_checks += 1
inspection = read_json(RECOVERY / 'retained/final-review-5823ce78/evidence-inspection.json')
assert digest(RECOVERY / 'retained/p04-156023ed/10-stable-ci.log') == inspection['stableCI']['sha256']
for entry in read_json(RECOVERY / 'retained/final-review-5823ce78/focused-checks.json'):
    assert entry['log'].startswith(old_prefix)
    assert digest(RECOVERY / 'retained' / entry['log'].removeprefix(old_prefix)) == entry['sha256']
    log_checks += 1

applicability = read_json(RECOVERY / 'source-applicability.json')
for name, expected in applicability['acceptedSourceHashes'].items():
    path = RECOVERY / 'originals/README.md' if name == 'README.md' else ROOT / name
    assert digest(path) == expected
original_acceptance = read_json(RECOVERY / 'originals/.missions/AS-M003/results/final-r3.json')
assert original_acceptance['outcome'] == 'accepted' and original_acceptance['freshContext'] is True
assert original_acceptance['actor'] not in original_acceptance['implementers']

candidate = read_json(RECOVERY / 'closure-candidate.json')
selection = read_json(RECOVERY / 'activation/selection.json')
assert digest(RECOVERY / 'closure-candidate.json') == selection['originalProposedCandidateSha256']
selected = read_json(RECOVERY / 'closure-selected-candidate.json')
assert digest(RECOVERY / 'closure-selected-candidate.json') == selection['selectedCandidateSha256']
expected_state = json.loads(json.dumps(candidate['state']))
review_refs = [
    {'kind': 'evidence', 'path': 'evidence/recovery/principal-review/firstmate-repair-review.json'},
    {'kind': 'evidence', 'path': 'evidence/recovery/principal-review/evidence/parent-repair-verification.json'},
]
expected_state['evidence'].extend(review_refs)
assert selected['state'] == expected_state, 'State changes exceed the selected closure and principal-review references'
assert selected['journal'].encode().startswith(repaired_prefix)
events = [json.loads(line) for line in selected['journal'].splitlines()]
assert len(events) == 21
last = events[-1]
assert last == selection['event']
assert last['sequence'] == last['stateRevision'] == 21 and last['status'] == 'completed'
assert last['mission'] == expected_state['mission'] and last['interruption'] is None
assert all(ref in last['evidence'] for ref in review_refs)
review = read_json(RECOVERY / 'principal-review/firstmate-repair-review.json')
assert review['outcome'] == 'accepted' and review['activationAuthorized'] is True
assert review['commitAuthorized'] is False and review['publicationAuthorized'] is False
for name, entry in read_json(RECOVERY / 'activation/principal-review-manifest.json')['files'].items():
    assert digest(RECOVERY / 'principal-review' / name) == entry['sha256']
preservation = read_json(RECOVERY / 'activation/preservation-manifest.json')
for name, expected in preservation['reviewedFiles'].items():
    if name not in preservation['authorizedModifiedPaths']:
        assert digest(ROOT / name) == expected, name + ': reviewed evidence changed'

state = read_json(ROOT / '.missions/AS-M003/state.json')
journal = (ROOT / '.missions/AS-M003/history.jsonl').read_bytes()
if state['status'] == 'validating':
    assert (ROOT / '.missions/AS-M003/state.json').read_bytes() == (RECOVERY / originals['.missions/AS-M003/state.json']['archivePath']).read_bytes()
    assert journal == repaired_prefix
else:
    assert state == selected['state']
    assert (ROOT / '.missions/AS-M003/state.json').read_bytes() == (json.dumps(selected['state'], indent=2) + '\n').encode()
    assert journal == selected['journal'].encode()
assert candidate['state']['validation'] == state['validation']
assert selected['state']['integration']['head'] == '1560c240d30711865bb01e039bad03c10f865662'
assert selected['state']['integration']['target'] == 'main'

print(json.dumps({'valid': True, 'originalFiles': 14, 'historicalEvents': 20, 'pathOnlyArtifacts': 11,
                  'pathReplacements': 48, 'requiredTargets': 40, 'retainedFiles': 118,
                  'P04RecordAndReviewLogHashes': log_checks + 1, 'acceptedSourceHashes': 8,
                  'activeState': state['status'] + ' revision ' + str(state['revision']),
                  'selectedCompletedEvent': 21, 'initialProposedCandidate': 'preserved unchanged',
                  'reviewedEvidenceFilesUnchanged': len(preservation['reviewedFiles']) - len(preservation['authorizedModifiedPaths'])}))
