---
title: Running wireshark live on a remote host
tags:
  - wireshark
  - networking
  - cybersecurity
  - blogumentation
slug:
  date: 2022-04-29
  ordinal: 0
  name: til-remote-wireshark
date:
  created: 2022-04-28 22:00:18-07:00
  published: 2022-04-28 22:00:18-07:00
  updated: 2024-04-15 16:10:34-07:00
---

TIL that you can analyze live packets from remote network interfaces in
Wireshark with the following command:

```bash
ssh $SSH_TARGET "tcpdump -w- -U -i $REMOTE_INTERFACE" | wireshark -k -i-
```

It's essentially three commands glued together in a big pipe.

On the left side of the pipe, we have the following to connect to the machine
and capture packets in pcap format:

- `ssh $SSH_TARGET ...` means "SSH into the machine and execute the given
  argument"
- `tcpdump -w- -U -i $REMOTE_INTERFACE`, which is executed in on the remote,
  works like so:
  - `-w` selects a file to write to. In this case, `-w-` means "write to STDOUT"
  - `-U` instructs wireshark to not buffer its output. This way, you can see
    packets in wireshark as they arrive, instead of having to wait.
  - `-i $REMOTE_INTERFACE` specifies the interface to capture on

On the right side of the pipe, there is `wireshark -k -i-`.

- `-k` means "start capturing immediately, don't bring up the start menu"
- `-i` selects an interface or file to get packets from. In this case, `-i-`
  means "capture packets from STDIN"

**Related note:** Oftentimes, when a flag that usually expects a file gets a
`-`, the program supports reading the data from STDIN, or writing the data to
STDOUT, depending on what the flag actually does. If you want to use the file
specifically named `-`, you should provide `./-` in that argument (i.e.
`-w ./-`)

In addition, you may want to specify filters on `tcpdump` to ignore packets from
your own device, if you are running SSH on the same interface you're monitoring.
Otherwise, you may accidentally amplify your packets.

**EDIT 2024-04-15:** add `-U` flag
