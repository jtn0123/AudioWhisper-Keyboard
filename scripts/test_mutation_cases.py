"""Unit tests for scripts/mutation_cases.py: when a case applies, how the STALE list is held to
account, and the --check hook and full run built on them.

Run with: python3 -I -B -m unittest discover -s scripts -p test_mutation_cases.py
"""
import contextlib
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import types
import unittest

sys.dont_write_bytecode = True
SCRIPTS = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPTS))  # -I leaves the script's folder off the path
import mutation_cases as mc


class CaseTest(unittest.TestCase):
    """A temporary source file `a.rs`, and the problems found in edits to it."""

    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.addCleanup(self.folder.cleanup)
        self.file = Path(self.folder.name) / 'a.rs'
        self.file.write_text('let a = 1;\nlet b = 2;\nlet b = 2;\n')

    def edit(self, case='c', old='let a = 1;', new='let a = 0;', **fields):
        return mc.Edit(case, fields.pop('path', self.file), old, new, **fields)

    def found(self, *edits, stale=None):
        return mc.problems(list(edits), stale)


class ApplyTest(CaseTest):
    def test_a_search_text_found_once_applies(self):
        self.assertEqual(self.found(self.edit()), [])

    def test_a_search_text_found_nowhere_names_the_case_and_the_text(self):
        self.assertEqual(self.found(self.edit('gone', old='let c = 3;')),
                         [f"gone: {self.file}: 'let c = 3;' found 0 times, not 1"])

    def test_a_search_text_found_twice_does_not_apply_once(self):
        self.assertEqual(self.found(self.edit(old='let b = 2;', new='let b = 0;')),
                         [f"c: {self.file}: 'let b = 2;' found 2 times, not 1"])

    def test_a_case_expecting_two_matches_applies_to_two(self):
        self.assertEqual(self.found(self.edit(old='let b = 2;', new='let b = 0;', count=2)), [])

    def test_a_multi_line_search_text_is_shown_by_its_first_line(self):
        self.assertEqual(self.found(self.edit(old='\n  let z = 9;\nlet y = 8;\n')),
                         [f"c: {self.file}: 'let z = 9;' found 0 times, not 1"])

    def test_a_missing_file_is_named(self):
        missing = Path(self.folder.name) / 'gone.rs'
        self.assertEqual(self.found(self.edit(path=missing)), [f'c: {missing} does not exist'])

    def test_an_edit_that_changes_nothing_does_not_apply(self):
        self.assertEqual(self.found(self.edit(new='let a = 1;')), [f'c: {self.file}: it changes nothing'])

    def test_an_empty_search_text_does_not_apply(self):
        self.assertEqual(self.found(self.edit(old='')), [f'c: {self.file}: its search text is empty'])

    def test_an_empty_search_text_makes_a_file_that_does_not_exist(self):
        made = Path(self.folder.name) / 'made.rs'
        self.assertEqual(self.found(self.edit(path=made, old='', new='let m = 1;\n')), [])

    def test_a_made_file_is_edited_in_turn(self):
        made = Path(self.folder.name) / 'made.rs'
        self.assertEqual(self.found(self.edit(path=made, old='', new='let m = 1;\n'),
                                    self.edit(path=made, old='let m = 1;', new='let m = 2;')), [])

    def test_a_missing_file_with_nothing_to_make_is_named(self):
        missing = Path(self.folder.name) / 'gone.rs'
        self.assertEqual(self.found(self.edit(path=missing, old='', new='')), [f'c: {missing} does not exist'])

    def test_a_path_relative_to_the_checkout_is_read_from_it(self):
        path = Path('scripts/mutation_cases.py')
        self.assertEqual(self.found(self.edit(path=path, old='class Edit(NamedTuple):', new='class E:')), [])


class InTurnTest(CaseTest):
    def test_a_later_edit_is_found_in_the_text_the_earlier_one_left(self):
        self.assertEqual(self.found(self.edit(), self.edit(old='let a = 0;', new='let a = 5;')), [])

    def test_a_later_edit_whose_text_the_earlier_one_removed_does_not_apply(self):
        self.assertEqual(self.found(self.edit(), self.edit(old='let a = 1;', new='let a = 5;')),
                         [f"c: {self.file}: 'let a = 1;' found 0 times, not 1"])

    def test_each_case_starts_from_the_file_as_it_is(self):
        self.assertEqual(self.found(self.edit('one'), self.edit('two')), [])

    def test_every_wrong_edit_of_a_case_is_named_on_its_line(self):
        self.assertEqual(self.found(self.edit(old='x'), self.edit(old='y')),
                         [f"c: {self.file}: 'x' found 0 times, not 1; {self.file}: 'y' found 0 times, not 1"])


class ChangeTest(CaseTest):
    def test_a_change_that_applies_is_fine(self):
        self.assertEqual(self.found(self.edit(change=mc.replace_once('let a = 1;', 'let a = 0;'))), [])

    def test_a_change_that_asserts_is_reported_with_its_message(self):
        self.assertEqual(self.found(self.edit(change=mc.replace_once('let c = 3;', ''))),
                         [f"c: {self.file}: expected exactly one 'let c = 3;'"])

    def test_a_change_raising_a_value_error_is_reported(self):
        def change(source):
            return source[:source.index('let c')]
        self.assertEqual(self.found(self.edit(change=change)), [f'c: {self.file}: substring not found'])

    def test_a_change_that_changes_nothing_does_not_apply(self):
        self.assertEqual(self.found(self.edit(change=lambda source: source)),
                         [f'c: {self.file}: its change changes nothing'])


