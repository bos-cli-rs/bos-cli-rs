# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2](https://github.com/bos-cli-rs/bos-cli-rs/compare/v0.3.1...v0.3.2) - 2023-05-24

### Added
- New command to delete components (#35)
- New command to grant access permission to SocialDB keys (#34)

### Fixed
- Trimmed extra space at the beginning of a line in interactive queries (#39) (#40)
- fixed consts

### Other
- Added test and release-plz workflows to CI (#41)
- Fixed the link in the GitHub Actions snippet to point to `master` branch
- Merge pull request #33 from FroVolod/update-near-library
- rustup update: 1.68.2
- updated near-cli-rs
- Updated reusable GitHub Actions Workflow to use bos-cli 0.3.1 version by default

## [0.3.1] - 2023-04-28

Fixes:
* Support large codebases

## [0.3.0] - 2023-04-20

Breaking changes:
* Renamed CLI to bos (crate name is bos-cli, and repo name is bos-cli-rs)
* Restructured the commands

## [0.2.4] - 2023-03-02

* Upgraded cargo-dist to 0.0.4-prelease.2 to reduce the Linux release binary size

## [0.2.3] - 2023-02-27

* Allow `null`-able tags in the widgets metadata to be able remove the tags on deployment

## [0.2.2] - 2023-02-18

* Added Apple M1/M2 binary release support (ARM64 Darwin target)

## [0.2.1] - 2023-02-18

* Reverted an accidental breaking change of renaming `network-config` to `network-for-transaction` subcommand

## [0.2.0] - 2023-02-17

* **BREAKING CHANGE:** Improved access keys handling in `deploy` command (allow using zero deposit for write-granted access), so now you can have a signer account ID that is different from the account ID where you want to deploy the widgets

## [0.1.1] - 2023-02-02

* Added shell script installers via cargo-dist

## [0.1.0] - 2023-02-02

* Initial version with only two commands: `deploy` and `download`
