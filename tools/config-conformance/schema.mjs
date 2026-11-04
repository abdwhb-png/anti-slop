// Actual VS Code JSON editor engine, headless, with an explicitly local schema.
// No server, GUI assertion or network schema loader is involved.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { getLanguageService } from 'vscode-json-languageservice';
import { TextDocument } from 'vscode-languageserver-textdocument';

const root = new URL('../../', import.meta.url);
const schema = JSON.parse(readFileSync(new URL('schemas/anti-slop.schema.json', root), 'utf8'));
const cases = JSON.parse(readFileSync(new URL('tests/fixtures/config/conformance.json', root), 'utf8'));
const requests = [];
const service = getLanguageService({ schemaRequestService: async uri => {
  requests.push(uri); throw new Error('Remote schema loading is disabled for local qualification');
}});
service.configure({ validate: true, allowComments: true, schemas: [{
  uri: schema.$id, fileMatch: ['*.jsonc'], schema,
}] });
const settings = { schemaValidation: 'error', comments: 'ignore', trailingCommas: 'ignore' };
function document(source) {
  const doc = TextDocument.create('file:///AS-M002-controlled/.anti-slop.jsonc', 'jsonc', 1, source);
  return [doc, service.parseJSONDocument(doc)];
}
async function diagnostics(source, explicit = false) {
  const [doc, ast] = document(source);
  return service.doValidation(doc, ast, settings, explicit ? schema : undefined);
}
const observations = [];
for (const item of cases) {
  // Explicit schema ensures even an unsupported project $schema remains an
  // offline structural-value test, rather than an external fetch attempt.
  const result = await diagnostics(JSON.stringify(item.document, null, 2), true);
  assert.equal(result.length === 0, item.valid, `${item.name}: ${JSON.stringify(result)}`);
  observations.push({ name: item.name, accepted: result.length === 0, diagnostics: result });
}
async function complete(marked) {
  const offset = marked.indexOf('|');
  const [doc, ast] = document(marked.replace('|', ''));
  return (await service.doComplete(doc, doc.positionAt(offset), ast)).items;
}
const fields = await complete('{\n  |\n}');
for (const key of ['$schema', 'rules', 'overrides']) assert(fields.some(item => item.label === key), key);
const ruleItems = await complete(`{"$schema":${JSON.stringify(schema.$id)},"rules":{ | }}`);
for (let i = 1; i <= 9; i++) assert(ruleItems.some(item => item.label === `R${i}`), `R${i}`);
assert(!ruleItems.some(item => item.label === 'R10'));
const policies = await complete(`{"$schema":${JSON.stringify(schema.$id)},"rules":{"R1": | }}`);
for (const policy of ['off', 'advisory', 'block']) assert(policies.some(item => item.label === JSON.stringify(policy)), policy);
assert(policies.some(item => JSON.stringify(item.documentation).includes('qualification')));
const hoverSource = `{"$schema":${JSON.stringify(schema.$id)},"rules":{"R1":"advisory"}}`;
const [hoverDoc, hoverAst] = document(hoverSource);
const hover = await service.doHover(hoverDoc, hoverDoc.positionAt(hoverSource.indexOf('"R1"') + 2), hoverAst);
// This editor displays the selected enum variant's description on the rule
// key, rather than the generic property description beside its $ref.
assert(JSON.stringify(hover).includes('Request evaluation without blocking'), JSON.stringify(hover));
const jsonc = `/* local editor document */\n{"$schema":${JSON.stringify(schema.$id)},"rules":{"R1":"off",},}`;
assert.equal((await diagnostics(jsonc)).length, 0);
const invalid = await diagnostics(`{"$schema":${JSON.stringify(schema.$id)},"rules":{"R1":"permit"}}`);
assert(invalid.some(item => item.message.includes('off') && item.message.includes('block')));
for (const [count, valid] of [[256, true], [257, false]]) {
  const result = await diagnostics(JSON.stringify({ $schema: schema.$id, overrides: Array.from({ length: count }, () => ({ include: ['**'], rules: {} })) }));
  assert.equal(result.length === 0, valid);
}
assert.deepEqual(requests, []);
process.stdout.write(JSON.stringify({
  tool: 'VS Code JSON language service', version: '5.7.2', mode: 'headless editor-service API',
  association: { artifact: 'schemas/anti-slop.schema.json', registeredUri: schema.$id, fileMatch: '*.jsonc' },
  schemaDialect: schema.$schema, remoteRequests: requests, cases: observations,
  completion: { fields, rules: ruleItems, policies }, hover, invalidSettingDiagnostics: invalid,
  jsoncCommentsTrailingCommas: 'accepted', overrideBounds: '256 accepted;257 rejected',
}, null, 2));
