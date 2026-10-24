# Recursive experiments from Python

This interface targets the `dev` API, not the current `main` API. It calls the
existing Rust generation loop; it does not reimplement that loop in Python.

```python
import json
import alchemy

config = json.loads(alchemy.PyConfig().to_json())
config.update(run_limit=2, sample_size=4, polling_interval=1)
config["recursive_config"].update(n_generations=2, wipeout_percent=25)

# No explicit population: seed with the configured native generator, as the CLI does.
generations = alchemy.simulate_recursive_experiment(json.dumps(config))

# Or supply the complete population, including repeated expressions/counts.
# This population replaces generated initial seeding; sample_size is not used.
generations = alchemy.simulate_recursive_experiment(
    json.dumps(config), initial_expressions=[r"\x.x"] * 4
)
for generation in generations:
    print(generation.generation, len(generation.reactions))
```

The example is usage documentation, not a captured successful run.

## Contract

`simulate_recursive_experiment(config_json, initial_expressions=None, *, polling_mode="legacy")` accepts
the same JSON schema as `PyConfig.to_json()` and the CLI. `initial_expressions`
is an optional list of expression strings, not a filename or a list of unique
species. With no list, the function explicitly seeds `sample_size` expressions
using the configured generator. It does not change `PySoup.from_config`.

The result is a list of `PyGenerationResult` objects in generation order:

- `generation`: one-based generation number.
- `history`: full population strings, including multiplicities, for each native
  polled snapshot. It is empty when `polling_interval` is null.
- `history_collisions`: completed collisions within this generation, paired
  one-for-one with `history`. These are independent of the legacy soup counter.
- `initial_expressions`: the exact full population before this generation runs.
- `final_expressions`: the exact full population after this generation runs,
  before any inter-generation wipeout/refill. Both boundary lists preserve
  multiplicities and remain available with null polling or zero collisions.
  They use the same Classic expression text as `history` and can be passed back
  as `initial_expressions` for a later run.
- `reactions`: the generation's `PyReactionRecord` entries, including unsuccessful
  reactions. `step` is the existing native zero-based, per-generation step.

The wrapper uses `RecordingType::All`. Result memory therefore grows with the
collision/generation counts and, when polling, the population sizes. Recorded
Python recursive runs also retain two boundary populations per generation; the
existing public Rust recursive API and ordinary `PySoup.simulate_for` calls do
not copy these boundary populations. The public Rust `Tape` shape and function
signatures remain unchanged. Use small
run limits when initially testing an integration. Native console output remains
unchanged. This API is synchronous; it does not supply progress callbacks.

Malformed JSON, invalid initial expressions, returned native parsing errors,
zero generations, wipeout percentages outside 0..100, zero polling intervals,
and initial populations smaller than two are reported as `ValueError`.
This is not comprehensive validation of all existing generator/reactor settings;
upstream engine failures are not converted into successful empty results.

The native operation releases the GIL using PyO3 0.23's `Python::allow_threads`.
The closure owns Rust configuration, expressions, and result values only; it
does not access Python objects or invoke callbacks. Thread behavior still needs
native verification.

## Dashboard integration boundary

| Current dashboard value | Native input |
| --- | --- |
| `num_generations` | `recursive_config.n_generations` |
| `total_collisions` | `run_limit` |
| `polling_frequency` | `polling_interval` |
| Fresh generator controls | `expression.UntypedLambda.generator` config |
| Full parent final population or manual/uploaded population | `initial_expressions` |
| `random_seed` / inherited parent seed | Existing hex encoding, then native soup/generator seed fields |
| Future extinction control | `recursive_config.wipeout_percent` (integer percent, not a 0..1 fraction) |
| Refill policy | `recursive_config.refill_type` and `repopulation_expression` |

This is not a drop-in replacement for `run_experiment`'s dashboard dictionary.
The dashboard still needs an adapter for its `metrics`, configuration IDs,
lineage, initial/final population persistence, and collision labels.

Native polled `history` is preserved, including its existing sampling convention
and any repeated final snapshot. For example, a three-collision run with polling
interval two has snapshot collision coordinates `[1, 3, 3]`, while a two-collision
run has only `[1]`. An adapter must use `history_collisions` rather than infer
coordinates from list indices, and use the explicit boundary populations rather
than assume that the last polled snapshot is final. Null polling yields empty
history/coordinates; a zero polling interval remains invalid. The legacy global
soup collision counter and `Tape.final_state()` semantics are unchanged.

For Dashboard integration, explicitly pass `polling_mode="completed"`. This
opt-in records interval multiples and the final collision without duplicates:
three collisions with interval two yield `[2, 3]`. The explicit initial boundary
supplies collision zero. Null polling remains empty, and a zero-collision run
has only its boundary population. The default `"legacy"` mode and public Rust
APIs preserve the inherited sampling behavior. Unknown modes are rejected.
Custom refill expressions for nonzero wipeout are parsed before simulation starts.

The metadata now supplies the exact population boundaries and snapshot labels
needed by a persistence adapter, but this change does not implement that adapter
or validate a running dashboard.
No dashboard dependency is switched from `main` to `dev`, and no existing
seeding or snapshot selection policy is changed here. Alchemy-Dashboard #45 remains open.

## Native tests

With the extension built from the patched `dev` checkout and available in the
selected Python environment:

```sh
python -m unittest discover -s tests -v
```

These tests import the real extension and use no native stubs. They cover
generated and supplied populations, per-generation records, a 100% native
wipeout/refill transition, disabled polling, and invalid inputs. Boundary tests
add deterministic growing populations, divisor/non-divisor snapshot coordinates,
zero collisions, and exact pre/post-wipeout populations. They must fail
at import when the native extension is unavailable; an import failure is not a
regression result.
