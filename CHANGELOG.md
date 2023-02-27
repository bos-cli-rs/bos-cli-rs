# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
