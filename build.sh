#!/bin/bash

set -e



DOCKERFILE_PATH=$1
IMAGE_PREFIX=$2

VERSION="$DRONE_TAG"
IMAGE_NAME="${IMAGE_PREFIX}${VERSION}"

echo "building ${IMAGE_NAME}"

source /etc/profile.d/ecs-credentials-endpoint
# aws ecr-public get-login-password --region us-east-1 | /usr/bin/docker login --username AWS --password-stdin public.ecr.aws

docker build . -t localauth0-temp:$VERSION
docker run -v $PWD:/code --env CARGO_HOME=/home/app/.cargo localauth0-temp:$VERSION

docker build -t "${IMAGE_NAME}" -f Dockerfile_localauth0 "${DOCKERFILE_PATH}"
CI=true dive "${IMAGE_NAME}"
docker push "${IMAGE_NAME}"