class StaleTest(CaseTest):
    def test_a_stale_case_that_still_does_not_apply_is_fine(self):
        self.assertEqual(self.found(self.edit('old', old='let c = 3;'), stale={'old': 'moved'}), [])

    def test_a_stale_case_that_applies_again_fails(self):
        self.assertEqual(self.found(self.edit('old'), stale={'old': 'moved'}),
                         ['old: applies again; take it out of STALE (moved)'])

    def test_a_stale_name_that_is_no_case_fails(self):
        self.assertEqual(self.found(self.edit(), stale={'renamed': 'moved'}),
                         ['renamed: in STALE, but no case has that name'])

    def test_stale_hides_only_the_case_it_names(self):
        self.assertEqual(self.found(self.edit('old', old='x'), self.edit('new', old='y'), stale={'old': 'moved'}),
                         [f"new: {self.file}: 'y' found 0 times, not 1"])


class ScriptTest(CaseTest):
    def script(self, edits, stale=None):
        module = types.SimpleNamespace(check_edits=lambda: edits)
        if stale is not None:
            module.STALE = stale
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            status = mc.check_script('x-mutations.py', module)
        return status, output.getvalue()

    def test_a_script_whose_cases_apply_passes_and_counts_them(self):
        self.assertEqual(self.script([self.edit('one'), self.edit('two')]),
                         (0, 'x-mutations.py: all 2 cases apply\n'))

    def test_a_script_with_a_stale_case_fails_and_names_only_that_case(self):
        self.assertEqual(self.script([self.edit('one'), self.edit('two'), self.edit('two', old='let b')]),
                         (1, f"x-mutations.py: two: {self.file}: 'let b' found 2 times, not 1\n"))

    def test_a_script_with_stale_cases_counts_them_apart(self):
        self.assertEqual(self.script([self.edit('one'), self.edit('old', old='x')], {'old': 'moved'}),
                         (0, 'x-mutations.py: all 1 cases apply, and the 1 in STALE still do not\n'))

    def test_a_script_with_no_check_edits_fails(self):
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            status = mc.check_script('x-mutations.py', types.SimpleNamespace())
        self.assertEqual((status, output.getvalue()),
                         (1, 'x-mutations.py: has no check_edits(), so its cases cannot be checked\n'))


class CommandTest(CaseTest):
    """The --check hook and the full run, on scripts written to a temporary folder."""

    def write(self, name, old, stale=None, main='raise SystemExit(7)'):
        path = Path(self.folder.name) / name
        path.write_text(
            'import sys\n'
            f'sys.path.insert(0, {str(SCRIPTS)!r})\n'
            'import mutation_cases\n'
            f'STALE = {stale!r}\n'
            'def check_edits():\n'
            f'    return [mutation_cases.Edit("case", {str(self.file)!r}, {old!r}, "changed")]\n'
            "if __name__ == '__main__':\n"
            '    mutation_cases.check_flag()\n'
            f'    {main}\n')
        return path

    def run_script(self, path, *arguments):
        return subprocess.run([sys.executable, '-I', '-B', str(path), *arguments],
                              capture_output=True, text=True)

    def test_check_exits_with_the_result_before_the_run(self):
        applies = self.run_script(self.write('a-mutations.py', 'let a = 1;'), 'out', '--check')
        self.assertEqual((applies.returncode, applies.stdout), (0, 'a-mutations.py: all 1 cases apply\n'))
        stale = self.run_script(self.write('b-mutations.py', 'let c = 3;'), '--check')
        self.assertEqual((stale.returncode, stale.stdout),
                         (1, f"b-mutations.py: case: {self.file}: 'let c = 3;' found 0 times, not 1\n"))

    def test_without_check_the_script_runs(self):
        self.assertEqual(self.run_script(self.write('a-mutations.py', 'let a = 1;'), 'out').returncode, 7)

    def test_the_full_run_checks_every_script_and_fails_on_any(self):
        self.write('a-mutations.py', 'let a = 1;')
        self.write('b-mutation.py', 'let c = 3;', stale={'case': 'moved'})
        (Path(self.folder.name) / 'c-mutations.py').write_text('def (:\n')
        (Path(self.folder.name) / 'helper.py').write_text('raise SystemExit("not a mutation script")\n')
        output = io.StringIO()
        scripts, mc.SCRIPTS = mc.SCRIPTS, Path(self.folder.name)
        self.addCleanup(setattr, mc, 'SCRIPTS', scripts)
        with contextlib.redirect_stdout(output):
            status = mc.main()
        lines = output.getvalue().splitlines()
        self.assertEqual(status, 1)
        self.assertEqual(lines[:2], ['a-mutations.py: all 1 cases apply',
                                     'b-mutation.py: all 0 cases apply, and the 1 in STALE still do not'])
        self.assertTrue(lines[2].startswith('c-mutations.py: does not load: SyntaxError: '), lines[2])
        self.assertEqual(lines[3:], ['3 mutation scripts checked: a case is stale, or a script cannot be checked'])

    def test_the_full_run_passes_when_every_script_does(self):
        self.write('a-mutations.py', 'let a = 1;')
        output = io.StringIO()
        scripts, mc.SCRIPTS = mc.SCRIPTS, Path(self.folder.name)
        self.addCleanup(setattr, mc, 'SCRIPTS', scripts)
        with contextlib.redirect_stdout(output):
            status = mc.main()
        self.assertEqual((status, output.getvalue().splitlines()[-1]),
                         (0, '1 mutation scripts checked: every case applies'))


if __name__ == '__main__':
    unittest.main()
