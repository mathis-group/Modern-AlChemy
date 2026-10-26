# Decision record: normal-form seeding and free-variable generation in FontanaGen

**Labels:** `decision-record` `rust-core` `generator`
**Status:** Implemented on `feat/normal-form_and_free-var` (commit `0a6f9a2`)

## Context

**Seeds and reaction products were treated differently.** The collider reduces every
reaction product to normal form, and fails the reaction if it exceeds `reduction_cutoff`
or `size_cutoff`. Generated expressions went into the soup unreduced. A soup therefore
started mostly unreduced and drifted toward normal form as products replaced the initial
population, so the first stretch of every run was partly the soup reducing its own seeds.

Fontana's original generator (`LambdaReactor/randomexpr.c` in `mathis-group/AlChemy`)
reduced every generated expression to normal form and generated a new one on failure
(lines 51–61 and 172–176). This is in the code; the paper does not state it.

**Reducing FontanaGen's output as it stood collapses it.** At the settings used as the
reference configuration (`min_depth 3`, `max_depth 20`, `p_abs 0.6`, `p_app 0.15`),
reducing each expression to normal form cut the median size from 20 nodes to 6 and the
share of distinct expressions from 86% to 27%. About 59% of its applications are redexes,
and about two thirds of its abstractions never use their variable, so reduction mostly
deletes argument subtrees.

**FontanaGen did not allow free variables, and two rules followed from that:**

- A branch with no abstraction above it could not emit a variable, so it was forced to be
  an application. That makes trees explode to the depth cap whenever `p_abs < 0.5`.
- At the depth cap with no abstraction above, it emitted `\x.x`. That is the source of
  expressions that are applications of identities and reduce to `\x.x`.

The original generator and `BTreeGen` both allow free variables during generation and
close the expression afterwards.

## Decisions

**1. Optional normal-form reduction in the Fontana generator.**
New `FontanaGen` config fields: `reduce_to_normal_form`, `reduction_cutoff`, `size_cutoff`.
When on, `generate()` reduces each expression and discards and regenerates any that
exceeds the cutoffs. It panics after 10,000 consecutive failures rather than looping
forever.

- The rejection rule is shared with the collider through
  `lambda::utils::reduce_to_normal_form`, so seeds and products are judged the same way.
- The cutoffs live in the generator config, independent of the reactor's. They should be
  no looser than the reactor's: a seed larger than the reactor's `size_cutoff` almost
  never reacts.
- Rejected alternative: a flag on `UntypedLambdaExpression` that reads the reactor's
  cutoffs, with `LambdaGenerator` wrapped in a struct to apply it. That works for every
  generator but changes the generator infrastructure; keeping it in the Fontana config
  leaves `LambdaGenerator`, `LambdaSoup` and the experiments untouched.

**2. Free variables in FontanaGen, using BTree's two settings.**
New config fields: `freevar_generation_probability` and `n_max_free_vars`.

- When a variable is generated, it comes from a pool of `n_max_free_vars` free variables
  with probability `freevar_generation_probability`. Otherwise it is bound by one of the
  abstractions above. With no abstraction above, it always comes from the pool.
- After generation, `lambda::utils::close_free_variables` adds one outer abstraction per
  distinct free variable that occurs. The first one used gets the innermost abstraction.
- With the pool on, the forced-application rule and the `\x.x` fallback no longer apply,
  because a variable is always possible.
- BTree's existing closing options were not reused. `prefix_standardize` binds every index
  up to the highest free one, adding unused abstractions for any gaps. `postfix_standardize`
  applies the closed expression to `\x.x` once per added abstraction, so it substitutes
  the identity for each free variable instead of binding it.

**3. Use `min_depth >= 1` with free variables; 3 is the working value.**
With `min_depth 0`, the root itself can be a free variable, which closes to `\x.x`. In
testing, that roughly doubled the identity share (from about 21% to about 40%).
`min_depth 3` keeps the first three levels to abstractions and applications.

**4. Existing behaviour is the default.**
Free variables and reduction are both off unless configured. The new fields have
`#[serde(default)]`, so config files without them still load, and the repository's
configs list them explicitly. With the pool off, the random stream is unchanged, so
existing seeds still generate the same expressions.

