#!/usr/bin/env bash
# /github/workspace is a bind mount and might have wonky uids that scare modern git versions
git config --global --add safe.directory "*"
cd /root/github-ci
yarn start
