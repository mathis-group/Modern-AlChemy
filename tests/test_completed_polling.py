"""Opt-in Dashboard cadence through the real native extension."""
import json
import subprocess
import sys
import unittest

import alchemy
from test_generation_boundaries import IDENTITY, growing_config


class CompletedPollingTests(unittest.TestCase):
    def run_native(self, collisions, polling=2, mode="completed"):
        return alchemy.simulate_recursive_experiment(
            json.dumps(growing_config(collisions=collisions, polling=polling)),
            initial_expressions=[IDENTITY] * 4,
            polling_mode=mode,
        )[0]

    def test_completed_polls_match_actual_population_at_multiples_and_final(self):
        for collisions, expected in ((0, []), (1, [1]), (2, [2]), (3, [2, 3]), (5, [2, 4, 5])):
            with self.subTest(collisions=collisions):
                result = self.run_native(collisions)
                self.assertEqual(result.history_collisions, expected)
                self.assertEqual([len(p) for p in result.history], [4 + n for n in expected])
                self.assertEqual(len(result.initial_expressions), 4)
                self.assertEqual(len(result.final_expressions), 4 + collisions)

    def test_explicit_legacy_mode_preserves_duplicate_last_poll(self):
        result = self.run_native(3, mode="legacy")
        self.assertEqual(result.history_collisions, [1, 3, 3])
        self.assertEqual([len(p) for p in result.history], [5, 7, 7])

    def test_completed_mode_with_disabled_polling_keeps_only_boundaries(self):
        result = self.run_native(3, polling=None)
        self.assertEqual(result.history, [])
        self.assertEqual(result.history_collisions, [])
        self.assertEqual(len(result.final_expressions), 7)

    def test_unknown_polling_mode_is_rejected(self):
        with self.assertRaises(ValueError):
            self.run_native(2, mode="approximate")

    def test_invalid_refill_fails_before_native_generation_starts(self):
        config = growing_config(collisions=0, generations=2)
        config["recursive_config"].update(
            wipeout_percent=100, refill_type="CustomExpression", repopulation_expression="("
        )
        code = """import alchemy,json,sys
try:
 alchemy.simulate_recursive_experiment(sys.argv[1], initial_expressions=[r'\\x.x'] * 4)
except ValueError as error:
 print('REJECTED', error)
else:
 raise SystemExit('invalid refill accepted')
"""
        run = subprocess.run([sys.executable, "-B", "-c", code, json.dumps(config)], capture_output=True, text=True, timeout=10)
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertIn("REJECTED", run.stdout)
        self.assertNotIn("Generation", run.stdout)


if __name__ == "__main__":
    unittest.main()
