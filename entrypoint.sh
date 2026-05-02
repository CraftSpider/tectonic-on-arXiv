#!/usr/bin/env bash
# /github/workspace is a bind mount and might have wonky uids that scare modern git versions
git config --global --add safe.directory "*"
rustup default stable
cd /root/github-ci
export HEAD_COMMIT=$1
yarn start
