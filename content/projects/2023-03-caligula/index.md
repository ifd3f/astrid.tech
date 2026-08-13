---
title: Caligula Burning ROM
tagline: dd for the lazy
slug: caligula
status: finished
date:
  started: 2023-03-01
  finished: 2023-03-10
  published: 2024-02-22 08:56:40 -0800
tags:
  - rust
  - linux
  - macos
  - nixos
  - ffi
url:
  source: https://github.com/ifd3f/caligula
---

![A screenshot of Caligula working.](https://github.com/ifd3f/caligula/blob/main/images/verifying.png)

Caligula is a tool for burning ROMs to drives. It was made as a replacement for
`dd` and Balena Etcher, and was borne out of frustrations with both.

This is a non-exhaustive feature list:

- pretty graphs
- removable USB device detection
- compressed inputs
- SHA-verification of inputs
- automatically running `su`, `sudo`, or `doas` for you
- post-write disk verification

Supported systems:

- `/(aarch64|x86_64)-(darwin|linux)/`

## Motivation

My frustrations with `dd`:

- Being unixware, it is comically minimal as usual
- It won't detect removable USB devices for you
- You need to remember type in the same string of flags
- Lack of good progress indicators on all systems (no, `status=progress` is not
  good enough and it's also Linux-only)
- No builtin SHA verification
- No post-write correctness verification

My frustrations with Etcher:

- It's a 100MB electron blob
- It spies on you
- It's not a terminal app
- Seriously why do you need a web browser to write bytes to a disk
- For how often compressed ISOs are distributed, it's kinda shameful that

My frustrations with both:

- No pretty graphs
- NIH syndrome

## Development

The bulk of the project was completed in 9 days, during weekends, nights, and
lunch breaks. This got the project to a state where I could release it, with
features like:

- the Wizard
- removable USB device detection
- pretty graphs
- compressed inputs
- disk verification

There are features occasionally added, when I have the time and motivation to
add them.

## Reception

It's the first repo I've made that got over 100 stars on Github. Supposedly
people use it.
