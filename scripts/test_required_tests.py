import subprocess
import unittest
from unittest.mock import patch

from required_tests import run_required_tests


class RequiredTestsTest(unittest.TestCase):
    def test_empty_selection_does_not_run_tests(self):
        with patch("required_tests.subprocess.run") as run:
            run.return_value = subprocess.CompletedProcess([], 0, "0 tests, 0 benchmarks\n")
            self.assertEqual(run_required_tests(["cargo", "test", "missing", "--"]), 1)
            self.assertEqual(run.call_count, 1)

    def test_selected_tests_preserve_command_and_failure(self):
        command = ["cargo", "test", "example", "--", "--ignored", "--exact"]
        for exit_code in [0, 101]:
            with self.subTest(exit_code=exit_code), patch("required_tests.subprocess.run") as run:
                run.side_effect = [
                    subprocess.CompletedProcess([], 0, "example: test\n"),
                    subprocess.CompletedProcess([], exit_code),
                ]
                self.assertEqual(run_required_tests(command), exit_code)
                self.assertEqual(run.call_args_list[0].args[0],
                                 [*command, "--list", "--format", "terse"])
                self.assertEqual(run.call_args_list[1].args[0], command)

    def test_listing_failure_does_not_run_tests(self):
        with patch("required_tests.subprocess.run") as run:
            run.return_value = subprocess.CompletedProcess([], 101, "")
            self.assertEqual(run_required_tests(["cargo", "test", "--"]), 101)
            self.assertEqual(run.call_count, 1)


if __name__ == "__main__":
    unittest.main()
