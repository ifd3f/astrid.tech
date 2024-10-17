#!/bin/sh

set -euxo pipefail

dir="${1:-astrid.tech-content}"
main_repo=https://github.com/ifd3f/astrid.tech.git
drafts_repo=https://github.com/ifd3f/astrid.tech-drafts.git

git clone "$main_repo" "$dir"
cd "$dir"
git remote add drafts "$drafts_repo"
git fetch --all
