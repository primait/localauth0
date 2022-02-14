#!/bin/bash

set -e



DOCKERFILE_PATH=$1
IMAGE_PREFIX=$2

VERSION="$DRONE_TAG"
IMAGE_NAME="${IMAGE_PREFIX}${VERSION}"

echo "building ${IMAGE_NAME}"

source /etc/profile.d/ecs-credentials-endpoint

docker build -t "${IMAGE_NAME}" -f Dockerfile_localauth0 "${DOCKERFILE_PATH}"
docker push "${IMAGE_NAME}"
