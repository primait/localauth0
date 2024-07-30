# Localauth0

[![Build Status](https://github.com/primait/localauth0/actions/workflows/ci.yml/badge.svg)](https://github.com/primait/localauth0/actions/workflows/ci.yml/badge.svg)

![localauth0](web/assets/static/media/localauth0.png)

Localauth0 is a project that aims to be a helper while developing
authentications inspired by [localstack](https://localstack.cloud/). Most of the
time people tend to mock authentication in order to not be forced to create
complex mocks. With localauth0 you can fake your [auth0](https://auth0.com/)
tenant and test it offline for "real".

## Table of contents

- [Installation](#installation)
- [APIs](#apis)
  - [Web page](#web-page)
  - [Jwt](#jwt)
  - [Jwks](#jwks)
- [Configuration](#configuration)
  - [Local development](#local-development)
  - [Integrate Localauth0 in an existing docker compose project](#integrate-localauth0-in-an-existing-docker-compose-project)
- [Healthchecks](#healthchecks)

## Installation

In order to run localauth0 docker image execute the following:

```shell
docker run -d -p 3000:3000 public.ecr.aws/primaassicurazioni/localauth0:0.7.2
```

By default, the container exposes an http server on the port 3000 and an https
one on port 3001.

Note: The latest version is the same `version` written in the `Cargo.toml` file.

## APIs

### Web page

After having run the localauth0 machine a web interface is available at
<http://localhost:3000/>. Here it's possible to:

- get a fresh new JWT with given `audience`.
- add/remove permissions for a given `audience`.

### Jwt

- `POST` <http://localhost:3000/oauth/token>: used to get a freshly new JWT.
  Body should be:

  ```json
  {
    "client_id": "client_id",
    "client_secret": "client_secret",
    "audience": "{{your-audience}}",
    "grant_type": "client_credentials"
  }
  ```
  for the client credentials grant and

  ```json
  {
    "client_id": "client_id",
    "client_secret": "client_secret",
    "grant_type": "authorization_code",
    "code": "{{your-auth-code}}"
  }
  ```
  for the authorization code grant.

- `GET` <http://localhost:3000/permissions>: used to get a the list of all
  audiences with their associated permissions.

- `POST` <http://localhost:3000/permissions>: used to set a list of permissions
  for given audience. Everytime a new JWT is requested for that audience those
  permissions will be injected in the JWT payload. Body should be:

  ```json
  {
    "audience": "{{your-audience}}",
    "permissions": ["your-permission-1", "your-permission-2", ".."]
  }
  ```

- `GET` <http://localhost:3000/permissions/{audience}>: used to get a the list
  of all permissions for the given audience.

### Jwks

- `GET` <http://localhost:3000/.well-known/jwks.json>: it's possible to fetch
  running instance jwks. Those jwks are randomly created starting from random
  certificates. Note that every generated JWT will be signed using one of those
  JWKS.

- `GET` <http://localhost:3000/rotate>: discard the last JWK of the JWKS list
  and prepend a freshly new JWK.

- `GET` <http://localhost:3000/revoke>: discard all the JWKs in the JWKS list
  and replace them with 3 freshly new JWKs.

## SSO page

Localauth0 could behave like Google SSO page. In order to achieve this your web
page should navigate to <http://localhost:3000/authorize> providing these query
params:

- redirect_uri: your web app callback page
- audience: the audience you want to use to generate the token
- response_type (optional): could be `token` or `code`. Use `token` to perform
  an implicit grant flow and retrieve an access token directly and use `code` to
  perform an authorization code flow and recieve an authorization code. If auth
  is succesful, the requested redirect is performed with the code or token
  contained in the query params.
- state (optional): An opaque value, used for security purposes. If this request
  parameter is set in the request, then it is returned to the application as
  part of the `redirect_uri`.
- bypass (optional): this is a dev feature. If set to true directly redirect to
  `redirect_uri`.

After redirection the redirect_url will contain these http fragments:

- access_token: the JWT token.
- token_type: always set to `Bearer`.
- expires_in: when will the jwt expires.
- state: present if set in authorize url; contains the same value.

For example navigating to:

<http://localhost:3000/authorize?redirect_uri=http%3A%2F%2Flocalhost%3A3000%2F&audience=audience1&client_id=client_id&connection=whatever&response_type=token&scope=whatever&state=test-state&bypass=true>

The page will automatically redirect to:

<http://localhost:3000/#access_token=eyJ..RrQ&token_type=Bearer&expires_in=3600&state=test-state>

## Configuration

Localauth0 can be configured using a `localauth0.toml` file (see
[localauth0.toml](localauth0.toml) as an example) or using the
`LOCALAUTH0_CONFIG` environment variable.

Take a look [here](#Integrate-localauth0-in-existing-project) to see how to
configure your docker compose cluster.

### Local development

#### Run localauth0 from within a docker-compose

Get into docker-compose container with:

```shell
docker-compose run --service-ports web bash
```

Build the artifact, the web dist and run the http server with:

```shell
# Build the web dist with trunk. Then run the server
cargo make run
```

Now website is available at <http://localhost:3000>.

#### Build and run localauth0 as an image

As mandatory step it's needed to create the artifact and the web dist. In order
to achieve this run `cargo make build` or `cargo make run` commands from within
the docker-compose container (alternatively from host machine if `cargo` and
`trunk` are installed).

For someone this error could occur on host machine

```shell
error[E0463]: can't find crate for `core`
  |
  = note: the `wasm32-unknown-unknown` target may not be installed
  = help: consider downloading the target with `rustup target add wasm32-unknown-unknown`

error[E0463]: can't find crate for `compiler_builtins`
```

In order to fix it run

```shell
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Then run:

```shell
docker build -f Dockerfile_localauth0 -t localauth0 . && \
docker run -d -p 3000:3000 localauth0
```

### Integrate Localauth0 in an existing docker compose project

Add this snippet to your `docker-compose.yml` file and reference it in your app
`depends_on` section.

```yaml
auth0:
  image: public.ecr.aws/primaassicurazioni/localauth0:0.7.2
  healthcheck:
    test: ["CMD", "/localauth0", "healthcheck"]
```

#### Configuration using inline environment variable

It is possible to add the `LOCALAUTH0_CONFIG` environment variable with an
inline configuration to let Localauth0 load the configuration at startup. For
example:

```yaml
auth0:
  image: public.ecr.aws/primaassicurazioni/localauth0:0.7.2
  healthcheck:
    test: ["CMD", "/localauth0", "healthcheck"]
  environment:
    LOCALAUTH0_CONFIG: |
issuer = "https://prima.localauth0.com/"
[user_info]
given_name = "Locie"
```

#### Configuration using a config file

Another way to configure Localauth0 is to mount a configuration file in the
container. For example, create a `localauth0.toml` file with the following:

```toml
issuer = "https://prima.localauth0.com/"

[user_info]
given_name = "Locie"
```

Then mount the file in the container using the following snippet in your
`docker-compose.yml` file:

```yaml
auth0:
  image: public.ecr.aws/primaassicurazioni/localauth0:0.7.2
  healthcheck:
    test: ["CMD", "/localauth0", "healthcheck"]
  environment:
    LOCALAUTH0_CONFIG_PATH: /etc/localauth0.toml
  volumes:
    - ./localauth0.toml:/etc/localauth0.toml:ro
  ports:
    - "3000:3000"
```

### Healthchecks

The localauth0 binary can perform a healthcheck on the running localauth0
service. Simply run `localauth0 healtcheck`
