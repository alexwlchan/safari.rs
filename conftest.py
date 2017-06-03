#!/usr/bin/env python
# -*- encoding: utf-8

import collections
import os
import subprocess
import unittest


Result = collections.namedtuple('Result', 'rc stdout stderr')

ROOT = subprocess.check_output([
    'git', 'rev-parse', '--show-toplevel']).decode('ascii').strip()

BINARY = os.path.join(ROOT, 'target', 'release', 'safari')


subprocess.check_call(['cargo', 'build', '--release'], cwd=ROOT)


class BaseTest(unittest.TestCase):

    def run_safari_rs(self, *args):
        proc = subprocess.Popen([BINARY] + list(args),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        stdout, stderr = proc.communicate()
        return Result(
            rc=proc.returncode,
            stdout=stdout.decode('ascii'),
            stderr=stderr.decode('ascii')
        )
