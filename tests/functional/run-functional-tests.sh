#!/bin/bash

set -euf -o pipefail

script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

testdir="/tmp/testroot"

if [ -d "${testdir}" ]; then
    rm -rf "${testdir}"
fi

mkdir -p "${testdir}/dir0/"
mkdir -p "${testdir}/dir1/"
mkdir -p "${testdir}/dir2/"
mkdir -p "${testdir}/dir3/"
mkdir -p "${testdir}/dir3/subdir/subdir2"

fallocate -l 512 "${testdir}/dir1/file"
fallocate -l 512 "${testdir}/dir2/file1"
fallocate -l 512 "${testdir}/dir2/file2"
fallocate -l 512 "${testdir}/dir3/file"
fallocate -l 512 "${testdir}/dir3/subdir/file"
fallocate -l 512 "${testdir}/dir3/subdir/subdir2/file"

cd "${testdir}/"

function test_dir_size {
    dir_size=$(${script_dir}/../../target/release/quick-size 2>/dev/null | grep ${1} | awk '{ print $2 }')
    if [ ${dir_size} != "${2}" ]; then
        exit 1
    fi
}

# Some really basic assertions

test_dir_size "dir1" "512"
test_dir_size "dir2" "1024"
test_dir_size "dir3" "1536"

echo "All tests pass"
