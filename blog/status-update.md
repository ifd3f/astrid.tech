---
title: Homelab redesign from the ground up
tagline: Patching security holes by putting in new ones
tags:
  - homelab
  - networking
slug:
  ordinal: 0
  name: woohoo-another-homelab-redesign
date:
  created: 2024-03-19 23:57:45-07:00
  published: 2024-03-19 23:57:45-07:00
---

## State before

This is how my network looked before the big changes I made.

```dot
graph G {
    node [shape=box]

    internet [label="Internet", shape=oval]
    sonic [label="Sonic (ISP)", shape=oval]
    ispmodem [label="ISP-provided\nfiber modem"]
    inferno [label="Firewall hypervisor"]
    belphegor [label="Belphegor\n(living room mgd. switch)"]
    officeswitch [label="Office switch"]
    nyaanet [label="Nyaanet (WAP)"]
    dcswitch [label="Datacenter switch\n(shitty 24port)"]
    gfdesk [label="gfdesk (rack server)"]
    partnerpc [label="Partner's PC"]
    astridpc [label="My PC"]

    internet -- sonic
    sonic -- ispmodem [label="fiber"]
    ispmodem -- inferno -- belphegor -- dcswitch
    belphegor -- nyaanet
    nyaanet -- {phones, laptops, printer} [style=dotted, label="802.11"]
    belphegor -- officeswitch -- {partnerpc, astridpc}
    dcswitch -- gfdesk [label="eno1"]
    dcswitch -- gfdesk [label="iLO"]
}
```

## Issues I ran into

### Firewall can't route between VLAN 10 and VLAN 69?

https://s3.us-west-000.backblazeb2.com/nyaabucket/0a7e04250877b4bcef9e542d176afc02b2dae7f55ba66b53cc844dd8db799d9e.png
