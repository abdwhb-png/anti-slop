"""Recompute the published statistics from the archived final observations."""
from pathlib import Path
import json
import math
import statistics
import sys

root = Path(sys.argv[1]).resolve()
results = root / 'measurements'


def read(path):
    return json.loads(path.read_text())


def distribution(values):
    ordered = sorted(values)
    return {
        'n': len(ordered),
        'p50_ms': statistics.median(ordered),
        'p95_ms': ordered[math.ceil(.95 * len(ordered)) - 1],
        'min_ms': ordered[0],
        'max_ms': ordered[-1],
    }


summary = {}
for size in ['small', 'medium']:
    group = {}
    for name in ['oxc-final', 'mago-native', 'mago-sdk']:
        runs = [read(results / f'{size}-{name}-{i:02}.measurement.json') for i in range(10)]
        group[name] = {
            'process_wall': distribution([r['wall_ms'] for r in runs]),
            'max_process_rss_mib': max(r['max_process_rss_kib'] for r in runs) / 1024,
            'sampled_peak_tree_rss_mib': max(r['sampled_tree_rss_kib'] for r in runs) / 1024,
            'output_bytes': max(r['stdout_bytes'] for r in runs),
        }
    for name in ['rust', 'typescript7-lsp']:
        runs = [read(results / f'{size}-{name}-final-{i:02}.json') for i in range(3)]
        group[name] = {
            'fresh_process_facts': distribution([r['facts_ready_ms'] for r in runs]),
            'unchanged': distribution([u['ms'] for r in runs for u in r['unchanged']]),
            'updates': distribution([u['ms'] for r in runs for u in r['updates']]),
            'sampled_peak_tree_rss_mib': max(r['sampled_peak_tree_rss_kib'] for r in runs) / 1024,
            'retained_rss_mib': max(r['retained_rss_kib'] for r in runs) / 1024,
        }
    api = [read(results / f'{size}-typescript7-api-final-{i:02}.stdout') for i in range(3)]
    measures = [read(results / f'{size}-typescript7-api-final-{i:02}.measurement.json') for i in range(3)]
    group['typescript7-api'] = {
        'first_type': distribution([r['initial']['ms'] for r in api]),
        'updates': distribution([u['ms'] for r in api for u in r['updates']]),
        'sampled_peak_tree_rss_mib': max(r['sampled_tree_rss_kib'] for r in measures) / 1024,
    }
    ts6 = [read(results / f'{size}-typescript-{i:02}.stdout') for i in range(3)]
    group['typescript6-reference'] = {
        'first_type': distribution([r['initial_ms'] for r in ts6]),
        'facts_and_diagnostics': distribution([r['initial_work_ms'] for r in ts6]),
        'updates': distribution([u['ms'] for r in ts6 for u in r['updates']]),
        'retained_rss_mib': max(r['final_rss'] for r in ts6) / (1024 * 1024),
    }
    watch = read(results / f'{size}-mago-watch.json')
    group['mago-watch'] = {
        'initial_ms': watch['initial_ms'],
        'updates': distribution([u['ms'] for u in watch['updates']]),
        'sampled_peak_tree_rss_mib': watch['sampled_peak_tree_rss_kib'] / 1024,
        'retained_tree_rss_mib': watch['retained_tree_rss_kib'] / 1024,
    }
    group['fixture'] = {}
    for language, suffix in [('ts', '.ts'), ('php', '.php'), ('rust', '.rs')]:
        sources = [p.read_bytes() for p in (root / f'fixtures-{size}' / language).rglob('*' + suffix)]
        group['fixture'][language] = {
            'files': len(sources),
            'source_bytes': sum(len(source) for source in sources),
            'lines': sum(len(source.splitlines()) for source in sources),
        }
    summary[size] = group
print(json.dumps(summary, indent=2))
