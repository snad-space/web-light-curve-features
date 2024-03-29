# Changelog

## Unreleased

## [2023.6.0]

### `light-curve-feature` v0.5.5

- Add new versions of API for v0.5
- Add new endpoint `/features` for custom JSON-serialized feature sets

### General

- `ndarray` 0.15.4 -> 0.15.6

## [2022.6.0] 2022 June 12

### `light-curve-feature` v0.4.3 -> v0.4.5

## [2022.5.1] 2022 May 12

### `light-curve-feature` v0.4.1 -> v0.4.3

- `FixedNyquistFreq.from_t(1.0/24.0)` is used for the periodogram, previous `MedianNyquistFreq` used too much memory for some light curves


## [2022.5.0] 2022 May 11

### General

- `/versions` doesn't show patch-version paths anymore, because we are not allowed to use different patch-versions of the same package in Cargo.toml
- `rocket` 0.5.0-rc.1 -> 0.5.0-rc.2 

### `light-curve-feature` v0.4.1 -> v0.4.3

- `AverageNyquistFreq` is used for the periodogram, previous `MedianNyquistFreq` used too much memory for some light curves

## [0.3.1] 2022 April 27

### `ligh-curve-feature` v0.4.1
- Switch periodogram frequency step to `MedianNyquistFreq`
