# Iterative Evolver — Model Routing

The evolver uses tiered model selection at the phase level. Assess, Analyze, Plan, and Reflect all require frontier reasoning because they operate at ambiguity boundaries. Only Status and Persist are safe on smaller models.

## Policy Source

```
project.json → model_policy.phases.evolver-*
project.json → model_policy.registry.<class>.<active_environment>
```

If `project.json` is not present or lacks `model_policy`, treat all phases as `frontier` and log a warning to `.evolver/<evolution_name>/model-routing.log`.

---

## Phase → Class Map

Slash CommandPhase KeyClassRationale`/evolve-assessevolver-assess`frontierHolistic cross-domain gap analysis`/evolve-analyzeevolver-analyze`frontierExternal landscape synthesis against current state`/evolve-planevolver-plan`frontierDecomposition under ambiguity`/evolve-executeevolver-execute`tieredScored per task (see Execute Tiering below)`/evolve-reflectevolver-reflect`frontierQuality judgment and regression detection`/evolve-reportevolver-reflect`frontierSynthesis — same class as reflect`/evolve-statusevolver-status`smallRead-only reporting from known state files

---

## Execute Phase Tiering

When `/evolve-execute` delegates to the KBD inner loop, routing responsibility transfers to `kbd-process-orchestrator`. The evolver does not override per-change model class — KBD's `references/model-routing.md` applies.

When `/evolve-execute` runs natively (non-software domains), score each task:

ComplexityClassIndicatorsLowsmallInformation retrieval, file writes, status updatesMediummediumStructured analysis, cross-source comparison, bounded synthesisHighfrontierCross-domain reasoning, ambiguous synthesis, judgment under uncertainty

---

## Routing Directive Format

The meta-controller emits a machine-readable routing directive at each phase transition. External orchestrators (prom-lanes, UAR) parse this to select the inference endpoint before the next phase begins:

```
[MODEL_ROUTING] phase=<phase-key> class=<class> model=<concrete-model> env=<environment>
```

Example:

```
[MODEL_ROUTING] phase=evolver-assess class=frontier model=claude-sonnet-4-6 env=local
[MODEL_ROUTING] phase=evolver-plan class=frontier model=claude-sonnet-4-6 env=local
[MODEL_ROUTING] phase=evolver-execute class=small model=Qwen3.5-9B-Q8_0 env=local
```

Emit this directive immediately before loading the phase controller prompt. Log it to `.evolver/<evolution_name>/model-routing.log` for audit.

---

## Fallback Rule

If `model_policy` is absent from `project.json`:

- Treat all phases as `frontier`
- Log: `[WARN] model_policy absent — defaulting to frontier for all phases`
- Do not silently downgrade any phase
