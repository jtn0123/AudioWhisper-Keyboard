"""Optional T3-bound UI transport. Never return UI text to fixture logs."""
import os
import subprocess


def run(*args):
    command = os.environ['AGENT_DEVICE']
    result = subprocess.run([command, *map(str, args)], capture_output=True, text=True, timeout=90)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return ''


def launch(args):
    args = list(map(str, args))
    if '-d' in args:
        command = ['open', args[args.index('-p') + 1], args[args.index('-d') + 1], '--no-test-ime']
        for index, arg in enumerate(args):
            if arg in ('-a', '-c', '-f'):
                command += ['--launch-args', arg, '--launch-args', args[index + 1]]
        return run(*command)
    component = args[args.index('-n') + 1]
    extra = args[args.index('-n') + 2:]
    command = ['open', component.split('/')[0], '--activity', component, '--no-test-ime']
    for arg in extra:
        command += ['--launch-args', arg]
    return run(*command)


def touch(args):
    args = list(map(str, args))
    if args[0] == 'tap':
        return run('press', *args[1:])
    if args[0] == 'text':
        return run('type', *args[1:])
    if args[0] == 'swipe':
        x, y, end_x, end_y, duration = map(int, args[1:])
        if (x, y) == (end_x, end_y):
            return run('press', x, y, '--hold-ms', duration) if duration < 300 else run('longpress', x, y, duration)
        return run('gesture', 'pan', x, y, end_x - x, end_y - y, duration)
    raise RuntimeError('UI transport cannot express this fixture: ' + args[0])