**5. Considered and not adopted: biasing abstractions out of function position.**
Scaling down the probability of an abstraction on the function side of an application
reduces redexes directly, and in testing it raised diversity after reduction. It was left
out to keep the generator's rules close to the original.

**6. Python binding.**
`PyFontanaGen.from_config` gains `freevar_generation_probability=0.0`, `max_free_vars=0`,
`reduce_to_normal_form=false`, `reduction_cutoff=500` and `size_cutoff=1000`. These are
trailing keyword arguments with defaults, so the dashboard's existing call is unaffected.

## Evidence

Soups were run with the library's own soup and collider and `default/config.json`:
1,000 expressions, 300,000 collisions, 30 replicates per variant. Two base settings were
used, both with `min_depth 3`, `max_depth 20`: `p_abs 0.6, p_app 0.15` and
`p_abs 0.5, p_app 0.25`.

Normal form plus free variables (`freevar_generation_probability 0.25`, `n_max_free_vars 6`)
compared with the previous generator seeding unreduced:

| | Previous generator | Normal form + free variables |
|---|---|---|
| Collisions until under 1% of products are new | 25k / 40k | 50k / 70k (p ≤ 0.001) |
| New products in the first 10k collisions | 6% / 10% | 11% / 21% (p < 10⁻⁶) |
| Species at the end (runs that did not collapse) | 22 / 39 | 31 / 49 (not significant, p 0.15) |

- **Reduction without free variables is worse than either.** It had the highest collapse
  rate measured (37% on the second base).
- **About one run in five collapses no matter the generator.** The soup ends dominated by
  `\x.\y.y`. Under the default rule, any collision led by `\x.\y.y` produces the identity,
  which `discard_identity` rejects. This is a reactor-configuration effect, to be tracked
  separately.

Behaviour was also checked directly:

- **Reduction:** each reduced output equals the normal form of the corresponding unreduced
  draw. Every output is in normal form and closed, and the cutoffs are honoured.
- **Free variables:** the free share among scoped variables matches the configured
  probability (0.000 / 0.252 / 0.498 / 1.000 for settings of 0 / 0.25 / 0.5 / 1).
  Closing leaves bound variables and tree shape unchanged and binds each free variable
  exactly once.
- **Removed rules:** the forced-application rule and the `\x.x` fallback no longer apply
  with the pool on.
- **Unchanged default:** with the pool off, output matches the previous generator exactly.

## Consequences

- **Seeds are smaller.** Normal forms at these settings have a median of about 7–9 nodes.
- **Every seed is reduced once,** which adds to the cost of generation.
- **Different expressions for the same seed.** With free variables on, a given seed produces
  different expressions than before.
- **The expected-size calculation used for feasibility checks assumes the forced-application
  rule.** With free variables on, growth is the single factor `p_abs + 2·p_app` everywhere.
- **A missing cutoff deserializes as 0.** A config that sets `reduce_to_normal_form: true` but
  omits a cutoff gets 0, and generation panics on the first expression.
- **`BTreeGen` has no normal-form option.** The same three fields could be added to its
  config if needed.

## Files

- `src/config/generators/fontana_gen.rs`: five new config fields
- `src/lambda/generators/fontana_gen.rs`: free-variable sampling, closing, normal-form
  generation; `new()` takes the new parameters
- `src/lambda/utils.rs`: `reduce_to_normal_form`, `close_free_variables`
- `src/lambda/collider.rs`: uses `reduce_to_normal_form` (behaviour unchanged)
- `src/python.rs`: new keyword arguments on `PyFontanaGen.from_config`
- `configs/fontana.json`: new fields listed explicitly, with both features off

## Follow-ups

- **A config for the chosen settings:** `min_depth 3`, free variables on, reduction on.
- **Sweeps:** `min_depth` (1–3), `freevar_generation_probability` and `n_max_free_vars`,
  measured in soups.
- **The recursive experiment:** run it with normal-form refills.
- **The `\x.\y.y` collapse:** investigate it under the default reaction rule with
  `discard_identity`.
- **Dashboard:** pass the free-variable settings its form already collects through to
  `PyFontanaGen`.
- **Docs:** describe the new fields in the README / config docs.
