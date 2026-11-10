"""Duty-cycles a process with SIGSTOP/SIGCONT so it gets `share` of its normal CPU time.
Usage: python3 -I throttle.py <pid> <share 0-1> [period_ms]"""
import os, signal, sys, time
pid, share = int(sys.argv[1]), float(sys.argv[2])
period = (float(sys.argv[3]) if len(sys.argv) > 3 else 10.0) / 1000
on, off = period * share, period * (1 - share)
def resume(*_):
    os.kill(pid, signal.SIGCONT); sys.exit(0)
signal.signal(signal.SIGTERM, resume); signal.signal(signal.SIGINT, resume)
try:
    while True:
        time.sleep(on); os.kill(pid, signal.SIGSTOP)
        time.sleep(off); os.kill(pid, signal.SIGCONT)
finally:
    os.kill(pid, signal.SIGCONT)
