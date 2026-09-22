# Agent documentation and workflow validation

Date: 2026-09-22. Scope: repository instructions, ADRs, task template and two local skills. These checks do not qualify the product, its analyzers or its rules.

## Static and scope checks

- Checked local Markdown targets and section anchors in the new instructions, ADRs, skills, template and edited documentation.
- Checked sequential ADR numbering, required sections, decision lineage D1–D18 and the complete R1–R9 rule table.
- Checked both skill frontmatters and evaluation JSON: two skills, three cases each, twenty observable assertions in total.
- Compared SHA-256 hashes for 69 pre-existing source, test, dependency and research files against a snapshot taken before this documentation task; all remained unchanged.
- Checked whitespace and document scope. No PRD, application feature, dependency installation, analyzer run or paid judge call was added by this task.

The skill creator's `quick_validate.py` could not run because its Python environment lacks PyYAML. No dependency was installed to work around this. A standard-library check validated the actual files' restricted frontmatter shape (plain `name` and `description` fields), name/description constraints, required trigger sections and evaluation structure. This is not a general YAML parser or a substitute for testing every skill loader.

## Instruction walkthrough

| Situation | Checked behavior |
| --- | --- |
| A small README correction | Documentation/diff checks are sufficient; no artificial application RED test or full qualification is required. |
| A later authorized implementation | The current request controls scope; the historical documentation-only phase is not a permanent ban on coding. |
| A rule skill fails to load | The root instructions still require counterevidence, explicit incompleteness and scoped blocking eligibility. |
| A harness has no automatic skill discovery | The catalog supports direct file reading without global paths, mirrors or a promised loader. |
| Required installation controls are unavailable | Stop the dependent installation; do not silently bypass controls or assume a temporary research approval covers production. |
| A proposal conflicts with an accepted decision | ADRs own accepted choices; research and historical interfaces retain their limited status. |

These are direct instruction reviews. No external harness configuration or permission mechanism was activated or tested.

## Simulated skill evaluations

The [rule cases](../.agents/skills/anti-slop-rule-evidence/evals/evals.json) and [analyzer cases](../.agents/skills/anti-slop-analyzer-qualification/evals/evals.json) contain a representative request, an ambiguous case and a boundary case each. The executor was a general-purpose **worker using gpt-5.6-terra with high reasoning**, appropriate for planning and evidence assessment. Every case used that executor configuration in both conditions; these were not code-execution or browser tests.

Fresh agents received the same canonical context excerpts and six prompts. The with-skill condition additionally received both skill bodies; the baseline did not. Responses were limited to 170 words per case. Expected outputs and assertions were withheld from the executors. The parent agent manually graded the twenty prewritten assertions; grading was not blinded. [Evaluation metadata and observations](validation/2026-09-22-agent-workflows.json) record the per-case results and executor fit.

| Case | Baseline | With skills | Discriminating observation |
| --- | --- | --- | --- |
| Rule 1: useful and redundant R6 wrappers | 3/4 | 4/4 | The baseline omitted the explicit positive/legitimate-negative/partial-evidence validation set. |
| Rule 2: partial references, R4, suppression and model score | 3/4 | 4/4 | Both rejected transferring Jev's score; the skill response also required independent qualification on frozen adjudicated data. |
| Rule 3: README spelling correction | 2/2 | 2/2 | Both kept the scope to documentation. Equal success does not demonstrate skill improvement. |
| Analyzer 1: cropped latency and process memory | 3/4 | 4/4 | Both identified the memory concern; the skill response explicitly requested raw observations and run counts across the required scenarios. |
| Analyzer 2: overlays, stale generation and unmapped macros | 4/4 | 4/4 | Both rejected unsupported current/exact evidence. Equal success does not demonstrate skill improvement. |
| Analyzer 3: historical version correction | 2/2 | 2/2 | Both preserved old evidence and avoided native execution. Equal success does not demonstrate skill improvement. |

The accepted run observed **17/20 baseline and 20/20 with skills**. This is one response per case and condition, batched in one agent per condition; it does not establish a statistically reliable improvement, trigger reliability or performance across models. All responses were simulated: no requested workflow action was executed.

### Input-delivery correction

An initial pair of runs and one focused rerun read only lines 1–240 of a longer context file, omitting ADR-007. Their resource answers lacked the budget table. They are excluded from the comparison above. The accepted run used a 90-line context and confirmed the final context marker and B1–B4 table were visible. The analyzer skill was also clarified to require explicit numeric comparisons and retained-idle observations; the accepted comparison used that revised version.

## Checks deliberately not run

No Cargo build, test, formatting or Clippy pass was repeated for this documentation-only change; production and dependency bytes were verified unchanged. Earlier scaffold validation remains a separate result, not a test performed by this task. No representative repository benchmark, production rule-quality evaluation, automatic skill-discovery test or cross-harness activation was performed.
