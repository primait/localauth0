# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2022-05-04

### Added

- New `catalog-info.yaml` to register this project on backstage
- Localauth0 can now be configured with a `.toml` file.
  Right now you can configure audiences and their permissions, which will be loaded at startup

### Changed

- Improve `README.md`
- Improve docker caching for better local development

## [0.2.1] - 2022-04-14

### Changed

- Expose frontend & backend under a single service

## [0.2.0] - 2022-04-08

### Added

- New WASM 😎 frontend to set permissions for audiences & get a valid token

## [0.1.1] - 2022-02-15

### Changed

- Align tag version & `Cargo.toml` version

## [0.1.0] - 2022-02-15

### Added

- First release 🎉

[Unreleased]: https://github.com/primait/localauth0/compare/0.2.2...HEAD
[0.2.2]: https://github.com/primait/localauth0/compare/0.2.1...0.2.2
[0.2.1]: https://github.com/primait/localauth0/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/primait/localauth0/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/primait/localauth0/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/primait/localauth0/releases/tag/0.1.0
