#!/usr/bin/env bash
# /github/workspace is a bind mount and might have wonky uids that scare modern git versions
git config --global --add safe.directory "*"
rustup default stable

if [ -n "$2" ] then
#  export GH_TOKEN="$GITHUB_TOKEN"
#  workflow_id=$(gh api repos/tectonic-typesetting/tectonic/actions/artifacts \
#    --jq ".artifacts[] | select(.workflow_run.head_sha == \"$2\") | select(.name == \"tectonic-on-arxiv\") | .workflow_run.id")
#  if [ -z "$2" ] then
#    run_url=gh workflow run tectonic-on-arxiv.yml -f head="$2"
#    gh run watch ${run_url##*/}
#  fi
#  gh run download $workflow_id -n tectonic-on-arxiv
#  unzip tectonic-on-arxiv.zip -d /root/reports/
fi

cd /root/github-ci
export HEAD_COMMIT="$1"
export BASE_COMMIT="$2"
yarn start
