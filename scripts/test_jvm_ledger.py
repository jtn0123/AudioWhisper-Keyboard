"""Unit tests for scripts/jvm-ledger.py: the counted-line rule, then the ledger's checks.

Run with: python3 -I -B -m unittest discover -s scripts -p test_jvm_ledger.py
"""
import contextlib
import importlib.util
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
_SPEC = importlib.util.spec_from_file_location("jvm_ledger", Path(__file__).resolve().parent / "jvm-ledger.py")
jl = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(jl)


class CountedLinesTest(unittest.TestCase):
    def count(self, source):
        return jl.count_code_lines(source)

    def test_blank_and_whitespace_lines_do_not_count(self):
        self.assertEqual(self.count(""), 0)
        self.assertEqual(self.count("   \n\t\n  \t "), 0)

    def test_line_comment_does_not_count(self):
        self.assertEqual(self.count("// hi\n    // there"), 0)

    def test_code_before_a_line_comment_counts(self):
        self.assertEqual(self.count("val x = 1 // note"), 1)

    def test_one_line_block_comment_does_not_count(self):
        self.assertEqual(self.count("/* a */"), 0)

    def test_code_after_a_block_comment_counts(self):
        self.assertEqual(self.count("/* a */ x"), 1)

    def test_block_comment_spanning_lines_counts_only_its_code(self):
        self.assertEqual(self.count("x /* a\nb */\nc"), 2)

    def test_kdoc_does_not_count(self):
        self.assertEqual(self.count("/**\n * doc\n */\nfun f()"), 1)

    def test_nested_block_comment_stays_open_until_the_outer_one_closes(self):
        self.assertEqual(self.count("/* a /* b */ still\n c */ d"), 1)
        self.assertEqual(self.count("/* a /* b */\nc\n*/\nd"), 1)

    def test_slash_star_slash_does_not_close_a_block_comment(self):
        self.assertEqual(self.count("/*/ x */ y"), 1)

    def test_unterminated_block_comment_runs_to_end_of_file(self):
        self.assertEqual(self.count("/* x\ny\nz"), 0)

    def test_comment_markers_in_a_string_are_code(self):
        self.assertEqual(self.count('val u = "http://x"'), 1)
        self.assertEqual(self.count('val s = "/*"\nval t = 1'), 2)

    def test_escaped_quote_keeps_the_string_open(self):
        self.assertEqual(self.count('val s = "a\\"//b"\n// c'), 1)

    def test_escaped_backslash_closes_the_string(self):
        self.assertEqual(self.count('val s = "\\\\"\nval t = 1'), 2)

    def test_empty_string_is_code(self):
        self.assertEqual(self.count('val e = ""'), 1)

    def test_raw_string_contents_are_code_even_when_comment_like(self):
        self.assertEqual(self.count('val r = """\n  // not\n  /* nor */\n"""'), 4)

    def test_blank_line_inside_a_raw_string_does_not_count(self):
        self.assertEqual(self.count('val r = """\n\n"""'), 2)

    def test_raw_string_quotes_inside_and_an_extra_closing_quote(self):
        self.assertEqual(self.count('val r = """say "hi" now"""'), 1)
        self.assertEqual(self.count('val r = """x""""\nval y = 2'), 2)

    def test_raw_string_with_a_triple_quote_run_closes_at_the_last_three(self):
        self.assertEqual(self.count('val r = """a""""""\n// c'), 1)

    def test_comment_marker_inside_a_template_string_is_code(self):
        self.assertEqual(self.count('val s = "${ "//" } b"\n// c'), 1)

    def test_closing_brace_inside_a_nested_string_does_not_end_the_template(self):
        self.assertEqual(self.count('val s = "${ "}" }"\nval t = 2'), 2)

    def test_braces_inside_a_template_are_balanced(self):
        self.assertEqual(self.count('val s = "${ if (x) { 1 } else { 2 } }"\nval n = 1'), 2)

    def test_block_comment_inside_a_template_expression(self):
        self.assertEqual(self.count('val s = "${ /* } */ 1 }"'), 1)

    def test_quote_inside_a_block_comment_does_not_start_a_string(self):
        self.assertEqual(self.count('/* " */ x\nval s = "y"'), 2)

    def test_char_literals_hold_quote_slash_and_star(self):
        self.assertEqual(self.count("val c = '\"'\nval d = 1"), 2)
        self.assertEqual(self.count("val s = '/' // c"), 1)
        self.assertEqual(self.count("val s = '*'"), 1)

    def test_char_escapes(self):
        self.assertEqual(self.count("val c = '\\n'\nval d = '\\u0041'"), 2)
        self.assertEqual(self.count("val c = '\\''\nval d = 1"), 2)

    def test_brace_only_line_counts(self):
        self.assertEqual(self.count("}"), 1)

    def test_byte_order_mark_and_crlf_are_whitespace(self):
        self.assertEqual(self.count("\ufeff\n\ufeffx"), 1)
        self.assertEqual(self.count("a\r\n\r\n// b\r\n"), 1)

    def test_code_flags_gives_one_flag_per_line(self):
        self.assertEqual(jl.code_flags("a\n\n// b\n/* c\n*/\nd"), [True, False, False, False, False, True])

    def test_block_comment_inside_a_string_is_not_a_block_comment(self):
        self.assertEqual(self.count('val s = "/* not"\n*/ x'), 2)



