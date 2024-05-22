#!/usr/bin/env bash

set -x
set -o pipefail


docker run \
  --rm \
  -d \
  -p 9042:9042
  cassandra