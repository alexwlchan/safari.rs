#!/usr/bin/env python
# -*- encoding: utf-8
"""
These are tests of the external behaviour -- feature tests, if you like.
They run the compiled binaries, and make assertions about the return code,
stdout and stderr.
"""

import unittest

from conftest import BaseTest


class TestSafariRS(BaseTest):

    def test_urls_all_flag_is_deprecated(self):
        result = self.run_safari_rs('urls-all')
        self.assertIn('deprecated', result.stderr)

    def test_list_tabs_flag_is_not_deprecated(self):
        result = self.run_safari_rs('list-tabs')
        self.assertNotIn('deprecated', result.stderr)


if __name__ == '__main__':
    unittest.main()
