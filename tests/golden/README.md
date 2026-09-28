# Golden charts

Each `.json` file here pins the complete output for one birth chart. The suite
in `crates/lagn-core/tests/golden.rs` replays every fixture on every test run.

## What a fixture proves

A fixture on its own proves only that **our output has not changed**. That is
worth having - it catches accidental drift - but it is not evidence that the
output is *right*, because we generated it ourselves.

Correctness comes from the `verified_against` field. Until a human has opened
the same birth data in JHora and confirmed every value, that field stays
`null`. The Phase 1 exit criterion is 50 fixtures with `verified_against` set.

    verified_against: null           regression lock only
    verified_against: "JHora 8.0"    externally confirmed

`crates/lagn-core/tests/golden.rs` reports the verified count on every run.

## Adding a fixture

```bash
cargo build --release -p lagn-cli
python3 scripts/make_golden.py --id kumbakonam-1962 \
    --date 1962-03-11 --time 05:42 \
    --lat 10.9601 --lon 79.3788 --tz 5.5 --place Kumbakonam \
    --description "pre-dawn birth, lagna close to a sign boundary"
```

Then open the same data in JHora, compare, and edit the file:

```json
"verified_against": "JHora 8.0",
"verified_by": "<name>",
"verified_on": "2026-09-21",
"notes": "Ketu pada differs if true node is selected - see below."
```

If JHora disagrees, **do not edit the expected values to match**. Open an
issue, find out which of the two is wrong, and fix the cause.

## JHora settings that must match

A disagreement is far more often a settings mismatch than a bug. Before
reporting one, confirm in JHora under *Preferences → Calculation*:

| Setting          | Must be                                      |
|------------------|----------------------------------------------|
| Ayanamsa         | Lahiri (Chitrapaksha)                        |
| Node             | Mean (not True)                              |
| House system     | Whole sign / Rasi = Bhava                    |
| Dasha year       | 365.25 days                                  |
| Ephemeris        | Swiss                                        |
| Time zone        | entered as a raw offset, not a city lookup   |

The last two rows cause most false alarms. JHora's built-in city database
applies its own historical timezone rules, which are not always the ones a
birth certificate implies - for any pre-1955 Indian birth, enter the offset by
hand on both sides and record which one you used in `notes`.

## Coverage the 50 fixtures should reach

- [ ] 15 ordinary Tamil Nadu / Kerala births across the year
- [ ] 5 with the lagna within 1 arcmin of a rasi boundary
- [ ] 5 with a graha within 1 arcmin of a nakshatra or pada boundary
- [ ] 5 pre-1947 births, including at least two pre-1906 local-mean-time
- [ ] 3 with a retrograde graha at its stationary point
- [ ] 3 where mean and true node fall in different nakshatras
- [ ] 3 southern hemisphere
- [ ] 3 high latitude (above 60 degrees)
- [ ] 3 born within a minute of midnight
- [ ] 2 leap day
- [ ] 3 with Moon within 1 arcmin of a nakshatra boundary (changes the
      birth dasha lord outright, the single highest-consequence edge case)
