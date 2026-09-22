// Qualification of the real TypeScript LanguageService; not product code.
const fs = require('node:fs');
const path = require('node:path');
const {performance} = require('node:perf_hooks');
const ts = require(process.argv[2]);
const root = path.resolve(process.argv[3]);
const start = performance.now();
const configPath = path.join(root,'tsconfig.json');
const config = ts.readConfigFile(configPath,ts.sys.readFile);
if(config.error) throw new Error(ts.flattenDiagnosticMessageText(config.error.messageText,'\n'));
const parsed = ts.parseJsonConfigFileContent(config.config,ts.sys,root);
const overlays = new Map();
let generation = 0;
const readFile = file => overlays.get(file)?.text ?? ts.sys.readFile(file);
const host = {
  getScriptFileNames:()=>parsed.fileNames,
  getScriptVersion:file=>String(overlays.get(file)?.version ?? 0),
  getProjectVersion:()=>String(generation),
  getScriptSnapshot:file=>{const text=readFile(file);return text===undefined?undefined:ts.ScriptSnapshot.fromString(text)},
  getCurrentDirectory:()=>root,
  getCompilationSettings:()=>parsed.options,
  getDefaultLibFileName:options=>ts.getDefaultLibFilePath(options),
  fileExists:ts.sys.fileExists,readFile,readDirectory:ts.sys.readDirectory,
  directoryExists:ts.sys.directoryExists,getDirectories:ts.sys.getDirectories,
};
const service = ts.createLanguageService(host);
const domain = path.join(root,'domain.ts');
const consumer = path.join(root,'service.ts');
const repository = path.join(root,'repository.ts');
const original = readFile(domain);
function resultType() {
  const program=service.getProgram();
  const checker=program.getTypeChecker();
  const source=program.getSourceFile(consumer);
  let result;
  function visit(node) {
    if(ts.isVariableDeclaration(node)&&node.name.getText(source)==='result') result=checker.typeToString(checker.getTypeAtLocation(node.name));
    ts.forEachChild(node,visit);
  }
  visit(source);
  return result;
}
const before=resultType();
const initialMs=performance.now()-start;
const beforeRss=process.memoryUsage().rss;
const referencePosition=original.indexOf('normalize')+1;
const aliasPosition=readFile(consumer).indexOf('parseId("')+1;
const interfacePosition=readFile(repository).indexOf('interface Repository')+'interface '.length+1;
const definitions=service.getDefinitionAtPosition(consumer,aliasPosition);
const references=service.findReferences(domain,referencePosition);
const implementations=service.getImplementationAtPosition(repository,interfacePosition);
const diagnostics=parsed.fileNames.flatMap(file=>service.getSyntacticDiagnostics(file).concat(service.getSemanticDiagnostics(file))).map(d=>({code:d.code,file:d.file?.fileName,start:d.start,message:ts.flattenDiagnosticMessageText(d.messageText,'\n')}));
const initialWorkMs=performance.now()-start;
const unchanged=[];
for(let i=0;i<20;i++){const t=performance.now();const type=resultType();unchanged.push({ms:performance.now()-t,type,rss:process.memoryUsage().rss});}
const updates=[];
for(let i=0;i<20;i++) {
  const edited=i%2===0;
  const text=edited?original.replace('): OrderId {','): number {').replace('return raw as OrderId;','return raw.length;'):original;
  const t=performance.now();
  overlays.set(domain,{text,version:++generation});
  const type=resultType();
  const consumerDiagnostics=service.getSemanticDiagnostics(consumer).map(d=>d.code);
  updates.push({generation,edited,ms:performance.now()-t,type,consumerDiagnostics,rss:process.memoryUsage().rss});
}
console.log(JSON.stringify({analyzer:'typescript-language-service',version:ts.version,files:parsed.fileNames.length,initial_ms:initialMs,initial_work_ms:initialWorkMs,before_rss:beforeRss,result_type:before,definitions,references,implementations,diagnostics,unchanged,updates,final_rss:process.memoryUsage().rss,scope:'native project semantics; dynamic imports remain unresolved'}));
service.dispose();
