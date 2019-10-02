#!/bin/bash

set -eu

cargo build --release

rm -r target/deploy
mkdir -p target/deploy

cd target/release
zip ../deploy/awscredx-${TRAVIS_OS_NAME}.zip awscredx$exe_suffix
cd ../..

ls -l target/deploy