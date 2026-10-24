"""Exact generation boundaries from the real native recursive experiment."""

import json
import unittest
from collections import Counter

import alchemy


IDENTITY = r"\x.x"
LEFT = r"\x.\y.x"


def canonical(expression):
    soup = alchemy.PySoup()
    soup.perturb([expression])
    return soup.expressions()[0]


def growing_config(collisions=2, polling=2, generations=1):
    config = json.loads(alchemy.PyConfig().to_json())
    config.update(run_limit=collisions, sample_size=0, polling_interval=polling)
    config["soup_config"].update(
        discard_parents=False, maintain_constant_population_size=False
    )
    config["expression"]["UntypedLambda"]["reactor"].update(
        rules=[LEFT], discard_copy_actions=False, discard_identity=False
    )
    config["recursive_config"].update(n_generations=generations, wipeout_percent=0)
    return config


class GenerationBoundaryTests(unittest.TestCase):
    def run_native(self, config):
        return alchemy.simulate_recursive_experiment(
            json.dumps(config), initial_expressions=[IDENTITY] * 4
        )

    def assert_metadata_present(self, generation):
        for name in ("initial_expressions", "final_expressions", "history_collisions"):
            self.assertTrue(
                hasattr(generation, name),
                f"native generation result must expose exact {name}",
            )

    def test_divisor_poll_keeps_legacy_history_but_exposes_true_final(self):
        generation = self.run_native(growing_config())[0]
        # A left-parent product is accepted at every collision, so population
        # size proves which collision each snapshot actually follows.
        self.assertEqual([len(p) for p in generation.history], [5])
        self.assert_metadata_present(generation)
        self.assertEqual(generation.history_collisions, [1])
        self.assertEqual(Counter(generation.initial_expressions), Counter({canonical(IDENTITY): 4}))
        self.assertEqual(Counter(generation.final_expressions), Counter({canonical(IDENTITY): 6}))

    def test_non_divisor_records_actual_coordinates_without_changing_history(self):
        generation = self.run_native(growing_config(collisions=3))[0]
        # Preserve the inherited repeated last snapshot; its coordinates must
        # be truthful so a persistence adapter can de-duplicate collision 3.
        self.assertEqual([len(p) for p in generation.history], [5, 7, 7])
        self.assert_metadata_present(generation)
        self.assertEqual(generation.history_collisions, [1, 3, 3])
        self.assertEqual(len(generation.final_expressions), 7)

    def test_disabled_polling_has_no_snapshots_but_keeps_exact_boundaries(self):
        generation = self.run_native(growing_config(polling=None))[0]
        self.assertEqual(generation.history, [])
        self.assert_metadata_present(generation)
        self.assertEqual(generation.history_collisions, [])
        self.assertEqual(len(generation.initial_expressions), 4)
        self.assertEqual(len(generation.final_expressions), 6)

    def test_zero_collision_run_has_equal_boundaries_without_fabricated_polls(self):
        generation = self.run_native(growing_config(collisions=0))[0]
        self.assertEqual(generation.history, [])
        self.assertEqual(generation.reactions, [])
        self.assert_metadata_present(generation)
        self.assertEqual(generation.history_collisions, [])
        self.assertEqual(Counter(generation.initial_expressions), Counter({canonical(IDENTITY): 4}))
        self.assertEqual(generation.final_expressions, generation.initial_expressions)

    def test_boundaries_are_captured_before_and_after_native_wipeout(self):
        config = growing_config(generations=2)
        config["recursive_config"].update(
            wipeout_percent=100,
            refill_type="CustomExpression",
            repopulation_expression=LEFT,
        )
        first, second = self.run_native(config)
        self.assertEqual([len(p) for p in first.history], [5])
        self.assertEqual([len(p) for p in second.history], [7])
        self.assert_metadata_present(first)
        self.assert_metadata_present(second)
        self.assertEqual(Counter(first.initial_expressions), Counter({canonical(IDENTITY): 4}))
        self.assertEqual(Counter(first.final_expressions), Counter({canonical(IDENTITY): 6}))
        self.assertEqual(Counter(second.initial_expressions), Counter({canonical(LEFT): 6}))
        self.assertEqual(Counter(second.final_expressions), Counter({canonical(LEFT): 8}))
        self.assertEqual(first.history_collisions, [1])
        self.assertEqual(second.history_collisions, [1])

    def test_final_population_round_trips_as_provided_population(self):
        first = self.run_native(growing_config())[0]
        continued = alchemy.simulate_recursive_experiment(
            json.dumps(growing_config(collisions=0, polling=None)),
            initial_expressions=first.final_expressions,
        )[0]
        self.assertEqual(continued.initial_expressions, first.final_expressions)
        self.assertEqual(continued.final_expressions, first.final_expressions)

    def test_existing_direct_soup_simulation_methods_keep_their_results(self):
        for method in ("simulate_for", "simulate_for_logged", "simulate_for_logged_filtered"):
            with self.subTest(method=method):
                config = alchemy.PyConfig.from_json(json.dumps(growing_config()))
                soup = alchemy.PySoup.from_config(config)
                soup.perturb([IDENTITY] * 4)
                result = getattr(soup, method)(2, False) if method == "simulate_for" else getattr(soup, method)(2)
                if method == "simulate_for":
                    self.assertEqual(result, 2)
                else:
                    self.assertEqual([reaction.step for reaction in result], [0, 1])
                    self.assertTrue(all(reaction.success for reaction in result))
                self.assertEqual(Counter(soup.expressions()), Counter({canonical(IDENTITY): 6}))

    def test_zero_polling_interval_remains_invalid_not_disabled(self):
        with self.assertRaises(ValueError):
            self.run_native(growing_config(polling=0))


if __name__ == "__main__":
    unittest.main()
