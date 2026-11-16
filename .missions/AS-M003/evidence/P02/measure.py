"""Reuse P01's canonical sampler with a comparison and a fresh output directory."""
import argparse
import os
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("selection", choices=["staged", "unstaged", "local", "base"])
parser.add_argument("--output", type=Path, required=True, help="new directory for observations; existing paths are refused before sampling")
args = parser.parse_args()
output = args.output.resolve()
try:
    output.mkdir(parents=True, exist_ok=False)
except FileExistsError:
    parser.error("output already exists; choose a new --output directory")
selection = args.selection
os.environ["CAPTURE_OBSERVATION_SELECTION"] = selection
canonical = Path(__file__).resolve().parent.parent / "P01" / "measure.py"
source = canonical.read_text()
source = source.replace('EVIDENCE = Path(__file__).resolve().parent', f'EVIDENCE = Path({str(output)!r})')
source = source.replace('f"observation-{count}', 'f"observation-' + selection + '-{count}')
source = source.replace('"measurements.json"', '"measurements-' + selection + '.json"')
source = source.replace('"measurement-build.log"', '"measurement-build-' + selection + '.log"')
exec(compile(source, str(canonical), "exec"), {"__name__": "__main__", "__file__": __file__})
