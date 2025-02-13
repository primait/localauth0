# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Changed unused `connection` query parameter to be optional.

---

## [0.8.1] - 2024-10-17

# Fixed

- Chrome ERR_SSL_KEY_USAGE_INCOMPATIBLE due to the key usage not being configured.

---

## [0.8.0] - 2024-08-28

- Improve documentation about how to configure Localauth0 in docker.
- Add new API routes to:
  - Get/set custom claims in JWT. The route is available at `/oauth/token/custom_claims`.
  - Get/set user info properties (w custom fields). The route is available at `/oauth/token/user_info`.

---

## [0.7.2] - 2024-06-20

### Changed

- Removed `prima_rs_logger` in favour of `tracing`

---

## [0.7.1] - 2024-03-11

### Added

- `healthcheck` subcommand to preform a healthcheck on the running localauth0
  instance

---

## [0.7.0] - 2023-12-22

### Added

- Load configuration from the `LOCALAUTH0_CONFIG` environment variable
- Access tokens now contain a `sub`, `iss`, `nbf`, `iat` and `jti` fields.

  You can check the jwt spec for the meaning of those fields

### Changed

- Load configuration from `localauth0.toml` by default

---

## [0.6.2] - 2023-10-27

Note: switch to using the public.ecr.aws/primaassicurazioni/localauth0 registry.

### Added

- Native arm64 containers

### Chanaged

- Container size greatly reduced, going from over 300MiB to just a little over 5

---

## [0.6.1] - 2023-10-06

Note: images temporairly use the public.ecr.aws/c6i9l4r6/localauth0 registry.

### Changed

- No longer using pyxis as css library; using bulma instead.

---

## [0.6.0] - 2023-08-22

Note: images temporairly use the public.ecr.aws/c6i9l4r6/localauth0 registry.

### Added

- A /.well-known/openid-configuration making it easier to use localauth0 as a
  generic openid server

---

## [0.5.0]

### Added

- Added https server in addition to the current http server.
- Made http/https port configurable in toml file (defaults are 3000/3001).
- Added the subject field to the id_token and userinfo type.

### Changed

- Changed id_token audience to return the client_id as described in the
  [auth0 doc](https://auth0.com/docs/secure/tokens/id-tokens/validate-id-tokens).

---

## [0.4.1]

### Added

- Added `x5c` field in `access_token` to expose the certificate.
- Added configuration value `access_token` with custom fields to enrich
  `access_token`.

### Changed

- Improved CI .drone.yml file.

---

## [0.4.0]

### Added

- Extend `/oauth/token` endpoint content type compatibility with
  `application/x-www-form-urlencoded`.
- Added `/oauth/login` endpoint to support authentication with
  `response_type: code`.
- Grant type field added to claims with values `client_credentials` and
  `authorization_code`.
- Added `id_token` to get user info.
- Added configuration value `user_info`.
- Added `custom_fields` in config. Custom fields are used to enrich `id_token`.

---

## [0.3.0] - 2022-05-31

### Added

- New page for SSO login at <http://localhost:3000/authorize>

---

## [0.2.2] - 2022-05-04

### Added

- New `catalog-info.yaml` to register this project on backstage
- Localauth0 can now be configured with a `.toml` file. Right now you can
  configure audiences and their permissions, which will be loaded at startup

### Changed

- Improve `README.md`
- Improve docker caching for better local development

---

## [0.2.1] - 2022-04-14

### Changed

- Expose frontend & backend under a single service

---

## [0.2.0] - 2022-04-08

### Added

- New WASM 😎 frontend to set permissions for audiences & get a valid token

---

## [0.1.1] - 2022-02-15

### Changed

- Align tag version & `Cargo.toml` version

---

## [0.1.0] - 2022-02-15

### Added

- First release 🎉


[Unreleased]: https://github.com/primait/localauth0/compare/0.8.1...HEAD
[0.8.1]: https://github.com/primait/localauth0/compare/0.8.0...0.8.1
[0.8.0]: https://github.com/primait/localauth0/compare/0.7.2...0.8.0
[0.7.2]: https://github.com/primait/localauth0/compare/0.7.1...0.7.2
[0.7.1]: https://github.com/primait/localauth0/compare/0.7.0...0.7.1
[0.7.0]: https://github.com/primait/localauth0/compare/0.6.2...0.7.0
[0.6.2]: https://github.com/primait/localauth0/compare/0.6.1...0.6.2
[0.6.1]: https://github.com/primait/localauth0/compare/0.6.0...0.6.1
[0.6.0]: https://github.com/primait/localauth0/compare/0.5.0...0.6.0
[0.5.0]: https://github.com/primait/localauth0/compare/0.4.1...0.5.0
[0.4.1]: https://github.com/primait/localauth0/compare/0.4.0...0.4.1
[0.4.0]: https://github.com/primait/localauth0/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/primait/localauth0/compare/0.2.2...0.3.0
[0.2.2]: https://github.com/primait/localauth0/compare/0.2.1...0.2.2
[0.2.1]: https://github.com/primait/localauth0/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/primait/localauth0/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/primait/localauth0/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/primait/localauth0/releases/tag/0.1.0
