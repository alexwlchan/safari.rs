#!/usr/bin/env python
# -*- encoding: utf-8
"""
These are tests of the external behaviour -- feature tests, if you like.
They run the compiled binaries, and make assertions about the return code,
stdout and stderr.
"""

from conftest import run_safari_rs


def test_urls_all_flag_is_deprecated():
    result = run_safari_rs('urls-all')
    assert 'deprecated' in result.stderr


def test_list_tabs_flag_is_deprecated():
    result = run_safari_rs('list-tabs')
    assert result.stderr == ''
