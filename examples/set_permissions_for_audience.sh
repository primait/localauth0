#!/bin/sh
curl --request POST \
  --url http://$HOST/permissions \
  --header 'Content-type: application/json' \
  --data-binary "{\"audience\":\"$AUDIENCE\",\"permissions\":[\"$PERMISSION\"]}"
