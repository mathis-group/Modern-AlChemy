"""Native integration tests: require the built alchemy extension, not a stub."""

import json
import unittest
from collections import Counter

import alchemy


def small_config():
    config = json.loads(alchemy.PyConfig().to_json())
    config.update(run_limit=2, sample_size=4, polling_interval=1)
    config["recursive_config"].update(n_generations=2, wipeout_percent=0)
    return config


class RecursiveBindingTests(unittest.TestCase):
    def test_generated_population_returns_each_generation(self):
        config = small_config()
        generations = alchemy.simulate_recursive_experiment(json.dumps(config))

        self.assertEqual([item.generation for item in generations], [1, 2])
        for item in generations:
            self.assertEqual(len(item.reactions), config["run_limit"])
            self.assertEqual(len(item.history), config["run_limit"])
            self.assertTrue(all(len(population) == 4 for population in item.history))
            self.assertEqual([r.step for r in item.reactions], [0, 1])
            self.assertTrue(all(isinstance(r.success, bool) for r in item.reactions))

    def test_explicit_population_and_wipeout_use_native_loop(self):
        config = small_config()
        config.update(run_limit=1, sample_size=0)
        reactor = config["expression"]["UntypedLambda"]["reactor"]
        reactor["rules"] = [r"\x.\y.x"]
        reactor["discard_copy_actions"] = True
        replacement = r"\x.\y.x"
        config["recursive_config"].update(
            wipeout_percent=100,
            refill_type="CustomExpression",
            repopulation_expression=replacement,
        )
        canonical = alchemy.PySoup()
        canonical.perturb([replacement])
        expected = canonical.expressions()[0]

        generations = alchemy.simulate_recursive_experiment(
            json.dumps(config), initial_expressions=[r"\x.x"] * 4
        )

        self.assertEqual(len(generations), 2)
        self.assertEqual(Counter(generations[1].history[-1]), Counter([expected] * 4))
        self.assertNotEqual(generations[0].history[-1], generations[1].history[-1])

    def test_single_generation_without_polling_keeps_reactions(self):
        config = small_config()
        config["recursive_config"]["n_generations"] = 1
        config["polling_interval"] = None
        generations = alchemy.simulate_recursive_experiment(json.dumps(config))
        self.assertEqual(len(generations), 1)
        self.assertEqual(generations[0].history, [])
        self.assertEqual(len(generations[0].reactions), 2)

    def test_malformed_json_and_invalid_expression_are_value_errors(self):
        with self.assertRaises(ValueError):
            alchemy.simulate_recursive_experiment("{")
        with self.assertRaises(ValueError):
            alchemy.simulate_recursive_experiment(
                json.dumps(small_config()), initial_expressions=["(", r"\x.x"]
            )

    def test_unsafe_recursive_arguments_are_rejected(self):
        cases = (
            ("n_generations", 0),
            ("wipeout_percent", 101),
            ("polling_interval", 0),
            ("sample_size", 1),
        )
        for field, value in cases:
            with self.subTest(field=field):
                config = small_config()
                target = config["recursive_config"] if field in (
                    "n_generations", "wipeout_percent"
                ) else config
                target[field] = value
                with self.assertRaises(ValueError):
                    alchemy.simulate_recursive_experiment(json.dumps(config))
        with self.assertRaises(ValueError):
            alchemy.simulate_recursive_experiment(
                json.dumps(small_config()), initial_expressions=[]
            )


if __name__ == "__main__":
    unittest.main()
