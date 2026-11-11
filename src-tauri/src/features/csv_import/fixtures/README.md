# CSV import fixtures

## Youth Tracker Monza export

`2030_07_01_Full_Squad_CA_PA_Monza.csv` is copied without changes from [`JG1995/fm-youth-tracker` at `5a9043d`](https://github.com/JG1995/fm-youth-tracker/blob/5a9043d3303984567680c229f6711ba4a022daaf/fixtures/imports/2030_07_01_Full_Squad_CA_PA_Monza.csv).

SHA-256: `5c3a4a686f7619f7001e01530d4d44576718f6b3c131a7bb1a5faf94b99301a6`

## Canonical Moneyball statistics

`moneyball_canonical_statistics.txt` records the 138 performance-stat keys from the legacy `schema.rs` in [`JG1995/fm-valuescout-react` at `366aa20b`](https://github.com/JG1995/fm-valuescout-react/tree/366aa20b5282d3a63c94854ddb8da6992462b0c5). The Moneyball catalog test in `features/moneyball/catalog.rs` uses it to lock the exact persisted-key spelling independently from the Rust key table. Moneyball stats come from the linked FM save file, and the CSV importer rejects Moneyball exports.