HEADER = "path\tclass\tbudget\tnote\n"


def ledger_text(*rows):
    return HEADER + "".join(f"{path}\t{cls}\t{budget}\t{note}\n" for path, cls, budget, note in rows)


class ParseLedgerTest(unittest.TestCase):
    def test_valid_rows_parse(self):
        rows, errors = jl.parse_ledger(ledger_text(("a/A.kt", "A", 3, "adapter")))
        self.assertEqual(errors, [])
        self.assertEqual(rows, [jl.Row("a/A.kt", "A", 3, "adapter")])

    def test_zero_budget_is_allowed(self):
        rows, errors = jl.parse_ledger(ledger_text(("a/A.kt", "A", 0, "empty")))
        self.assertEqual((len(rows), errors), (1, []))

    def test_unknown_class_is_an_error(self):
        rows, errors = jl.parse_ledger(ledger_text(("a/A.kt", "Z", 3, "what")))
        self.assertEqual(rows, [])
        self.assertIn("unknown class", errors[0])

    def test_malformed_budgets_are_errors(self):
        for budget in ["01", "-1", "+1", "1.0", "x", "", " 2"]:
            with self.subTest(budget=budget):
                rows, errors = jl.parse_ledger(HEADER + f"a/A.kt\tA\t{budget}\tnote\n")
                self.assertEqual(rows, [])
                self.assertIn("malformed budget", errors[0])

    def test_rows_need_four_fields_and_a_note(self):
        for line in ["a/A.kt\tA\t3", "a/A.kt\tA\t3\t", "\tA\t3\tnote"]:
            with self.subTest(line=line):
                rows, errors = jl.parse_ledger(HEADER + line + "\n")
                self.assertEqual(rows, [])
                self.assertIn("malformed row", errors[0])

    def test_header_is_required(self):
        rows, errors = jl.parse_ledger("a/A.kt\tA\t3\tnote\n")
        self.assertIn("header", errors[0])
        self.assertEqual(rows, [])

    def test_format_sorts_by_path_and_round_trips(self):
        rows = [jl.Row("b/B.kt", "M", 2, "two"), jl.Row("a/A.kt", "A", 1, "one")]
        text = jl.format_ledger(rows)
        self.assertEqual(text, HEADER + "a/A.kt\tA\t1\tone\nb/B.kt\tM\t2\ttwo\n")
        self.assertEqual(jl.parse_ledger(text), (sorted(rows, key=lambda r: r.path), []))


