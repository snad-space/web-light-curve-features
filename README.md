# REST API for [light curve feature extractor](http://crates.io/crates/light-curve-feature)

## Python example

```python
import requests


# API version can differ, HTTPS is also availible
API_URL = 'http://features.lc.snad.space/api/v0.1.7'

def main():
    n = 100
    light_curve = [dict(t=t, m=0, err=0.1) for t in range(n)]
    data = dict(light_curve=light_curve)
    resp = requests.post(API_URL, json=data)
    resp.raise_for_status()
    features = resp.json()
    print(features)


if __name__ == '__main__':
    main()
```
