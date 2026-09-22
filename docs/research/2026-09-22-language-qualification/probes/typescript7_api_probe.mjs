// Real, version-pinned TypeScript 7 unstable API: qualify structured type access.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {performance} from 'node:perf_hooks';
const {API}=await import(pathToFileURL(process.argv[2]));
const root=path.resolve(process.argv[4]);
const api=new API({cwd:root,tsserverPath:process.argv[3],collectTiming:true});
const domain=path.join(root,'domain.ts');const source=path.join(root,'service.ts');
const original=fs.readFileSync(domain,'utf8');const text=fs.readFileSync(source,'utf8');
const position=text.indexOf('result =')+1;
let snapshot;
const started=performance.now();
const report={analyzer:'typescript-native-unstable-api',version:'7.0.2',updates:[]};
try {
  snapshot=api.updateSnapshot({openProjects:[path.join(root,'tsconfig.json')]});
  let project=snapshot.getProject(path.join(root,'tsconfig.json'));
  const type=project.checker.getTypeAtPosition(source,position);
  report.initial={ms:performance.now()-started,type:project.checker.typeToString(type),flags:type.flags,isError:type.isErrorType()};
  const symbol=project.checker.getSymbolAtPosition(source,text.indexOf('parseId("')+1);
  const target=project.checker.getAliasedSymbol(symbol);
  report.alias={name:target.name,flags:target.flags,isUnknown:project.checker.isUnknownSymbol(target),declarations:target.declarations.map(handle=>{const node=handle.resolve();return {file:handle.path,kind:handle.kind,pos:node?.pos,end:node?.end};})};
  for(let i=0;i<20;i++){
    const changed=i%2===0;const t=performance.now();
    fs.writeFileSync(domain,changed?original.replace('): OrderId {','): number {').replace('return raw as OrderId;','return raw.length;'):original);
    const prior=snapshot;snapshot=api.updateSnapshot({fileChanges:{changed:[domain]}});prior.dispose();
    project=snapshot.getProject(path.join(root,'tsconfig.json'));
    const type=project.checker.getTypeAtPosition(source,position);
    report.updates.push({ms:performance.now()-t,expected:changed?'number':'OrderId',type:project.checker.typeToString(type),flags:type.flags});
  }
  report.timing=api.getTimingInfo();
  console.log(JSON.stringify(report));
} finally {
  if(fs.readFileSync(domain,'utf8')!==original)fs.writeFileSync(domain,original);
  snapshot?.dispose();api.close();
}
