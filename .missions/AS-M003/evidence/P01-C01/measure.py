"""Reuse the unmodified P01 measurement implementation with C01 output placement."""
from pathlib import Path

canonical = Path(__file__).resolve().parent.parent / "P01" / "measure.py"
exec(compile(canonical.read_bytes(), str(canonical), "exec"), {"__name__": "__main__", "__file__": __file__})
