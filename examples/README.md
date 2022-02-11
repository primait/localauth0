# Auth0 Local Stack examples

For local development (from outside the container):
```shell
# Export HOST env var
export HOST='localhost:3000'
# Run the server
cargo run
```

## Get jwks

```shell
examples/jwks.sh
```

## Get new jwt

First export environment variables

```shell
# The correct value is `client_id`
export CLIENT_ID=client_id
# The correct value is `client_secret`
export CLIENT_SECRET=client_secret
export AUDIENCE=<whatever>
```

Then run the example
```shell
examples/jwt.sh
```

## Set permissions for audience

```shell
export AUDIENCE=<whatever>
export PERMISSION=<whatever>
```

Then run the example
```shell
examples/set_permissions_for_audience.sh
```

Calling again the [get new jwt](#Get-new-jwt) API the resulting `access_token` contains given permissions

## Precooked env vars
```shell
export HOST='localhost:3000'
export CLIENT_ID=client_id
export CLIENT_SECRET=client_secret
export AUDIENCE=audience
export PERMISSION=permission
```
