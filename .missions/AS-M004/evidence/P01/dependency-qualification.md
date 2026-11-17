# SQLite dependency qualification — P01

Qualified on 2026-11-17 against the live crates.io API before the build. The seven-day release-age window applies to every added package, including transitives.

## Choice

`rusqlite` is used as the maintained Rust binding over `libsqlite3-sys`; it provides transactions, busy timeout, pragmas and typed row access without adding a query builder or repository layer. `bundled` compiles the vendored SQLite 3.54.1 amalgamation, so the build does not depend on a system SQLite version. Default features are disabled: `rusqlite`'s `default = ["cache", "ffi-sqlite-wasm-rs"]` would add a statement cache (`hashlink`) and a wasm-only dependency that this store does not need.

The same exact package is a dev-dependency so tests can open a second raw connection for lock contention, trigger-based failure injection and row-count verification. It adds no second package to the graph.

## Registry facts

| Package | Version | Published | Yanked | Rust | Locked checksum |
| --- | --- | --- | --- | --- | --- |
| `rusqlite` | 0.41.1 | 2026-10-28 | no | 1.89.0 | `da9bba6f3506135e120a6586df05be18b48d18896600646a4bec8f993d91a594` |
| `libsqlite3-sys` | 0.39.1 | 2026-10-25 | no | 1.88.0 | `14a5f01b87b92705d07b433c06f96cbc764637d81a0bffb5734ce17cf0f9a2b3` |
| `cc` | 1.11.1 | 2026-11-06 | no | 1.65.0 | `4ff2a8126fff0f8c839080d619675a585e61720bb12d2baceae45769fd66f155` |
| `find-msvc-tools` | 0.1.15 | 2026-10-19 | no | 1.65.0 | `1c66915cc5391097de7a2b0b972e8505aa33c0df401fd3a6811dbdb32881ba48` |
| `fallible-iterator` | 0.3.0 | 2023-05-22 | no | — | `2acce4a10f12dc2fb14a218589d4f1f62ef011b2d0cc4b3cb1bba8e94da14649` |
| `fallible-streaming-iterator` | 0.1.9 | 2018-05-02 | no | — | `7360491ce676a36bf9bb3c56c1aa791658183a54d2744120f27285738d90465a` |
| `pkg-config` | 0.3.34 | 2026-08-14 | no | 1.63 | `f6b464fbc74e149a392436b17d523f769e057cb6877f6a5c4618bc6f11800548` |
| `shlex` | 2.0.1 | 2026-05-17 | no | 1.46.0 | `f8fadd59c855ef2080decdef8ff161eb6661b86933c9d82e5ba29dc602a55aba` |
| `vcpkg` | 0.2.15 | 2021-06-19 | no | — | `accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426` |

Rejected as too recent at the qualification date:

- `rusqlite 0.43.0` (2026-11-15, two days) and `rusqlite 0.42.0` (2026-11-11, six days).
- `libsqlite3-sys 0.40.0` (2026-11-11, six days); `rusqlite 0.41.1` requires `^0.39.0`, so the resolved `0.39.1` is used.
- `cc 1.11.2` (2026-11-13, four days). `cc` was resolved at 1.11.2 first and the dependency policy refused the graph; it was then pinned to `1.11.1` with `sfw cargo update -p cc --precise 1.11.1`.

Existing locked `bitflags` and `smallvec` satisfy the new requirements and were not replaced.

## Policy result

`python3 scripts/ci/policy.py dependencies --base 45dac3c241338c94166369fe821fe1d3499cf974`:

```text
Qualified cargo cc@1.11.1
Qualified cargo fallible-iterator@0.3.0
Qualified cargo fallible-streaming-iterator@0.1.9
Qualified cargo find-msvc-tools@0.1.15
Qualified cargo libsqlite3-sys@0.39.1
Qualified cargo pkg-config@0.3.34
Qualified cargo rusqlite@0.41.1
Qualified cargo shlex@2.0.1
Qualified cargo vcpkg@0.2.15
Qualified 9 added/changed locked registry packages.
```

The first run of the same command failed the release-age/integrity check while `cc 1.11.2` was locked; `dependency-qualification-first-policy-failure.txt` retains that refusal. The full command output is in `policy-dependencies.txt`.

## Ordering deviation, disclosed

The intended order was: resolve the graph, run `policy.py`, then fetch. In practice, `sfw cargo metadata --format-version 1` resolved the graph and downloaded the nine package archives through Socket Firewall in that same step, before `policy.py` ran. `cc 1.11.2` was downloaded this way and later rejected. No build or test ran before policy qualification; the eventual build used only the qualified `cc 1.11.1`. The cached `1.11.2` archive was preserved rather than cleared, per the repository's cache policy, and is not part of the resolved graph. This ordering shortfall is recorded rather than hidden.

`sfw --version` reported `Socket Firewall Free, version 1.16.0`. All networked Cargo operations (`metadata`, `update`, `fetch`) ran through `sfw`. Existing cached archives are not rescanned by Socket Firewall; that limitation is unchanged.

## Toolchain compatibility

`rust-toolchain.toml` pins 1.96.1; `rusqlite 0.41.1` declares Rust 1.89.0 and `libsqlite3-sys 0.39.1` declares Rust 1.88.0, both satisfied. `Cargo.lock` remains version 4. The resolved graph is recorded in `cargo-tree.txt`.
