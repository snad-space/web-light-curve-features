# Changelog

## [2022.5.0] 2022 May 11

### General

- `rocket` 0.5.0-rc.1 -> 0.5.0-rc.2 

### `light-curve-feature` v0.4.1 -> v0.4.3

- `FixedNyquistFreq(1.0/24.0)` is used for the periodogram, previous `MedianNyquistFreq` used too much memory for some light curves

## [0.3.1] 2022 April 27

### `ligh-curve-feature` v0.4.1
- Switch periodogram frequency step to `MedianNyquistFreq`