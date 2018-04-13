#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")"
for dir in $(dirname ./*/Dockerfile); do
  pushd "$dir"
  ./build.sh
  popd
done
