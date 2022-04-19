# Localauth0

[![Build Status](https://drone-1.prima.it/api/badges/primait/localauth0/status.svg)](https://drone-1.prima.it/primait/localauth0)

![localauth0](web/assets/static/media/localauth0.png)

Localauth0 is a project that aims to be a helper while developing authentications inspired by [localstack](https://localstack.cloud/).
Most of the time people tend to mock authentication in order to not be forced to create complex mocks.
With localauth0 you can fake your [auth0](https://auth0.com/) tenant and test it offline for "real".

## Installation

In order to run localauth0 docker image execute the following:

```
docker run -d -p 3000:3000 public.ecr.aws/prima/localauth0:0.2.1
```

Note: The latest version is the same `version` written in the `Cargo.toml` file.

## APIs

### Web page

After having run the localauth0 machine a web interface is available at [http://localhost:3000/](http://localhost:3000/).
Here it's possible to:
- get a fresh new JWT with given `audience`.
- add/remove permissions for a given `audience`.

### Jwt

- `POST` [http://localhost:3000/oauth/token](http://localhost:3000/oauth/token): used to get a freshly new JWT. Body 
  should be: 
  ```json
  {
    "client_id": "client_id",
    "client_secret": "client_secret",
    "audience": "{{your-audience}}",
    "grant_type": "client_credentials"
  }
  ```

- `POST` [http://localhost:3000/permissions](http://localhost:3000/permissions): used to set a list of permissions for 
  given audience. Everytime a new JWT is requested for that audience those permissions will be injected in the JWT 
  payload. Body should be:
  ```json
  {
    "audience": "{{your-audience}}",
    "permissions": ["your-permission-1", "your-permission-2", ".."]
  }
  ```

### Jwks

- `GET` [http://localhost:3000/.well-known/jwks.json](http://localhost:3000/.well-known/jwks.json): it's possible to 
fetch running instance jwks. Those jwks are randomly created starting from random certificates. 
Note that every generated JWT will be signed using one of those JWKS.

- `GET` [http://localhost:3000/rotate](http://localhost:3000/rotate): discard the last JWK of the JWKS list and 
  prepend a freshly new JWK.

- `GET` [http://localhost:3000/revoke](http://localhost:3000/revoke): discard all the JWKs in the JWKS list and 
  replace them with 3 freshly new JWKs.

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

Now website is available at http://localhost:3000.

#### Build and run localauth0 as an image

As mandatory step it's needed to create the artifact and the web dist. In order to achieve this run `cargo make 
build` or `cargo make run` commands from within the docker-compose container (alternatively from host machine if 
`cargo` and `trunk` are installed).

Then run:
```shell
docker build -f Dockerfile_localauth0 -t localauth0 . && \
docker run -d -p 3000:3000 localauth0
```

### Integrate localauth0 in existing project

Add this snippet to your `docker-compose.yml` file and reference it in your app `depends_on` section.
```yaml
  auth0:
    image: public.ecr.aws/prima/localauth0:0.2.1
    ports:
      - "3000:3000"
```
