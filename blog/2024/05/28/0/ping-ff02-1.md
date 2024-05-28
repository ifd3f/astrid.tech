---
title: ping ff02::1
tagline:
  Link-local IPv6 addresses and where to find them and cool ways to use them
tags:
  - networking
  - ipv6
  - linux
slug:
  ordinal: 0
  name: ping-ff02-1
date:
  created: 2024-05-28 03:42:06-08:00
  published: 2024-05-28 03:42:06-08:00
---

**TL;DR:** Link local addresses are great and can be used for lots of things,
like:

- finding and connecting to hosts that didn't get an address via DHCP
- transferring files directly between two computers via Ethernet with almost no
  extra configuration

To enumerate every IPv6-supporting device on your LAN, even if the device isn't
DHCP'd or SLAAC'd correctly, you can run a command like this:

```sh
ping ff02::1%$interface
```

For example:

```
% ping ff02::1%enp4s0
PING ff02::1%enp4s0(ff02::1%enp4s0) 56 data bytes
64 bytes from fe80::54a4:74f4:3735:33ad%enp4s0: icmp_seq=1 ttl=64 time=0.023 ms
64 bytes from fe80::da9d:67ff:fe26:463f%enp4s0: icmp_seq=1 ttl=64 time=0.243 ms
64 bytes from fe80::28c7:cb09:9bf6:1062%enp4s0: icmp_seq=1 ttl=64 time=0.293 ms
64 bytes from fe80::a02:8eff:fe9e:cf67%enp4s0: icmp_seq=1 ttl=64 time=0.796 ms
64 bytes from fe80::5054:ff:fed6:96c%enp4s0: icmp_seq=1 ttl=64 time=0.888 ms
64 bytes from fe80::768e:f8ff:feed:b700%enp4s0: icmp_seq=1 ttl=64 time=3.45 ms
^C
--- ff02::1%enp4s0 ping statistics ---
1 packets transmitted, 1 received, +5 duplicates, 0% packet loss, time 0ms
rtt min/avg/max/mdev = 0.023/0.949/3.453/1.160 ms
```

Some applications, like modern web browsers, don't support the `%$interface`
syntax, because they are made by stupid people. However, you can still work
around it with socat, like so in the case of TCP:

```sh
socat TCP6-LISTEN:$LOCAL_PORT_TO_FORWARD,fork "TCP6:[$LINK_LOCAL_IPV6%$YOUR_INTERFACE_NAME]:$REMOTE_TCP_PORT"
```

Examples:

```sh
socat TCP6-LISTEN:8000,fork "TCP6:[fe80::dead:beef%enowhatever]:80"
firefox http://localhost:8000
```

```sh
socat TCP6-LISTEN:8000,fork "TCP6:[fe80::dead:beef%enowhatever]:443"
firefox https://localhost:8000
```

The rest of the article will dive deeper into IPv6 link-local and multicast
addresses, and demonstrate a way to transfer files to your friend's computer
with directly-attached ethernet with absolutely no prior configuration besides
IPv6.

## Link local addresses

