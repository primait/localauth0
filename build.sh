#!/bin/bash

set -e



DOCKERFILE_PATH=$1
IMAGE_PREFIX=$2

VERSION="$DRONE_TAG"
IMAGE_NAME="${IMAGE_PREFIX}${VERSION}"

echo "building ${IMAGE_NAME}"

source /etc/profile.d/ecs-credentials-endpoint
# aws ecr-public get-login-password --region us-east-1 | /usr/bin/docker login --username AWS --password-stdin public.ecr.aws

#docker run -v $PWD:/code --env CARGO_HOME=/home/app/.cargo prima/localauth0-ci:$DRONE_COMMIT

docker build -t "${IMAGE_NAME}" -f Dockerfile_localauth0 "${DOCKERFILE_PATH}"
docker push "${IMAGE_NAME}"
