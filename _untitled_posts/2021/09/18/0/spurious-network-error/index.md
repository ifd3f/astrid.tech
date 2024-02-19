---
title: null
tags:
  - project:astrid-tech
  - ci-cd
slug:
  date: 2021-09-19
  ordinal: 0
  name: spurious-network-error
date:
  created: 2021-09-18 21:07:20-07:00
  published: 2021-09-18 21:07:20-07:00
---

For some reason, my the automated build action for the Webmention receiver is
failing due to spurious network errors. They're definitely spurious, but they've
been consistently happening for the last several hours. I hope
[this strange workaround](https://github.com/rust-lang/cargo/issues/6513#issuecomment-920920238)
fixes it.