In IPv6, every device almost always has at least one address on every interface
it has -- the
**[link-local address](https://en.wikipedia.org/wiki/Link-local_address#IPv6)**.
Technically, the block `fe80::/10` is reserved for link-local addresses, but in
practice, only `fe80::/64` is really used.

Computers almost always assign themselves a link-local address when they set up
the network interface, unlike in IPv4, where computers _rarely_ do
this.[^ipv4-ll] Sometimes, the address is based on the MAC address, sometimes
it's completely random, sometimes you can even statically assign yourself a
link-local address!

[^ipv4-ll]:
    Yes, there are link-local IPv4 addresses in `169.254.0.0/16`. Most IPv4
    devices do not assign themselves one automatically, which is one of many
    other ways IPv4 sucks. We will pretend these don't exist because they only
    really seem to be used by routers for BGP extended-next-hop, and not normal
    people's computers.

You can see your link-local address on a modern Linux computer if you type in
`ip address` or any of its various aliases:

<pre>
% ip a
1: lo: &lt;LOOPBACK,UP,LOWER_UP&gt; mtu 65536 qdisc noqueue state UNKNOWN group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    inet 127.0.0.1/8 scope host lo
       valid_lft forever preferred_lft forever
    inet6 ::1/128 scope host noprefixroute
       valid_lft forever preferred_lft forever
2: enp4s0: &lt;BROADCAST,MULTICAST,UP,LOWER_UP&gt; mtu 1500 qdisc fq_codel state UP group default qlen 1000
    link/ether a8:a1:59:d6:b1:67 brd ff:ff:ff:ff:ff:ff
    inet 192.168.1.102/24 brd 192.168.1.255 scope global dynamic noprefixroute enp4s0
       valid_lft 6292sec preferred_lft 6292sec

    &lt;snip&gt;

    <b>inet6 fe80::54a4:74f4:3735:33ad/64 scope link noprefixroute</b>
       valid_lft forever preferred_lft forever
</pre>

It's marked as `scope link`, indicating it's a link local address. Even if you
already have a DHCP IPv4 address, or a SLAAC-assigned IPv6 address, you probably
still have a link-local IPv6 most of the time.

Let's try pinging it from another host on the subnet.

```
% ping fe80::54a4:74f4:3735:33ad
ping: Warning: IPv6 link-local address on ICMP datagram socket may require ifname or scope-id => use: address%<ifname|scope-id>
PING fe80::54a4:74f4:3735:33ad(fe80::54a4:74f4:3735:33ad) 56 data bytes
^C
--- fe80::54a4:74f4:3735:33ad ping statistics ---
8 packets transmitted, 0 received, 100% packet loss, time 7167ms
```

Uh oh, it didn't work. Why not?

The problem is that link-local addresses are meant to be restricted to a
specific link. One `fe80::whatever` on one link won't be the same on another
link, and these addresses are to never cross router boundaries, either. So,
that's why link-local addresses must be associated with an interface.

As `ping` suggests, you can instead try `address%<ifname|scope-id>`, like so:

```
% ping fe80::54a4:74f4:3735:33ad%eno1
PING fe80::54a4:74f4:3735:33ad%eno1(fe80::54a4:74f4:3735:33ad%eno1) 56 data bytes
64 bytes from fe80::54a4:74f4:3735:33ad%eno1: icmp_seq=1 ttl=64 time=0.182 ms
64 bytes from fe80::54a4:74f4:3735:33ad%eno1: icmp_seq=2 ttl=64 time=0.203 ms
64 bytes from fe80::54a4:74f4:3735:33ad%eno1: icmp_seq=3 ttl=64 time=0.178 ms
64 bytes from fe80::54a4:74f4:3735:33ad%eno1: icmp_seq=4 ttl=64 time=0.214 ms
^C
--- fe80::54a4:74f4:3735:33ad%eno1 ping statistics ---
4 packets transmitted, 4 received, 0% packet loss, time 3052ms
rtt min/avg/max/mdev = 0.178/0.194/0.214/0.014 ms
```

The value after the `%` is called the **zone index**.

Besides ping, most network applications do support link-local addresses:

```sh
ssh fe80::da9d:67ff:fe26:463f%enp4s0
```

```sh
scp ./a.out '[fe80::da9d:67ff:fe26:463f%enp4s0]:~'
```

```sh
rsync ./a.out '[fe80::da9d:67ff:fe26:463f%enp4s0]:~'
```

```sh
nc fe80::da9d:67ff:fe26:463f%enp4s0 80
```

```sh
curl 'http://[fe80::da9d:67ff:fe26:463f%enp4s0]'
```

However, some applications do _not_ support link-local addresses. In fact,
[most web browsers explicitly do not support this](https://stackoverflow.com/a/46881540),
which is a damn shame, especially if you have to access an admin page. However,
there is a workaround -- more on that later.

Link-local addresses themselves are not that useful. The way they are
auto-configured, and the way you can scan for them, _is_.

## Easily finding other link-local addresses on your subnet

Suppose you have a server or Raspberry Pi or whatever, you can't connect a
monitor to, and you've fucked up the network configuration and it's not getting
an IP address from DHCP. Or maybe your server has an out-of-band management page
that isn't DHCPing.

Out of luck, right? Nope!

If it supports IPv6, it probably assigned itself a link-local address. The only
question now is, which of the <m>2^{64}</m> addresses in `fe80::/64` did it give
itself?

Thankfully, link-local multicast addresses[^ll-mcast-list] are here to save the
day! Simply run:

```sh
ping ff02::1%$YOUR_INTERFACE_NAME
```

and that will send a ping to every link-local address on your subnet. And then,
every host will reply:

```
% ping ff02::1%enp4s0
PING ff02::1%enp4s0(ff02::1%enp4s0) 56 data bytes
64 bytes from fe80::54a4:74f4:3735:33ad%enp4s0: icmp_seq=1 ttl=64 time=0.023 ms
64 bytes from fe80::da9d:67ff:fe26:463f%enp4s0: icmp_seq=1 ttl=64 time=0.243 ms
64 bytes from fe80::28c7:cb09:9bf6:1062%enp4s0: icmp_seq=1 ttl=64 time=0.293 ms
64 bytes from fe80::a02:8eff:fe9e:cf67%enp4s0: icmp_seq=1 ttl=64 time=0.796 ms
64 bytes from fe80::5054:ff:fed6:96c%enp4s0: icmp_seq=1 ttl=64 time=0.888 ms
64 bytes from fe80::768e:f8ff:feed:b700%enp4s0: icmp_seq=1 ttl=64 time=3.45 ms
^C
--- ff02::1%enp4s0 ping statistics ---
1 packets transmitted, 1 received, +5 duplicates, 0% packet loss, time 0ms
rtt min/avg/max/mdev = 0.023/0.949/3.453/1.160 ms
```

Now you can sort through them and determine which host is which.

[^ll-mcast-list]:
    `ff02::1` is not the only link-local multicast address out there, but it is
    one of the more useful ones for debugging.
    [IANA has a whole list of other standard multicast addresses](https://www.iana.org/assignments/ipv6-multicast-addresses/ipv6-multicast-addresses.xhtml).

You don't even need to have an ethernet switch between the two machines! I've
even saved a friend's Raspberry Pi just by directly connecting its ethernet port
directly into my computer, pinging `ff02::1` to find the link-local IP it gave
itself, and SSHing into the discovered IP.

![Two plush sharks staring intently at a surface pro. The surface pro has an ethernet dongle connected directly into a raspberry pi.](https://s3.us-west-000.backblazeb2.com/nyaabucket/f12beedd22dda999dd4c895bd70936a9544e194276f49d9625f90f505c795304/pi-rescue-reenactment.jpg "A reenactment of the raspberry pi rescue.")

## Workaround for web browsers not supporting link-local addresses

Let's say I've determined that `fe80::5054:ff:fed6:96c%enp4s0` is a router with
an admin page. Technically, URLs actually do support this syntax, you just need
to escape the `%` as a `%25`, like so:
<http://[fe80::5054:ff:fed6:96c%25enp4s0]>

However, if you're using a modern web browser, this is most likely completely
unsupported. Try clicking on it. I'm quite sure that it won't work for you.
Firefox simply refuses to take me to it, and Chrome sends me to
`about:blank#blocked`.

As a workaround, you can forward the port with a socat command like this:

```sh
socat TCP6-LISTEN:8000,fork "TCP6:[fe80::5054:ff:fed6:96c%enp4s0]:443"
```

Explanation of arguments:

- `TCP6-LISTEN:8000,fork` - listen on TCP/IPv6 port 8000. when you see a new
  connection, fork a process to handle it and keep listening.
- `TCP6:[fe80::5054:ff:fed6:96c%enp4s0]:443` - after you've got a connection on
  port `8000`, open a connection to `[fe80::5054:ff:fed6:96c%enp4s0]:443`, and
  forward data to it.

Once you have that running in the background, you can go to
<https://localhost:8000> and look at your beautiful admin page.

## Sending files over an ethernet cable with minimal configuration

Tired of using USB sticks to transfer files between laptops? Connect your two
computers directly like this:

![My surface pro directly connected to my partner's Thinkpad via ethernet cable and USB to ethernet adapter.](https://s3.us-west-000.backblazeb2.com/nyaabucket/df978c5ae673ed0748f9b3f0a4e412493e2a21f978b9a44785a00d4d379bff77/direct-ethernet-attachment.jpg "yuri")

You can determine the link-local IP by typing `ip a` on both computers and
manually typing 26 characters in... or you can be lazy, ping `ff02::1`, and
copy/paste:

```
% ping ff02::1%enp0s20f0u1c2
PING ff02::1%enp0s20f0u1c2(ff02::1%enp0s20f0u1c2) 56 data bytes
64 bytes from fe80::8a40:4905:d926:861c%enp0s20f0u1c2: icmp_seq=1 ttl=64 time=0.112 ms
64 bytes from fe80::29b8:771a:dd16:fb28%enp0s20f0u1c2: icmp_seq=1 ttl=64 time=1.60 ms  # this is the other computer, time was much longer
^C
--- ff02::1%enp0s20f0u1c2 ping statistics ---
1 packets transmitted, 1 received, +1 duplicates, 0% packet loss, time 0ms
rtt min/avg/max/mdev = 0.112/0.855/1.598/0.743 ms

% rsync -rv my-silly-files/ [fe80::29b8:771a:dd16:fb28%enp0s20f0u1c2]:/wherever
```

### hang on i didn't set up any infrastructure up how can i do this help

What if you don't have SSH perms on the other machine because it's your friend's
machine, and you have a device-local firewall blocking any attempts at running
`nc -l`, and you're too lazy to set up a firewall allow rule?

Well, you can do a little "TCP hole-punching" trick using netcat. Simultaneously
on both computers, run the following command to perform a TCP simultaneous-open:

```sh
nc -v $IP%$INTERFACE $PORT -p $PORT
```

where:

- `$IP` is the other machine's IP
- `$INTERFACE` is the interface you're using
- `$PORT` is agreed upon between BOTH OF YOU. List it
  TWICE.[^simultaneous-open-port] `-p` tells netcat to use that specific source
  port.
- The `-v` is useful because it will tell you how successfull it was.

[^simultaneous-open-port]:
    You can technically do it asymmetrically (i.e. having one computer's port be
    10003 and another computer's port be 10004), but having the same port is
    just easier.

Example:

```
astrid@shai-hulud% nc -v fe80::29b8:771a:dd16:fb28%enp0s20f0u1c2 10000 -p 10000
Connection to fe80::29b8:771a:dd16:fb28%enp0s20f0u1c2 10000 port [tcp/ndmp] succeeded!
```

```
alia@stargazer% nc -v fe80::8a40:4905:d926:861c%enp0s31f6 10000 -p 10000
Connection to fe80::8a40:4905:d926:861c%enp0s31f6 10000 port [tcp/ndmp] succeeded!
```

And congratulations, you now have a shitty version of IRC running over that
ethernet cable.

If you want to transfer files, just pipe them in on the source:

```sh
nc -v $IP%$INTERFACE $PORT -p $PORT < ./my-silly-file
```

and pipe them out on the destination:

```sh
nc -v $IP%$INTERFACE $PORT -p $PORT > ./my-silly-file
```

If you have multiple files, you can `tar` them. If you want to go faster, you
could `gzip` them. The possibilities are endless.

## Conclusion

ipv6 gives you lots of cool auto-configured goodies

## Appendix: hang on, isn't `ff02::1` basically just a broadcast?

In general, IPv6 multicast requires usage of
[Multicast Listener Discovery (MLD)](https://en.wikipedia.org/wiki/Multicast_Listener_Discovery)
-- essentially, hosts ask the router to subscribe to a multicast address, so we
can minimize the number of packets we transmit. But `ff02::1` is special -- it
does _not_ need MLD or any kind of subscription to work.

So what distinguishes it from a broadcast?

Here's Wireshark's dissection of that ICMP request to `ff02::1` from earlier:

<pre>
Frame 449: 118 bytes on wire (944 bits), 118 bytes captured (944 bits) on interface enp4s0, id 0
Ethernet II, Src: ASRockIn_d6:b1:67 (a8:a1:59:d6:b1:67), Dst: IPv6mcast_01 (33:33:00:00:00:01)
    Destination: IPv6mcast_01 (33:33:00:00:00:01)
        Address: IPv6mcast_01 (33:33:00:00:00:01)
        .... ..1. .... .... .... .... = LG bit: Locally administered address (this is NOT the factory default)
        <b>.... ...1 .... .... .... .... = IG bit: Group address (multicast/broadcast)</b>
    Source: ASRockIn_d6:b1:67 (a8:a1:59:d6:b1:67)
        Address: ASRockIn_d6:b1:67 (a8:a1:59:d6:b1:67)
        .... ..0. .... .... .... .... = LG bit: Globally unique address (factory default)
        .... ...0 .... .... .... .... = IG bit: Individual address (unicast)
    Type: IPv6 (0x86dd)
Internet Protocol Version 6, Src: fe80::54a4:74f4:3735:33ad, Dst: ff02::1
    0110 .... = Version: 6
    .... 0000 0000 .... .... .... .... .... = Traffic Class: 0x00 (DSCP: CS0, ECN: Not-ECT)
    .... 0010 0000 0011 0111 0011 = Flow Label: 0x20373
    Payload Length: 64
    Next Header: ICMPv6 (58)
    Hop Limit: 1
    Source Address: fe80::54a4:74f4:3735:33ad
    Destination Address: ff02::1
Internet Control Message Protocol v6
    Type: Echo (ping) request (128)
    Code: 0
    Checksum: 0x7e2a [correct]
    [Checksum Status: Good]
    Identifier: 0x0001
    Sequence: 1
    Data (56 bytes)
</pre>

This is sent to the MAC address `33:33:00:00:00:01`. Where the hell does that
come from?
[IETF RFC 2464 §7](https://datatracker.ietf.org/doc/html/rfc2464#section-7),
which specifies how to map multicast IPv6's into multicast MAC addresses:

> An IPv6 packet with a multicast destination address DST, consisting of the
> sixteen octets DST[1] through DST[16], is transmitted to the Ethernet
> multicast address whose first two octets are the value 3333 hexadecimal and
> whose last four octets are the last four octets of DST.
>
> ```
>                 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
>                 |0 0 1 1 0 0 1 1|0 0 1 1 0 0 1 1|
>                 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
>                 |   DST[13]     |   DST[14]     |
>                 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
>                 |   DST[15]     |   DST[16]     |
>                 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
> ```

Further note the bolded Group Address bit from above. That specific bit being 1
signals to ethernet switches "hey, this needs to be sent to multiple hosts."
It's not just `ff:ff:ff:ff:ff:ff` that has this behavior, any MAC with the group
bit set will be treated like this.

According to
[IETF RFC 4541 §1](https://datatracker.ietf.org/doc/html/rfc4541#section-1),
most "dumb" switches will effectively treat any packet with that bit enabled,
multicast or broadcast, as a broadcast, and simply forward the packet to every
single device on the layer 2 network. Of course, this would suck if you are
using IPv6, because it takes advantage of multicast very heavily to reduce
congestion. IGMP snooping was introduced to have switches (layer 2 devices)
"snoop" on the IP headers above them (layer 3 data) and _not_ forward
multicasted packets to devices that don't care about it.

However, [§3](https://datatracker.ietf.org/doc/html/rfc4541#autoid-7) recommends
that `ff02::1` specifically should be ignored for IGMP purposes and be treated
as simply a broadcast:

> In IPv6, the data forwarding rules are more straight forward because MLD is
> mandated for addresses with scope 2 (link-scope) or greater. The only
> exception is the address FF02::1 which is the all hosts link-scope address for
> which MLD messages are never sent. Packets with the all hosts link-scope
> address should be forwarded on all ports.

So yes, even though it is _technically_ a multicast, and it isn't addressed to
`ff:ff:ff:ff:ff:ff`, it is _effectively_ a broadcast in all but the most painful
of vendor hardware.

As for why they didn't just send it to `ff:ff:ff:ff:ff:ff`, it's probably to
keep consistent with RFC 2464. I would imagine that `33:33:00:00:00:01` would
_theoretically_ be beneficial over `ff:ff:ff:ff:ff:ff` because switches could
simply not forward those to IPv4-only hosts. In practice, I don't know how often
that's used, or if it's used at all.

The reply packet, however, is merely unicasted, and does not have to deal with
any of these technicalities.

<pre>
Frame 452: 118 bytes on wire (944 bits), 118 bytes captured (944 bits) on interface enp4s0, id 0
Ethernet II, Src: HewlettP_26:46:3f (d8:9d:67:26:46:3f), Dst: ASRockIn_d6:b1:67 (a8:a1:59:d6:b1:67)
    Destination: ASRockIn_d6:b1:67 (a8:a1:59:d6:b1:67)
        Address: ASRockIn_d6:b1:67 (a8:a1:59:d6:b1:67)
        .... ..0. .... .... .... .... = LG bit: Globally unique address (factory default)
        <b>.... ...0 .... .... .... .... = IG bit: Individual address (unicast)</b>
    Source: HewlettP_26:46:3f (d8:9d:67:26:46:3f)
        Address: HewlettP_26:46:3f (d8:9d:67:26:46:3f)
        .... ..0. .... .... .... .... = LG bit: Globally unique address (factory default)
        .... ...0 .... .... .... .... = IG bit: Individual address (unicast)
    Type: IPv6 (0x86dd)
Internet Protocol Version 6, Src: fe80::da9d:67ff:fe26:463f, Dst: fe80::54a4:74f4:3735:33ad
    0110 .... = Version: 6
    .... 0000 0000 .... .... .... .... .... = Traffic Class: 0x00 (DSCP: CS0, ECN: Not-ECT)
    .... 1001 1100 0000 0101 0001 = Flow Label: 0x9c051
    Payload Length: 64
    Next Header: ICMPv6 (58)
    Hop Limit: 64
    Source Address: fe80::da9d:67ff:fe26:463f
    Destination Address: fe80::54a4:74f4:3735:33ad
    [Source SLAAC MAC: HewlettP_26:46:3f (d8:9d:67:26:46:3f)]
Internet Control Message Protocol v6
    Type: Echo (ping) reply (129)
    Code: 0
    Checksum: 0xf6a9 [correct]
    [Checksum Status: Good]
    Identifier: 0x0001
    Sequence: 1
    Data (56 bytes)
</pre>

**TL;DR:** you can only call it a broadcast if it comes from the
`ff:ff:ff:ff:ff:ff` region of France, otherwise it's just sparkling multicast
