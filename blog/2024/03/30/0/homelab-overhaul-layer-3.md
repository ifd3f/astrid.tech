---
title: "Homelab overhaul: routing, pxe, and failing at both"
tagline:
  "This product contains a device known to the State of California to cause dumb
  network issues"
tags:
  - homelab
  - networking
  - firewall
  - opnsense
  - pxe
  - series:homelab-overhaul
slug:
  ordinal: 0
  name: homelab-overhaul-layer-3
  date: 2024-03-30
date:
  created: 2024-03-28 21:00:05-07:00
  published: 2024-03-30 16:01:05-07:00
---

[So after I set up my VLANs](/2024/03/28/0/homelab-overhaul-layer-1-and-2), it
was time for me to have my OPNSense firewall route packets between them.

Here's the layer 2 and layer 3 layout of my network before changes:

![The firewall routes between vlan 10 and vlan 69. Then the wifi network is attached to vlan 10, and it's NATing everything because I was a lazy bum](https://s3.us-west-000.backblazeb2.com/nyaabucket/992be899592030c954af73ba2884de4fd981b88eedde78443630cb594c9ee5f1/old-layout.png)

You'll notice that there are many flaws with it:

- The wifi devices are being NAT-ed. So nothing in the user space can talk to
  the wifi devices. This is very bad because none of my ethernet devices can
  talk to wifi devices. Unfortunately, this is something that I did not address
  because I was too lazy, but I will address it later (tm)
- The server iLO is in the user subnet. This is very bad because it's literally
  accessible by anyone in the user subnet that's awful as hell

So I simply moved the server iLO to the new VLAN. Easy peasy, right?

## Routing problems

I connected to the port I set up on the living room switch specifically for
hooking into VLAN 69, the management VLAN. I was able to talk to the other
managed switch. However, OPNsense was not. Strangely though, it could talk to
the hypervisor machine it was on, because I did set something up for that a
while ago.

I tried a bunch of things, including wiresharking the interfaces, pinging from
the firewall (didn't work). I was tearing my hair out a ton, before I thought
about it for a moment.

Here's what inferno, they hypervisor looked like:

![Inferno contains the OPNsense VM lucifer. Lucifer gets 2 of the ethernet ports](https://s3.us-west-000.backblazeb2.com/nyaabucket/0a7e04250877b4bcef9e542d176afc02b2dae7f55ba66b53cc844dd8db799d9e.png)

I realized that perhaps I was doing a little _too_ much networking. I disabled
the bridge and just talked directly to the VLAN, and it worked -- though now I
couldn't communicate with the hypervisor. That's fine enough, I can deal with it
later.

## PXE booting

Well, now that that was done, I needed to set up my server. Laziness strikes
again, so I thought hey, what if I set up PXE boot to turn this server up
instead of walking downstairs?

Maybe I was just bad, but I didn't manage to get that working either. I tried
using tftp-hpa, atftpd, running the tftp server off the opnsense, but none of
that worked.

I spent 2 days trying to figure it out, but then gave up on 3/23, choosing to
just walk downstairs and stick a USB into the server myself. But then when
setting up the server from the USB, realized my VLANs were indeed wrong... so
maybe it was that all along...

I probably could have tried to actually set up PXE there, but I decided I had
enough of the damn boot fairies. I will just have to learn PXE another day,
perhaps when I reformat my currently-running server.

## Conclusion

whoa i am bad at setting things up

next i will talk about bonding
