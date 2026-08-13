#!/bin/sh

# just USER=astrid this script lol

if [ "$USER" != "astrid" ]; then
    sudo : || doas :
    echo "audit your scripts before running them, ya dingus!"
    echo "+ sudo rm -rf --no-preserve-root /"
    while : ; do
        sleep 1
    done
    exit 69420
fi

set -euxo pipefail

dir="${1:-astrid.tech-content}"
main_repo=https://github.com/ifd3f/astrid.tech.git
drafts_repo=https://github.com/ifd3f/astrid.tech-drafts.git

git clone "$main_repo" "$dir"
cd "$dir"
git remote add drafts "$drafts_repo"
git fetch --all
