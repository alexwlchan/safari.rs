#!/usr/bin/env python
# -*- encoding: utf-8
"""
This script auto-generates the ``fix_se_referral`` lines used in ``urls.rs``.

It takes somebody's Stack Exchange account URL, of the form

    https://stackexchange.com/users/:user_id/:user_name

and spits out a list of ``fix_se_referral`` calls.
"""

import sys
from urllib.parse import urlparse

import bs4
import requests


se_account_url = sys.argv[1]
resp = requests.get(se_account_url, params={'tab': 'accounts'})

soup = bs4.BeautifulSoup(resp.text, 'html.parser')
for account in soup.find_all('div', attrs={'class': 'account-container'}):
    user_page_url = account.find('h2').find('a').attrs['href']
    components = urlparse(user_page_url)
    user_id = components.path.split('/')[2]
    print(f'fix_se_referral(&mut parsed_url, "{components.netloc}.com", "{user_id}");')
