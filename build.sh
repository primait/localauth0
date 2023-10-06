#!/bin/bash

set -e

DOCKERFILE_PATH=$1
IMAGE_PREFIX=$2

VERSION="$DRONE_TAG"
IMAGE_NAME="${IMAGE_PREFIX}${VERSION}"

echo "building ${IMAGE_NAME}"

source /etc/profile.d/ecs-credentials-endpoint

version=$(grep -m1 version Cargo.toml | cut -d'"' -f2)

if [[ "$DRONE_TAG" != "$version" ]] ; then
    echo "Package version $version does not match release version $DRONE_TAG"
    exit 1
fi

docker build -t "${IMAGE_NAME}" -f Dockerfile_localauth0 "${DOCKERFILE_PATH}"
docker push "${IMAGE_NAME}"
