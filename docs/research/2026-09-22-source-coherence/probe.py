"""Bounded source-view experiment; reuses the qualified transport and SDK worker.

No package installation, user repository analysis, or model requests. This is not
a production snapshot manager or an end-to-end Anti-Slop performance benchmark.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import time

sys.dont_write_bytecode = True


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--qualification-root', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    qualification = args.qualification_root.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    retained = Path(__file__).resolve().parents[1] / '2026-09-22-language-qualification'
    provenance = json.loads((retained / 'provenance.json').read_text())
    for name in ['rust_lsp_probe.py', 'measure.py']:
        if (qualification / name).read_bytes() != (retained / 'probes' / name).read_bytes():
            raise RuntimeError(f'Qualified probe differs: {name}')
    binaries = {
        'rust-analyzer': qualification / 'bin/rust-analyzer',
        'mago': qualification / 'bin/mago',
        'typescript7-native': Path(provenance['binaries_and_clients']['typescript7-native']['path']),
    }
    for name, path in binaries.items():
        if digest(path) != provenance['binaries_and_clients'][name]['sha256']:
            raise RuntimeError(f'Qualified binary differs: {name}')
    sys.path.insert(0, str(qualification))
    from rust_lsp_probe import Client, position
    sys.path.insert(0, str(retained / 'probes'))
    from generate_fixtures import generate

    live = output / 'live'
    generate(live, 0)
    managed = output / 'managed'
    managed.mkdir()
    facts_file = output / 'mago-facts.jsonl'
    php = shutil.which('php')
    if not php:
        raise RuntimeError('Existing PHP runtime unavailable')
    worker_command = [php, str(retained / 'probes/mago_worker.php'),
                      str(qualification / 'mago-source/composer/src'),
                      str(qualification / 'php-sdk/vendor/autoload.php'), str(facts_file)]
    config = live / 'php/mago.toml'
    config.write_text(config.read_text() + '\n[extension-hosts.qualification]\ncommand = '
                      + json.dumps(worker_command) + '\nworkers = 1\n')
    for language in ['ts', 'rust', 'php']:
        shutil.copytree(live / language, managed / language)

    checks = []
    observations = {}
    versions = {}

    def check(name, ok):
        checks.append({'name': name, 'passed': bool(ok)})

    def capture(name, root):
        # The declared scope deliberately excludes native build output/sysroots.
        files = {str(p.relative_to(root)): {'sha256': digest(p), 'text': p.read_text()}
                 for p in sorted(root.rglob('*')) if p.is_file()
                 and p.suffix in {'.ts', '.json', '.rs', '.toml', '.php'}
                 and 'target' not in p.relative_to(root).parts}
        versions[name] = files
        return {path: data['sha256'] for path, data in files.items()}

    for language in ['ts', 'rust']:
        root = managed / language
        rust = language == 'rust'
        relative_domain = 'domain/src/lib.rs' if rust else 'domain.ts'
        relative_consumer = 'app/src/lib.rs' if rust else 'service.ts'
        domain = root / relative_domain
        consumer = root / relative_consumer
        source = domain.read_text()
        updated = source.replace('pub struct OrderId(pub u64);', 'pub struct OrderId(pub u32);') if rust else source.replace('): OrderId {', '): number {').replace('return raw as OrderId;', 'return raw.length;')
        initial_text = 'size = 8' if rust else 'const result: OrderId'
        changed_text = 'size = 4' if rust else 'const result: number'
        obs = observations[language] = {}
        check(f'{language}: independent source files', not os.path.samefile(domain, live / language / relative_domain))
        initial_manifest = capture(f'{language}-generation-a', root)
        binary = binaries['rust-analyzer' if rust else 'typescript7-native']
        client = Client(binary, root, output / f'{language}-transport.json', [] if rust else ['--lsp', '--stdio'])
        try:
            configuration = {'checkOnSave': False, 'cargo': {'buildScripts': {'enable': False}, 'offline': True},
                             'procMacro': {'enable': False}, 'cachePriming': {'enable': False}, 'numThreads': 1} if rust else {}
            client.configuration = configuration
            params = {'processId': os.getpid(), 'rootUri': root.as_uri(), 'capabilities': {'experimental': {'serverStatusNotification': True}} if rust else {'textDocument': {'hover': {'contentFormat': ['plaintext']}}}, 'initializationOptions': configuration}
            obs['initialize'] = client.request('initialize', params)
            client.notification('initialized', {})
            if rust:
                obs['ready'] = client.ready()
            for file in [domain, consumer]:
                client.notification('textDocument/didOpen', {'textDocument': {'uri': file.as_uri(), 'languageId': 'rust' if rust else 'typescript', 'version': 1, 'text': file.read_text()}})
            query = {'textDocument': {'uri': consumer.as_uri()}, 'position': position(consumer.read_text(), 'id =' if rust else 'result =')}

            def hover(stage):
                result = client.request('textDocument/hover', query)
                obs[stage] = result
                return json.dumps(result['response'].get('result'))

            check(f'{language}: initial dependent fact', initial_text in hover('initial'))
            (live / language / relative_domain).write_text(updated)
            check(f'{language}: live edit cannot change captured file', domain.read_text() == source)
            check(f'{language}: live edit leaves captured analysis unchanged', initial_text in hover('after_live_edit'))

            # Deliberately change an in-memory input without changing its disk file.
            client.notification('textDocument/didChange', {'textDocument': {'uri': domain.as_uri(), 'version': 2}, 'contentChanges': [{'text': updated}]})
            changed_hover = hover('overlay_only')
            check(f'{language}: disk manifest can stay unchanged while semantic input changes',
                  capture(f'{language}-overlay-disk', root) == initial_manifest and changed_text in changed_hover)

            domain.write_text(updated)
            client.notification('textDocument/didChange', {'textDocument': {'uri': domain.as_uri(), 'version': 3}, 'contentChanges': [{'text': domain.read_text()}]})
            capture(f'{language}-generation-b', root)
            check(f'{language}: explicit new generation matches captured bytes', changed_text in hover('published_b') and domain.read_text() == updated)

            if not rust:
                ref_params = {'textDocument': {'uri': domain.as_uri()}, 'position': position(updated, 'normalize'), 'context': {'includeDeclaration': True}}
                obs['references_before'] = client.request('textDocument/references', ref_params)
                extra = root / 'new-caller.ts'
                extra.write_text('import { normalize } from "@domain";\nexport const additional = normalize("new");\n')
                client.notification('workspace/didChangeWatchedFiles', {'changes': [{'uri': extra.as_uri(), 'type': 1}]})
                client.notification('textDocument/didOpen', {'textDocument': {'uri': extra.as_uri(), 'languageId': 'typescript', 'version': 1, 'text': extra.read_text()}})
                obs['references_after'] = client.request('textDocument/references', ref_params)
                capture('ts-generation-c', root)
                before = obs['references_before']['response'].get('result') or []
                after = obs['references_after']['response'].get('result') or []
                check('ts: new caller invalidates prior reference result',
                      not any(r['uri'] == extra.as_uri() for r in before) and any(r['uri'] == extra.as_uri() for r in after))
            else:
                check('rust: build script remains disabled', not (root / 'domain/qualification-build-script-ran').exists())
        except Exception as error:
            obs['error'] = repr(error)
            check(f'{language}: experiment completes', False)
        finally:
            client.close()
            check(f'{language}: native process exits after shutdown', client.proc.poll() is not None)
        print(json.dumps({'completed': language, 'failed_checks': [c['name'] for c in checks if not c['passed']]}), flush=True)

    root = managed / 'php'
    domain = root / 'src/Repository.php'
    original = domain.read_text()
    updated = original.replace('): string;', '): int;')
    capture('php-generation-a', root)
    check('php: independent source files', not os.path.samefile(domain, live / 'php/src/Repository.php'))
    (live / 'php/src/Repository.php').write_text(updated)
    obs = observations['php'] = {}

    def run_mago(stage):
        started = time.perf_counter()
        command = [str(binaries['mago']), '--workspace', str(root), 'analyze', '--reporting-format', 'json']
        completed = subprocess.run(command, capture_output=True, text=True, timeout=30, cwd=root)
        (output / f'mago-{stage}.stdout').write_text(completed.stdout)
        (output / f'mago-{stage}.stderr').write_text(completed.stderr)
        obs[stage] = {'command': command, 'exit': completed.returncode, 'ms': (time.perf_counter() - started) * 1000}
        records = [json.loads(line) for line in facts_file.read_text().splitlines()]
        return records[-1], completed.returncode

    try:
        record, code = run_mago('a-after-live-edit')
        captured = [file for file in record['files'] if file['source'] == original]
        check('php: live edit leaves captured SDK input unchanged', code == 0 and len(captured) == 1 and domain.read_text() == original)
        domain.write_text(updated)
        capture('php-generation-b', root)
        record, code = run_mago('b')
        # The deliberate interface change creates real type diagnostics; retain them.
        check('php: explicit new generation reaches SDK', any(file['source'] == updated for file in record['files']))
        check('php: native type mismatch remains visible', code != 0 and record['issueCount'] > 0)
    except Exception as error:
        obs['error'] = repr(error)
        check('php: experiment completes', False)
    (output / 'captured-versions.json').write_text(json.dumps(versions, indent=2) + '\n')
    result = {'platform': platform.platform(), 'scope': 'small controlled source fixtures; no product performance claim',
              'binaries': {name: {'path': str(path), 'sha256': digest(path)} for name, path in binaries.items()},
              'checks': checks, 'observations': observations, 'passed': all(c['passed'] for c in checks)}
    (output / 'results.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'passed': result['passed'], 'checks': len(checks), 'output': str(output)}), flush=True)
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
