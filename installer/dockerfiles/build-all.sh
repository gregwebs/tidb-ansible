#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")"
for dir in $(find . -name Dockerfile | xargs dirname); do
  pushd "$dir"
  ./build.sh
  popd
done