class FindProblemsTest(unittest.TestCase):
    def test_matching_ledger_has_no_problems(self):
        rows = [jl.Row("a.kt", "A", 2, "x")]
        self.assertEqual(jl.find_problems(rows, [], {"a.kt": 2}), [])

    def test_each_mismatch_is_reported(self):
        rows = [
            jl.Row("grew.kt", "A", 1, "x"),
            jl.Row("shrank.kt", "A", 5, "x"),
            jl.Row("gone.kt", "A", 1, "x"),
            jl.Row("dup.kt", "A", 1, "x"),
            jl.Row("dup.kt", "A", 1, "x"),
        ]
        counts = {"grew.kt": 2, "shrank.kt": 2, "dup.kt": 1, "unlisted.kt": 4}
        problems = jl.find_problems(rows, ["bad row"], counts, missing=["lost.kt"])
        text = "\n".join(problems)
        self.assertIn("bad row", text)
        self.assertIn("tracked but missing from the working tree: lost.kt", text)
        self.assertIn("duplicate row: dup.kt", text)
        self.assertIn("grew: grew.kt has 2 counted lines, budget 1", text)
        self.assertIn("shrank: shrank.kt has 2 counted lines, budget 5", text)
        self.assertIn("no row: unlisted.kt has 4 counted lines", text)
        self.assertIn("row for a file that is not a production file", text)
        self.assertIn("gone.kt", text)
        self.assertEqual(len(problems), 7)


class TightenTest(unittest.TestCase):
    def test_lowers_over_generous_budgets(self):
        kept, changes = jl.tighten([jl.Row("a.kt", "A", 9, "x")], {"a.kt": 4})
        self.assertEqual(kept, [jl.Row("a.kt", "A", 4, "x")])
        self.assertEqual(changes, ["budget 9 -> 4: a.kt"])

    def test_never_raises_a_budget(self):
        kept, changes = jl.tighten([jl.Row("a.kt", "A", 1, "x")], {"a.kt": 4})
        self.assertEqual(kept, [jl.Row("a.kt", "A", 1, "x")])
        self.assertEqual(changes, [])

    def test_drops_rows_of_deleted_files_and_never_adds_rows(self):
        kept, changes = jl.tighten([jl.Row("gone.kt", "A", 1, "x")], {"new.kt": 3})
        self.assertEqual(kept, [])
        self.assertEqual(changes, ["removed row for a file that is gone: gone.kt"])


class ProductionFilterTest(unittest.TestCase):
    def test_only_src_main_outside_the_benchmark_module(self):
        self.assertTrue(jl.is_production("app/src/main/kotlin/A.kt"))
        self.assertTrue(jl.is_production("lib/rust/src/main/kotlin/B.kt"))
        self.assertFalse(jl.is_production("benchmark/src/main/kotlin/C.kt"))
        self.assertFalse(jl.is_production("lib/benchmark/src/main/kotlin/D.kt"))
        self.assertFalse(jl.is_production("app/src/test/kotlin/E.kt"))
        self.assertFalse(jl.is_production("app/src/androidTest/kotlin/F.kt"))
        self.assertFalse(jl.is_production("scripts/render-benchmark/G.java"))
        self.assertFalse(jl.is_production("app/src/main/kotlin/H.txt"))


