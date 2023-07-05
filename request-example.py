#!/usr/bin/env python3

import sys
import time
from pprint import pprint
from random import random

import requests


API_URL = 'http://features.lc.snad.space/api/latest'
if len(sys.argv) > 1:
    API_URL = sys.argv[1]


def get_default_features(light_curve):
    data = dict(light_curve=light_curve)
    t = time.monotonic()
    resp = requests.post(API_URL, json=data)
    resp.raise_for_status()
    features = resp.json()
    t = 1e3 * (time.monotonic() - t)
    print(f'Requested in {t:.3f} ms')
    pprint(features)


def get_custom_features(light_curve):
    data = dict(
        light_curve=light_curve,
        extractor={
            "FeatureExtractor": {
                "features": [
                    {
                        "Amplitude": {}
                    },
                    {
                        "AndersonDarlingNormal": {}
                    },
                    {
                        "BazinFit":
                            {
                                "algorithm":
                                     {
                                         "Ceres":
                                             {
                                                "niterations": 10,
                                                "loss_factor": None,
                                             }
                                     },
                                "ln_prior":
                                     {
                                         "Fixed":
                                             {
                                                 "None":{}
                                             }
                                    },
                                "inits_bounds":"Default"
                            }
                    },
                    {
                        "BeyondNStd": {
                            "nstd": 1.0
                        }
                    },
                ]
            },
        },
    )
    t = time.monotonic()
    resp = requests.post(f'{API_URL}/features', json=data)
    resp.raise_for_status()
    features = resp.json()
    t = 1e3 * (time.monotonic() - t)
    print(f'Requested in {t:.3f} ms')
    pprint(features)


def main():
    light_curve = [dict(t=t, m=random(), err=0.1) for t in range(100)]
    get_default_features(light_curve)
    print()
    get_custom_features(light_curve)


if __name__ == '__main__':
    main()
