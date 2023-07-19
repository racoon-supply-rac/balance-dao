# CHANGELOG
Changes to this project will be added to this file.

## [Unreleased] - TBD

When the new Juno burn module is released, we'll add a burn function to burn the accumulated Juno.

## [0.1.1] - 2023-07-19

### Info
Juno Network's burn module re-mints the burned Juno when `Bank::Burn` is used.
It does so by sending to a dead address. 

### Changed
We now keep track of the amount of Juno to be burned and when the new `burn` module
that will burn without re-minting will be released, a function will be added to burn the amount.

### Added
- Snapshot of the amount that was "fake" burned so it can be actually burned later.
- New state to tracke the amount to be burned when the module is out.
- Placeholder for a one-time burn when new module is live.