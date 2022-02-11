#!/bin/sh
curl --request POST \
  --url http://$HOST/oauth/token \
  --header 'Content-type: application/json' \
  --data-binary "{\"client_id\":\"$CLIENT_ID\",\"client_secret\":\"$CLIENT_SECRET\",\"audience\":\"$AUDIENCE\",\"grant_type\":\"client_credentials\"}"
