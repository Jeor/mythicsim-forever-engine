from pathlib import Path
import json
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import job_bench


class ReportTest(unittest.TestCase):
    def run_report(self, *options, refuse_last=False):
        with tempfile.TemporaryDirectory() as temp:
            folder = Path(temp)
            bundle = folder / "bundle"
            (bundle / "bin").mkdir(parents=True)
            (bundle / "tools").mkdir()
            (bundle / "manifest.json").write_text('{"synthetic": true}')
            if refuse_last:
                (bundle / "refuse-last").touch()
            # Stand in for the external bundle, leaving the benchmark and its process
            # measurements real. These timings are not engine performance evidence.
            program = f"#!{sys.executable}\n" + '''
import json
from pathlib import Path
import sys
args = sys.argv[1:]
def argument(name):
    return Path(args[args.index(name) + 1])
report = {"elapsedNs": 123456789, "elapsed_ns": 123456789,
          "raidMetrics": {"dps": {"avg": 123.25}},
          "result": {"raidMetrics": {"dps": {"avg": 123.25}}}}
if "--output" in args:
    folder = argument("--output")
    folder.mkdir(parents=True)
    (folder / "request.json").write_text(argument("--request").read_text())
    refused = ((Path(__file__).resolve().parents[1] / "refuse-last").exists()
               and folder.parent.name == "1"
               and folder.parent.parent.name == "production-destruction-warlock")
    (folder / "decision.json").write_text(json.dumps(
        {"status": "fallback" if refused else "rust", "preparation": {"provider": "rust"}}))
    (folder / "rust-report.json").write_text(json.dumps(report))
else:
    argument("--outfile").write_text(json.dumps(report))
'''
            for name in ("bin/forever-engine", "bin/forever-go-oracle-v2", "tools/route.py"):
                path = bundle / name
                path.write_text(program)
                path.chmod(0o755)
            output = folder / "report.json"
            done = subprocess.run(
                [sys.executable, str(Path(job_bench.__file__)), "--bundle", str(bundle),
                 "--output", str(output), "--rounds", "2", *options,
                 "production-destruction-warlock", "production-protection-paladin"],
                capture_output=True, text=True)
            self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
            return json.loads(output.read_text())

    def assert_measurement(self, sample):
        self.assertEqual(sample["status"], 0)
        for key in ("wall_s", "cpu_s", "peak_rss_mb"):
            self.assertIsInstance(sample[key], (int, float))
            self.assertGreaterEqual(sample[key], 0)

    def test_report_retains_each_round_and_step(self):
        row = self.run_report()["rows"][0]
        self.assertEqual([sample["round"] for sample in row["samples"]], [0, 1])
        for sample in row["samples"]:
            for engine in ("go", "rust"):
                self.assert_measurement(sample[engine])
                self.assertEqual(sample[engine]["dps"], 123.25)
                self.assertEqual(sample[engine]["engine_s"], 0.123456789)
            self.assertEqual(sample["rust"]["decision"], "rust")
            self.assertEqual(sample["rust"]["preparation"], "rust")
            self.assertEqual(set(sample["rust_steps"]), {"go_prepare", "rust_prepare", "sim"})
            for step in sample["rust_steps"].values():
                self.assert_measurement(step)
        for engine in ("go", "rust"):
            for key, digits in (("wall_s", 4), ("cpu_s", 4), ("peak_rss_mb", 1), ("engine_s", 4)):
                values = [sample[engine][key] for sample in row["samples"]]
                self.assertEqual(row[engine][key], round((values[0] + values[1]) / 2, digits))
        for step in ("go_prepare", "rust_prepare", "sim"):
            for key, digits in (("wall_s", 4), ("cpu_s", 4), ("peak_rss_mb", 1)):
                values = [sample["rust_steps"][step][key] for sample in row["samples"]]
                self.assertEqual(row["rust_steps"][step][key],
                                 round((values[0] + values[1]) / 2, digits))
        self.assertTrue(row["same_dps"])
        self.assertEqual(row["go_over_rust_wall"], round(row["go"]["wall_s"] / row["rust"]["wall_s"], 2))

    def test_refused_round_retains_its_decision_without_steps(self):
        row = self.run_report(refuse_last=True)["rows"][0]
        self.assertEqual(row["samples"][0]["round"], 0)
        self.assertIsNotNone(row["samples"][0]["rust_steps"])
        refused = row["samples"][1]
        self.assertEqual(refused["round"], 1)
        self.assertEqual(refused["rust"]["decision"], "fallback")
        self.assert_measurement(refused["rust"])
        self.assertIsNone(refused["rust_steps"])

    def test_throughput_retains_every_case_and_round(self):
        report = self.run_report("--concurrency", "2")
        expected = [(case, index) for index in (0, 1) for case in
                    ("production-destruction-warlock", "production-protection-paladin")]
        for engine in ("go", "rust"):
            result = report["throughput"][engine]
            self.assertEqual(result["jobs"], 4)
            self.assertEqual([(sample["case"], sample["round"]) for sample in result["samples"]],
                             expected)
            for sample in result["samples"]:
                self.assert_measurement(sample["measurement"])
                self.assertEqual(sample["measurement"]["dps"], 123.25)
                self.assertEqual(sample["measurement"]["engine_s"], 0.123456789)
            walls = sorted(sample["measurement"]["wall_s"] for sample in result["samples"])
            self.assertEqual(result["median_job_wall_s"], round((walls[1] + walls[2]) / 2, 3))
            self.assertEqual(result["peak_rss_mb"], round(max(
                sample["measurement"]["peak_rss_mb"] for sample in result["samples"]), 1))
            self.assertGreater(result["jobs_per_minute"], 0)
            self.assertGreater(result["wall_s"], 0)


class MeasureTest(unittest.TestCase):
    def test_a_process_reports_its_time_memory_and_status(self):
        # A child that holds about 50 MB and spends a little CPU before it fails.
        code = "b = bytearray(50 * 2**20); sum(range(10**6)); raise SystemExit(3)"
        sample = job_bench.measure([sys.executable, "-c", code])
        self.assertEqual(sample["status"], 3)
        self.assertGreater(sample["peak_rss_mb"], 45)
        self.assertGreater(sample["cpu_s"], 0)
        self.assertGreaterEqual(sample["wall_s"], sample["cpu_s"] * 0.5)

    def test_memory_includes_the_processes_a_job_waits_for(self):
        child = "b = bytearray(80 * 2**20)"
        parent = f"import subprocess, sys; subprocess.run([sys.executable, '-c', {child!r}])"
        self.assertGreater(job_bench.measure([sys.executable, "-c", parent])["peak_rss_mb"], 75)

    def test_summaries_are_medians(self):
        samples = [{"wall_s": 1.0, "peak_rss_mb": 10.0}, {"wall_s": 3.0, "peak_rss_mb": 30.0},
                   {"wall_s": 2.0, "peak_rss_mb": 20.0}]
        self.assertEqual(job_bench.summarize(samples), {"wall_s": 2.0, "peak_rss_mb": 20.0})


if __name__ == "__main__":
    unittest.main()
