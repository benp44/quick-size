#!/bin/bash

set -euf -o pipefail

script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
context="${script_dir}/../../"

echo "Building docker image with context: ${context}..."
docker build --tag quick_size "${context}" -f "${context}/docker/tests/functional/Dockerfile"

echo "Running tests"
docker run --rm quick_size /tmp/tests/functional/run-functional-tests.sh
