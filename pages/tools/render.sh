#!/usr/bin/env bash

set -euxo pipefail

# OH GOD WHY AM I DOING THIS
# TODO: i need to make html pages a thing already 

dir="$(dirname "$0")"

cd "$dir"

cat <<EOF > laser-pattern.md
---
title: Laser pattern generator
tags: []
slug: /tools/laser-pattern
navbar_path: []
---
EOF

sed '/^[[:space:]]*$/d' < laser-pattern.html >> laser-pattern.md

