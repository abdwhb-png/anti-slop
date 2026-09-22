"""Controlled evidence fixtures; expected observations are not design-quality labels."""
from pathlib import Path
import json
import sys

def generate(root: Path, count: int = 200):
    if root.exists():
        raise FileExistsError(root)
    def put(path, text):
        dest=root/path; dest.parent.mkdir(parents=True,exist_ok=True); dest.write_text(text)
    put('ts/tsconfig.json',json.dumps({'compilerOptions':{'strict':True,'target':'ES2022','module':'ESNext','moduleResolution':'Bundler','paths':{'@domain':['./domain.ts']}},'include':['**/*.ts']},indent=2))
    put('ts/domain.ts','''// Reject empty identifiers at the boundary.
export type OrderId = string & { readonly __brand: "OrderId" };
export function normalize(raw: string): OrderId {
  if (raw.length === 0) throw new Error("empty");
  return raw as OrderId;
}
''')
    put('ts/repository.ts','''import type { OrderId } from "@domain";
export interface Repository { load(id: OrderId): string; }
export class MemoryRepository implements Repository {
  load(id: OrderId): string { return id; }
}
''')
    put('ts/service.ts','''import { normalize as parseId } from "@domain";
import { MemoryRepository } from "./repository";
export const result = parseId("42");
export const answer = new MemoryRepository().load(result);
export async function dynamicModule(path: string) { return import(path); }
''')
    put('ts-errors/broken.ts','export function broken( {\n')
    put('ts-errors/missing.ts','import { absent } from "./does-not-exist"; export const value = absent(42);\n')
    put('php/composer.json',json.dumps({'name':'qualification/fixture','autoload':{'psr-4':{'Fixture\\':'src/'}}},indent=2))
    put('php/mago.toml','php-version = "8.5"\nthreads = 1\n[source]\npaths = ["src"]\n[analyzer]\n')
    put('php/src/OrderId.php','''<?php
namespace Fixture;
final readonly class OrderId {
    public function __construct(public string $value) {
        if ($value === '') { throw new \\InvalidArgumentException('empty'); }
    }
}
''')
    put('php/src/Repository.php','''<?php
namespace Fixture;
interface Repository { public function load(OrderId $id): string; }
''')
    put('php/src/MemoryRepository.php','''<?php
namespace Fixture;
final class MemoryRepository implements Repository {
    public function load(OrderId $id): string { return $id->value; }
}
''')
    put('php/src/Service.php','''<?php
namespace Fixture;
function serve(Repository $repository, string $raw): string {
    if ($raw === '') { throw new \\InvalidArgumentException('empty'); }
    $id = new OrderId($raw);
    return $repository->load($id);
}
function dynamic_call(object $service, string $method): mixed {
    return $service->$method();
}
''')
    put('php-errors/Broken.php','<?php function broken( {\n')
    put('php-errors/Missing.php','<?php namespace Fixture; final class Child extends MissingParent {}\n')
    put('rust/Cargo.toml','[workspace]\nmembers = ["domain", "app"]\nresolver = "3"\n')
    put('rust/domain/Cargo.toml','[package]\nname = "qualification-domain"\nversion = "0.1.0"\nedition = "2024"\n[features]\nextra = []\n')
    put('rust/domain/build.rs','fn main() { std::fs::write(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("qualification-build-script-ran"), b"executed").unwrap(); println!("cargo:rustc-cfg=from_build"); }\n')
    put('rust/domain/src/lib.rs','''pub struct OrderId(pub u64);
pub trait Repository { fn load(&self, id: &OrderId) -> u64; }
pub struct MemoryRepository;
impl Repository for MemoryRepository { fn load(&self, id: &OrderId) -> u64 { id.0 } }
#[cfg(feature = "extra")]
pub fn feature_only() -> u64 { 99 }
macro_rules! make_getter { ($name:ident) => { pub fn $name() -> u64 { 7 } }; }
make_getter!(generated);
''')
    put('rust/app/Cargo.toml','[package]\nname = "qualification-app"\nversion = "0.1.0"\nedition = "2024"\n[dependencies]\nqualification-domain = { path = "../domain" }\n')
    put('rust/app/src/lib.rs','''use qualification_domain::{OrderId, Repository, MemoryRepository, generated};
pub fn serve() -> u64 {
    let id = OrderId(42);
    MemoryRepository.load(&id) + generated()
}
''')
    for i in range(count):
        put(f'ts/generated/item{i}.ts', 'import { normalize } from "@domain";\n'+''.join(f'export function item{i}_{j}(raw: string) {{\n  if (raw.length === {j}) throw new Error("length");\n  return normalize(raw);\n}}\n' for j in range(12)))
        put(f'php/src/Generated/Item{i}.php', '<?php\nnamespace Fixture\\Generated;\n'+f'final class Item{i} {{\n'+''.join(f' public function value{j}(int $raw): int {{ return $raw + {j}; }}\n' for j in range(12))+'}\n')
        put(f'rust/app/src/item{i}.rs',''.join(f'pub fn value{j}(raw: u64) -> u64 {{ raw + {j} }}\n' for j in range(12)))
    app=root/'rust/app/src/lib.rs'
    app.write_text(app.read_text()+''.join(f'pub mod item{i};\n' for i in range(count)))
    return root

if __name__=='__main__':generate(Path(sys.argv[1]),int(sys.argv[2]) if len(sys.argv)>2 else 200)
