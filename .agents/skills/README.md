# Repository skills

These skills contain reusable procedures for Anti-Slop development. [AGENTS.md](../../AGENTS.md) holds the shorter repository instructions and critical fallbacks; [accepted ADRs](../../docs/decisions/README.md) own architectural choices.

| Skill | Use for | Skip for |
| --- | --- | --- |
| [anti-slop-rule-evidence](anti-slop-rule-evidence/SKILL.md) | Designing, implementing or qualifying a rule's evidence, counterexamples and assessment boundaries. | Unrelated formatting, dependency setup, CLI metadata or generic product planning. |
| [anti-slop-analyzer-qualification](anti-slop-analyzer-qualification/SKILL.md) | Evaluating a native adapter's actual facts, generation coherence or whole-path resource cost. | Routine dependency metadata edits, ordinary Rust builds or a documentation-only version correction. |
| [axi](axi/SKILL.md) | Designing, implementing or reviewing agent-facing CLI ergonomics: bounded output, help, errors, explicit truncation and discovery. | Internal analysis or storage work that does not change the CLI contract. |

Apply AXI within the accepted Anti-Slop decisions, especially [ADR-006](../../docs/decisions/006-output-formats-and-shared-projection.md): the canonical contract is Rust-owned, TOON is the default report encoding, explicit JSON remains available, and JSONL serves dataset interchange. AXI's general recommendations do not by themselves adopt new flags, exit codes or session integrations, or establish full AXI conformance.

The skills are ordinary repository files with YAML frontmatter. A harness may discover them automatically, but this repository does not assume that it does. When starting another agent, point it to the root `AGENTS.md` and the relevant `SKILL.md`; direct reading is sufficient to access the instructions. Automatic discovery, global installation and harness-specific mirrors have not been configured or verified here.

The two Anti-Slop workflow skills include representative evaluation prompts and observable assertions in `evals/evals.json`. They test a normal case, an ambiguous case and a boundary that should not trigger the workflow. These are workflow evaluations, not production analyzer tests or rule-quality evidence. The added AXI skill was not part of those evaluations. See [validation notes](../../docs/agent-workflow-validation.md) for what was actually checked.
