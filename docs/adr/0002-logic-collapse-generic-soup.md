*Logical collapse: one simulate, associated types, generic soup*

**Labels:** `decision-record` `architecture` `rust-core`
**Status:** Implemented on `refac/generator-abstraction`
**Commits:** `fb038c7`, `81606fe`, `2301c77`, `bcae1aa`, `2285ce2`, `d1d3f98`

## Context

`Soup` exposed eight overlapping ways to run a simulation:

```
react                          react_logged
simulate_for                   simulate_for_logged
simulate_for_logged_filtered   simulate_and_poll
simulate_and_poll_with_killer  simulate_and_record
```

These were not orthogonal — they were the cross product of three independent
questions (how many steps, what to record, whether to poll) expanded into
separate functions. `react` and `react_logged` were near-identical: the sampling,
collision, trimming and parent-handling blocks were duplicated line for line,
differing only in that the logged version materialized products into a `Vec`
first.

That duplication was already doing damage. A fix to the reaction logic had to be
applied twice, and applying it once would have produced **different physics
depending on whether logging was enabled** — the worst possible failure mode for
a scientific tool, because it is invisible and only manifests when someone
compares a logged run to an unlogged one.

Separately, `Soup` carried four type parameters and two `PhantomData` fields with
a standing TODO:

```rust
pub struct Soup<P, C, T, E> {
    // TODO: Figure out how to get rid of these horrible phantomdatas
    pub(crate) t: PhantomData<T>,
    pub(crate) e: PhantomData<E>,
}
```

## Decisions

**1. One `simulate`, parameterized by a recording enum.**

```rust
pub fn simulate(
    &mut self,
    n: usize,
    recording: RecordingType,
    polling_interval: Option<usize>,
) -> Option<Tape<P, C, G>>
```

with `RecordingType { None, All, SuccessOnly }`.

An enum rather than a boolean, because the filtered variant is a genuine third
state, not a second flag — `simulate_for_logged_filtered` existed precisely
because "record everything" and "record successes" are different requests.
Booleans would have made that a `(bool, bool)` with one impossible combination.

`react` collapsed to a single implementation taking `(step, record)`. The bool
parameter is a mild smell, but it is contained to one private call site and it
buys a single definition of the reaction sequence.

**2. `polling_interval` as `Option<usize>` rather than a separate function.**
Polling was never a different algorithm — it was the same loop with a snapshot
condition. `None` means never poll, which mirrors the config field exactly.

**3. Move `T` and `E` into `Collider` as associated types.**

```rust
pub trait Collider<P> where P: Particle {
    type Product: Residue<P> + Display + Clone;
    type Error: std::error::Error + Display + Clone;
    fn collide(&self, left: P, right: P) -> Result<Self::Product, Self::Error>;
}
```

`Soup<P, C, T, E>` became `Soup<P, C, G>`. This is the change that **deleted both
`PhantomData` fields and the TODO above them.**

The reasoning: a collider's product and error types are not free choices a caller
should make — they are determined by the collider. Expressing them as separate
type parameters forced every signature in the crate to carry and re-bind two
types that were never actually variable, which is exactly what `PhantomData` was
papering over. Associated types say "the collider decides", which is true.

**4. Add `G: Generator<P>` to `Soup`.**
The soup now owns its generator. This is what makes `seed_with_generator(n)`
possible, and that is what the recursive experiment's refill step needs — a soup
that can repopulate itself without the caller reaching back into config.

Net type-parameter count is unchanged at three, but all three are now things a
caller genuinely chooses.

**5. Fold `Tape` into `record.rs` carrying both histories.**
`Tape` now holds `soup_history` and `reaction_record` together, rather than being
the return type of one specific function. One recording concept, one type.

**6. Generic `cull` on `Soup`**
Made generic cull method on `Soup` to remove records at random.
Putting them on `Soup` rather than in the experiment module means any future 
experiment gets them. Also sets the stage for the spacial extension by consolidating
expression removal to a singular call point on the `Soup`.

**7. Extract the recursive experiment to its own module.**
`recursive_experiment.rs` rather than another variant in `experiments/`. This
implements the agreed "recursion as the global outer loop" decision — it is not
one experiment among many, it is the loop every run passes through, with
`n_generations` defaulting to 1 so a single run is the degenerate case.

## Consequences

**Good:** `soupercollider.rs` 340 → 188 lines. One definition of the reaction
sequence, so the dilution-ordering question now has exactly one place to be
answered. No `PhantomData`. The recursive experiment became expressible.

**Costs and follow-ups:**
- `simulate` returns `Option<Tape>`, which is `None` for `RecordingType::None`.
  Slightly awkward — the alternative was always allocating a `Tape` and returning
  an empty one, which is worse. Noting it as a known wart rather than a problem.
- **The collapse did not fix the bugs it inherited.** `n_collisions` is still
  never incremented, and the hardcoded `32000, 16000` in the collider
  survived the move. Consolidating the code made them single-site fixes,
  which is the point — but they are still open.
- A regression introduced here: `RecordingType::All` is hardcoded at the
  recursive experiment's call site, so every run records every collision and then
  discards it.
- `PySoup.from_config` signature changed as a consequence.
