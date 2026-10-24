"""Progress and GIL behavior through the real native recursive binding."""

import json
import subprocess
import sys
import unittest

import alchemy
from test_generation_boundaries import IDENTITY, growing_config


class RecursiveProgressTests(unittest.TestCase):
    def progress(self):
        self.assertTrue(
            hasattr(alchemy, "PyExperimentProgress"),
            "recursive experiments must expose live collision progress",
        )
        return alchemy.PyExperimentProgress()

    def test_progress_counts_unsuccessful_collisions_across_generations(self):
        progress = self.progress()
        config = growing_config(collisions=3, polling=None, generations=2)
        config["expression"]["UntypedLambda"]["reactor"]["discard_copy_actions"] = True
        results = alchemy.simulate_recursive_experiment(
            json.dumps(config), initial_expressions=[IDENTITY] * 4, progress=progress
        )
        self.assertTrue(all(not r.success for g in results for r in g.reactions))
        self.assertEqual(progress.completed_collisions, 6)
        self.assertFalse(progress.running)

    def test_reusing_progress_resets_the_count_including_zero_collision_runs(self):
        progress = self.progress()
        for collisions, expected in ((2, 4), (0, 0), (1, 2)):
            alchemy.simulate_recursive_experiment(
                json.dumps(growing_config(collisions=collisions, generations=2)),
                initial_expressions=[IDENTITY] * 4,
                progress=progress,
            )
            self.assertEqual(progress.completed_collisions, expected)
            self.assertFalse(progress.running)

    def test_native_parse_failure_releases_progress_for_another_run(self):
        progress = self.progress()
        config = json.dumps(growing_config(collisions=1))
        with self.assertRaises(ValueError):
            alchemy.simulate_recursive_experiment(
                config, initial_expressions=["(", IDENTITY], progress=progress
            )
        self.assertFalse(progress.running)
        self.assertEqual(progress.completed_collisions, 0)
        alchemy.simulate_recursive_experiment(
            config, initial_expressions=[IDENTITY] * 4, progress=progress
        )
        self.assertEqual(progress.completed_collisions, 1)

    def test_python_can_poll_live_native_progress_and_overlapping_reuse_is_rejected(self):
        self.progress()
        # A subprocess bounds a GIL deadlock/regression without leaving a hung
        # test runner. Actual collisions, rather than elapsed time, synchronize
        # the reader with the native worker.
        code = r'''
import json, threading, time
import alchemy

config = json.loads(alchemy.PyConfig().to_json())
config.update(run_limit=100000, sample_size=0, polling_interval=None)
config["recursive_config"].update(n_generations=2, wipeout_percent=0)
config["expression"]["UntypedLambda"]["reactor"].update(
    rules=[r"\x.\y.x"], discard_copy_actions=True
)
progress = alchemy.PyExperimentProgress()
finished = threading.Event()
errors = []
def run():
    try:
        alchemy.simulate_recursive_experiment(
            json.dumps(config), initial_expressions=[r"\x.x"] * 4, progress=progress
        )
    except BaseException as error:
        errors.append(error)
    finally:
        finished.set()

worker = threading.Thread(target=run)
worker.start()
observed = []
deadline = time.monotonic() + 15
while not finished.is_set() and time.monotonic() < deadline:
    count = progress.completed_collisions
    if progress.running and 0 < count < 100000:
        observed.append(count)
        if len(observed) == 1:
            try:
                alchemy.simulate_recursive_experiment(
                    json.dumps(config), initial_expressions=[r"\x.x"] * 4,
                    progress=progress
                )
            except ValueError:
                pass
            else:
                raise AssertionError("overlapping progress reuse accepted")
            separate = alchemy.PyExperimentProgress()
            short_config = dict(config, run_limit=3)
            alchemy.simulate_recursive_experiment(
                json.dumps(short_config), initial_expressions=[r"\x.x"] * 4,
                progress=separate
            )
            assert separate.completed_collisions == 6
            assert not separate.running
    time.sleep(0.001)
worker.join(timeout=15)
assert not worker.is_alive(), "native run did not finish"
assert not errors, errors
assert observed, "Python could not read intermediate native collision progress"
assert observed == sorted(observed), observed
assert progress.completed_collisions == 200000, progress.completed_collisions
assert not progress.running
print("LIVE_PROGRESS_OK")
'''
        run = subprocess.run(
            [sys.executable, "-B", "-c", code],
            capture_output=True, text=True, timeout=35,
        )
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        self.assertIn("LIVE_PROGRESS_OK", run.stdout)


if __name__ == "__main__":
    unittest.main()
