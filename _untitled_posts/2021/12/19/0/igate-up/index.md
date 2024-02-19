---
title: null
tags:
  - ham-radio
  - aprs
  - docker
  - project:infrastructure
slug:
  date: 2021-12-19
  ordinal: 0
  name: igate-up
date:
  created: 2021-12-19 08:19:28+00:00
  published: 2021-12-19 08:19:28+00:00
---

As promised from
[my frustrations while taking a walk](https://astrid.tech/2021/11/21/1/aprs-walk/),
I finally got an I-Gate set up in my bedroom. Sadly not a digipeater (_**yet**_)
but it's something!

<!-- excerpt -->

![The definitely-not-shabby setup.](./the-igate.jpg)

It is powered by [Dire Wolf](https://github.com/wb2osz/direwolf), which runs
inside a Docker container
[`w2bro/direwolf`](https://hub.docker.com/r/w2bro/direwolf), which runs on
[Armbian](https://www.armbian.com/), which runs on an
[Orange Pi One SBC](http://www.orangepi.org/orangepione/). I was originally
gonna run NixOS for this but as it turns out, cross-compiling an armv7l SD image
is hard, so I just went with the Docker route.
