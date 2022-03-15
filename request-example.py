#!/usr/bin/env python3

import sys
import time
from random import random

import requests


API_URL = 'http://features.lc.snad.space/'
if len(sys.argv) > 1:
    API_URL = sys.argv[1]


def main():
    light_curve = [dict(t=t, m=random(), err=0.1) for t in range(100)]
    data = dict(light_curve=light_curve)
    t = time.monotonic()
    resp = requests.post(API_URL, json=data)
    resp.raise_for_status()
    features = resp.json()
    t = 1e3 * (time.monotonic() - t)
    print(f'Requested in {t:.3f} ms')
    print(features)


if __name__ == '__main__':
    main()
