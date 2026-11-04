// Independent TypeScript reference decoder. Exact package/integrity in lockfile.
import { readFileSync } from 'node:fs';
import { decode } from '@toon-format/toon';
process.stdout.write(JSON.stringify(decode(readFileSync(0, 'utf8'), { strict: true, indentSize: 2 })));
