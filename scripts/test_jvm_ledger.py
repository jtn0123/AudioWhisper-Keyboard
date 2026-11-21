"""Unit tests for scripts/jvm-ledger.py: the counted-line rule, then the ledger's checks.

Run with: python3 -I -m unittest discover -s scripts -p test_jvm_ledger.py
"""
import importlib.util
from pathlib import Path
import sys
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


if __name__ == "__main__":
    unittest.main()
