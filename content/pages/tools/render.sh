#!/usr/bin/env bash

set -euxo pipefail

# OH GOD WHY AM I DOING THIS
# TODO: i need to make html pages a thing already 

dir="$(dirname "$0")"
slug=peg-engine

cd "$dir"

cat <<EOF > $slug.md
---
title: Pegging engine
tags: []
slug: /tools/$slug
navbar_path: []
---
EOF

sed '/^[[:space:]]*$/d' < $slug.html >> $slug.md

