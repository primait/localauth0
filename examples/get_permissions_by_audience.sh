#!/bin/sh
curl --request GET \
  --url http://$HOST/permissions/$AUDIENCE \
  --header 'Content-type: application/json'
