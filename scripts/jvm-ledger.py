"""Ratchet for the hand-written production JVM files: Kotlin and Java under a module's src/main.

Counted lines are the non-blank lines that still hold code once `//` comments and `/* */` block
comments (nested, and KDoc) are removed. Text inside string and character literals is code, so
`//` or `/*` inside a literal is not a comment. That is the rule of the 500-line file limit.
"""

LIMIT = 500

CODE, STR, RAW, BLOCK, LINE = "code", "str", "raw", "block", "line"


def _is_code(ch):
    return not (ch.isspace() or ch == "\ufeff")


def _char_literal_end(line, i):
    """Index just past the character literal whose opening quote is at line[i], or None."""
    j = i + 3 if line[i + 1:i + 2] == "\\" else i + 2
    while j < len(line) and j <= i + 12 and line[j] != "'":
        j += 1
    return j + 1 if j < len(line) and line[j] == "'" else None


def _line_has_code(line, stack):
    """Whether this line holds code. Updates `stack`, whose top is [kind, n]: n is the brace
    count of a ${...} template in code, or the comment depth in a block comment."""
    has = False
    i = 0
    while i < len(line):
        kind, depth = stack[-1]
        if kind == LINE:
            break
        if kind == BLOCK:
            if line.startswith("/*", i):
                stack[-1][1] += 1
                i += 2
            elif line.startswith("*/", i):
                stack[-1][1] -= 1
                i += 2
                if stack[-1][1] == 0:
                    stack.pop()
            else:
                i += 1
            continue
        ch = line[i]
        if kind in (STR, RAW) and line.startswith("${", i):
            stack.append([CODE, 0])
            has = True
            i += 2
        elif kind == STR and ch == "\\":
            has = True
            i += 2
        elif kind == STR and ch == '"':
            stack.pop()
            has = True
            i += 1
        elif kind == RAW and ch == '"':
            run = len(line[i:]) - len(line[i:].lstrip('"'))
            if run >= 3:
                stack.pop()
            has = True
            i += run
        elif kind in (STR, RAW):
            has = has or _is_code(ch)
            i += 1
        elif line.startswith("//", i):
            stack.append([LINE, None])
            break
        elif line.startswith("/*", i):
            stack.append([BLOCK, 1])
            i += 2
        elif ch == '"':
            has = True
            if line.startswith('"""', i):
                stack.append([RAW, None])
                i += 3
            else:
                stack.append([STR, None])
                i += 1
        elif ch == "'" and (end := _char_literal_end(line, i)) is not None:
            has = True
            i = end
        else:
            if depth is not None and ch == "{":
                stack[-1][1] += 1
            elif depth is not None and ch == "}":
                if depth == 0:
                    stack.pop()
                else:
                    stack[-1][1] -= 1
            has = has or _is_code(ch)
            i += 1
    return has


def code_flags(text):
    """One bool per line of `text`: whether that line holds code once comments are removed."""
    stack = [[CODE, None]]
    flags = []
    for line in text.split("\n"):
        flags.append(_line_has_code(line, stack))
        if stack[-1][0] == LINE:
            stack.pop()
        if stack[-1][0] == STR:
            stack.pop()
    return flags


def count_code_lines(text):
    """The counted lines of a Kotlin or Java source text."""
    return sum(code_flags(text))
