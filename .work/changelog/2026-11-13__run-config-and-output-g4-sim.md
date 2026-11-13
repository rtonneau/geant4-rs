---
bump: major
floor: minor
reason: config key rename and unknown-key rejection break existing run configs and the output schema
---
- Breaking: the run config key `geometry` is now `volume` (`[volume]` in TOML), and unknown config keys are rejected. Output schemas are now `run/v5` and `run-unverified/v2`.
- The provenance record includes the run's volume (omitted for infinite water).
- New `summary.escape` block for runs in a volume: primary escape fraction, escaped tracks per history, escaped energy fraction and escaped-energy spectra (primary, secondaries, all).
- New exit-event CSV (`CsvEscapeWriter`): one row per escaped track with exit point, direction and energy, identical for any thread count.
