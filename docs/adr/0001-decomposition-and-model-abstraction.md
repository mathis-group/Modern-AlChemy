# Structural refactor: module decomposition and the abstraction seam

**Labels:** `decision-record` `architecture` `rust-core`
**Status:** Implemented on `refac/generator-abstraction`
**Commits:** `036aaea`, `e52bb49`, `2d6de30`, `976f007`, `1b8827e`, `35bcc0e`, `1c1ee90`

## Context

The core library had accumulated in a handful of very large files:

| File | Lines | Problem |
|---|---|---|
| `src/lambda/recursive.rs` | 398 | particle, collider, result, soup and helpers all in one file |
| `src/lambda/core.rs` | 306 | **dead near-duplicate** of the above — declared in `mod.rs`, imported by nothing |
| `src/supercollider.rs` | 340 | soup, tape, traits and six simulate entry points |
| `src/generators.rs` | 330 | config structs and generator behaviour interleaved |
| `src/config.rs` | 285 | every config struct in one file |
| `src/main.rs` | 257 | CLI definition, config resolution and dispatch |

Two forces made this urgent rather than cosmetic:

1. **The agreed direction is multiple expression types** — untyped lambda first,
   then simply-typed lambda and Haskell. Nothing in the layout distinguished
   "this is lambda-specific" from "this is generic machinery", so there was no
   seam to extend along.
2. **`lambda/core.rs` had already caused harm.** It carried its own divergent
   copies of the same bugs (the wrong `WARNING` comment, the `n == rlimit`
   off-by-one, the `parse().unwrap()`). Anyone fixing `recursive.rs` would have
   left it stale — and it had the better name, so a newcomer would plausibly have
   imported the wrong one.

## Decisions

**1. Delete `lambda/core.rs` outright rather than reconcile it.**
It was unreachable and divergent. Reconciling two copies of collision logic that
nothing imports is work with no beneficiary. Git retains it.

**2. Split by responsibility, not by type.**
`lambda/` became `particle.rs`, `collider.rs`, `result.rs`, `soup.rs`,
`generator.rs`, `utils.rs`, `analysis.rs`. Each file answers one question. The
alternative — keeping one file and using inline `mod` blocks — was rejected
because it improves nothing about navigation or review diffs.

**3. Separate config *representation* from config *behaviour*.**
This is the least obvious call and the one most worth recording. Generators now
live in two places:

- `config/generators/{b_tree_gen,fontana_gen}.rs` — the serde structs
- `lambda/generators/{b_tree_gen,fontana_gen}.rs` — the implementations

The reasoning: a config file is a serialization format that must stay stable and
is *not* inherently expression-specific, whereas a generator's behaviour is
entirely expression-specific. Keeping them in one file forced the serde surface
to move every time the implementation did. Splitting them means a Haskell
generator can be added under `lambda/` — or eventually a `haskell/` — without
touching the config schema layout.

**4. Move `analysis.rs` under `lambda/`.**
It is `impl LambdaSoup` over `HashSet<Term>` and `HashMap<Term, u32>`. It is
honestly lambda-specific today. Moving it says so, rather than leaving it at the
crate root implying a generality it does not have. Genericizing it over
`P: Particle + Hash + Eq` is deferred to the reactor work, where it belongs.

**5. Introduce `traits.rs` as the explicit abstraction seam.**
`Particle`, `Collider`, `Generator`, `Residue` in one place. Previously `Particle`
existed but its methods were never called, and the real coupling ran through
concrete types — so the abstraction that existed was not the one bearing load.
Putting the traits in a dedicated module makes the seam something you have to
deliberately cross.

**6. Extract the CLI from `main.rs`.**
`main.rs` went 257 → 71 lines; `cli.rs` is 86. `main` is now a dispatcher: parse,
resolve config, build a soup, run. Argument definition and config precedence live
in one place instead of being interleaved with execution.

**7. Enums live next to what they select over.**
`ExpressionType` → `src/enums.rs` (crate-level, it selects a whole backend);
`Experiment` → `src/experiments/enums.rs` (it selects within that module).

**8. Rename `AlchemyCollider` → `LambdaCollider`.**
There will be more than one collider. A name that says "Alchemy" says nothing
distinguishing once `HaskellCollider` exists. Also renamed `supercollider.rs` →
`soupercollider.rs`.

**9. Add `ExpressionType` with `todo!()` arms rather than waiting.**

```rust
match config.expression_type {
    ExpressionType::UntypedLambda     => LambdaSoup::from_config(&config),
    ExpressionType::SimplyTypedLambda => todo!("typed lambda soup"),
    ExpressionType::Haskell           => todo!("haskell soup"),
}
```

Deliberate. The alternative was to add the enum only when a second expression
type exists. Adding it now means the compiler enforces that every dispatch point
acknowledges all three, so the seam cannot silently close up as more code lands.
The cost is two reachable `todo!()`s, which is acceptable while the variants are
unreachable from any shipped config.

## Consequences

**Good:** 306 lines of dead code gone. `soupercollider.rs` 340 → 188. The
generator abstraction is now real — `Generator<P>` is a trait with two
implementations behind it, which is what made the recursive experiment's
`seed_with_generator` possible.

**Costs and follow-ups:**
- `Config` gained `expression_type` and `recursive_config`, both required, no
  serde defaults — this invalidates every pre-refactor config file (MA-03, MA-04)
- `PySoup.from_config` now takes `PyConfig` rather than `PyReactor`, which leaves
  `PyReactor` orphaned (MA-20, XR-02)
- The reactor **trait** seam is complete — `Collider<P>` with associated
  `Product`/`Error`, and `Soup<P, C, G>` never names `LambdaCollider`. What this
  commit range did *not* carry was the reactor **config** shape, which was still
  a flat struct of untyped-lambda fields. That landed subsequently,
  leaving only the collider's filter chain and the illegal-pairing problem
 between here and a second expression type.