class CliTest(unittest.TestCase):
    """main() on a temporary git repository, so the real git listing and file reads run."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.git("init", "-q")
        self.file("app/src/main/kotlin/A.kt", "val a = 1\n// note\n")
        self.file("app/src/main/kotlin/B.kt", "val b = 1\nval c = 2\n")
        self.file("benchmark/src/main/kotlin/Bench.kt", "val x = 1\n")
        self.file("app/src/test/kotlin/T.kt", "val t = 1\n")
        self.write_ledger(("app/src/main/kotlin/A.kt", "A", 1, "a"), ("app/src/main/kotlin/B.kt", "M", 2, "b"))

    def tearDown(self):
        self.tmp.cleanup()

    def git(self, *args):
        subprocess.run(["git", "-C", str(self.root), *args], check=True, capture_output=True)

    def file(self, path, text):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")
        self.git("add", path)

    def write_ledger(self, *rows):
        ledger = self.root / "docs" / "jvm-ledger.tsv"
        ledger.parent.mkdir(parents=True, exist_ok=True)
        ledger.write_text(ledger_text(*[(p, c, b, n) for p, c, b, n in rows]), encoding="utf-8")

    def run_main(self, *args):
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = jl.main(list(args), root=self.root)
        return code, out.getvalue(), err.getvalue()

    def ledger(self):
        return (self.root / "docs" / "jvm-ledger.tsv").read_text(encoding="utf-8")

    def test_matching_repository_passes_and_ignores_benchmark_and_tests(self):
        code, out, _ = self.run_main("--check")
        self.assertEqual(code, 0)
        self.assertIn("matches 2 production files and 2 rows", out)

    def test_growth_fails_with_the_hand_edit_message(self):
        self.file("app/src/main/kotlin/B.kt", "val b = 1\nval c = 2\nval d = 3\n")
        code, _, err = self.run_main("--check")
        self.assertEqual(code, 1)
        self.assertIn("grew: app/src/main/kotlin/B.kt has 3 counted lines, budget 2", err)
        self.assertIn("hand edit", err)

    def test_shrink_fails_then_tighten_fixes_it_and_passes(self):
        self.write_ledger(("app/src/main/kotlin/A.kt", "A", 5, "a"), ("app/src/main/kotlin/B.kt", "M", 2, "b"))
        code, _, err = self.run_main("--check")
        self.assertEqual(code, 1)
        self.assertIn("--tighten", err)
        code, out, _ = self.run_main("--tighten")
        self.assertEqual(code, 0)
        self.assertIn("budget 5 -> 1", out)
        self.assertIn("app/src/main/kotlin/A.kt\tA\t1\ta", self.ledger())
        self.assertEqual(self.run_main("--check")[0], 0)

    def test_tighten_does_not_accept_growth(self):
        self.write_ledger(("app/src/main/kotlin/A.kt", "A", 1, "a"), ("app/src/main/kotlin/B.kt", "M", 1, "b"))
        before = self.ledger()
        code, _, err = self.run_main("--tighten")
        self.assertEqual(code, 1)
        self.assertIn("grew: app/src/main/kotlin/B.kt", err)
        self.assertEqual(self.ledger(), before)

    def test_deleted_file_row_fails_then_tighten_removes_it(self):
        self.write_ledger(("app/src/main/kotlin/A.kt", "A", 1, "a"), ("app/src/main/kotlin/B.kt", "M", 2, "b"),
                          ("app/src/main/kotlin/Gone.kt", "A", 1, "gone"))
        self.assertEqual(self.run_main("--check")[0], 1)
        self.git("rm", "-q", "-f", "app/src/main/kotlin/B.kt")
        self.assertEqual(self.run_main("--check")[0], 1)
        self.assertEqual(self.run_main("--tighten")[0], 0)
        self.assertNotIn("Gone.kt", self.ledger())
        self.assertNotIn("B.kt", self.ledger())

    def test_new_unlisted_file_fails_and_tighten_does_not_add_it(self):
        self.file("app/src/main/kotlin/New.kt", "val n = 1\n")
        code, _, err = self.run_main("--check")
        self.assertEqual(code, 1)
        self.assertIn("no row: app/src/main/kotlin/New.kt", err)
        self.assertEqual(self.run_main("--tighten")[0], 1)
        self.assertNotIn("New.kt", self.ledger())

    def test_missing_tracked_file_fails_and_tighten_refuses_to_write(self):
        before = self.ledger()
        (self.root / "app/src/main/kotlin/A.kt").unlink()
        code, _, err = self.run_main("--check")
        self.assertEqual(code, 1)
        self.assertIn("tracked but missing", err)
        self.assertEqual(self.run_main("--tighten")[0], 1)
        self.assertEqual(self.ledger(), before)

    def test_malformed_ledger_fails_and_tighten_refuses_to_write(self):
        bad = HEADER + "app/src/main/kotlin/A.kt\tQ\t1\ta\n"
        (self.root / "docs" / "jvm-ledger.tsv").write_text(bad, encoding="utf-8")
        self.assertEqual(self.run_main("--check")[0], 1)
        self.assertEqual(self.run_main("--tighten")[0], 1)
        self.assertEqual(self.ledger(), bad)

    def test_missing_ledger_fails(self):
        (self.root / "docs" / "jvm-ledger.tsv").unlink()
        code, _, err = self.run_main("--check")
        self.assertEqual(code, 1)
        self.assertIn("docs/jvm-ledger.tsv is missing", err)

    def test_summary_lists_each_class(self):
        code, out, _ = self.run_main("--summary")
        self.assertEqual(code, 0)
        self.assertRegex(out, r"(?m)^A\s+1\s+1\s")
        self.assertRegex(out, r"(?m)^M\s+1\s+2\s")
        self.assertRegex(out, r"(?m)^total\s+2\s+3$")


if __name__ == "__main__":
    unittest.main()
