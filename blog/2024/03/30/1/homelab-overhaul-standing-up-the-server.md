---
title: "Homelab overhaul: standing up the new server"
tagline:
  "Standing it up, and walking into EVEN MORE PROBLEMS. You might notice a
  recurring theme"
tags:
  - homelab
  - networking
  - lacp
  - initrd
  - nixos
  - booting
  - series:homelab-overhaul
slug:
  ordinal: 1
  name: homelab-overhaul-standing-up-the-server
date:
  created: 2024-03-30 17:00:00-07:00
  published: 2024-03-30 22:18:20-07:00
---

This is a continuation of my woes in overhauling my homelab.

## The server, and requirements

My server is a HP DL380P Gen9 that I acquired shortly after its end-of-support
date, likely from a Bosch datacenter[^bosch] that wanted to get rid of them.
Provisionally, I will designate this machine with the hostname boop, because I
don't have any better names for it.

[^bosch]:
    Based on the original iLO configurations, which had Bosch written all over
    them.

I had the following requirements for this server:

- RAID 1 NVME root partition
- Link aggregation
- Fully-encrypted drives
- Decryption by logging in during initrd phase

## The NVME drives

To have RAID 1, you need at least 2 drives. So I purchased a pair of Silicon
Power 1TB NVME drives. I thought, "hey, in order to save PCIE slots on the
server, what if I just did bifurcation?"

I looked it up, and it turns out my server _does_ support bifurcation! So I got
this kind of adapter to put the server in:

![One x8 PCIE card with two NVME slots on it](https://s3.us-west-000.backblazeb2.com/nyaabucket/cd3499c03e7395fe612a45c9a16a211c665f06b102b904c2a7f5ef148681583c/bifurcated-nvme.jpg)

I plugged it in, and after all the other crap I did on 3/23, I went into the
server's BIOS settings... and it turns out it only supports turning the x16 PCIE
into a pair of x8's.

Well, guess that didn't work. I'll have to occupy two whole PCIE slots 😭

![One x4 PCIE card with one NVME slot on it](https://s3.us-west-000.backblazeb2.com/nyaabucket/b4c063393041e07f275f488d7c3d39d902ea84fff4ded64969837e2f7da5e41b/non-bifurcated-nvme.jpg)

Those arrived on 3/24, and I installed them and proceeded with my NixOS. I'm
sure I will probably find a use for the bifurcation card at some point, just on
a different machine.

## Bonding over pain

boop was connected to the Brocade switch like so:

- iLO + eno1 went to VLAN 69, the management VLAN. This single ethernet is the
  management point for the server.
- the rest of the ethernets were connected to non-69 ports on the switch. These
  will be aggregated.

After booting up boop via USB, I thought to try setting up link aggregation for
it. To experiment, instead of setting it up via NixOS configs, I set it up via
the ip command.

I had the switch configured to output untagged packets, and have LACP on the
three ports (25, 26, 27), something like this:

```
vlan 100 name prod by port
 tagged ethernet 1/1/1
 untagged ethernet 1/1/25 to 1/1/27
exit

interface ethernet 1/1/25 to 1/1/27
 link-aggregate configure timeout short
 link-aggregate configure key 10001
 link-aggregate active
exit
```

Then, I ran a series of commands that looked something like this:

```sh
ip l add bond007 type bond mode 802.3ad lacp_rate fast lacp_active on
for i in eno2 eno3 eno4; do ip l set $i master bond007; done
ip l set bond007 up
```

This got DHCP all fine! But then when I tried pinging the firewall, it didn't
work. I was getting timeouts.

Capturing packets on the firewall, I saw packets, and it was sending replies
back. Even running tcpdump into wireshark on the server, I saw those packets
coming back. So I had no idea why ping wasn't seeing them, _especially_ since I
somehow got DHCP working over it!

I eventually ended up flipping the order -- instead of doing LACP over untagged
packets, I did VLAN tagging over LACP.

```
vlan 100 name prod by port
 tagged ethernet 1/1/1
 tagged ethernet 1/1/25 to 1/1/27
exit

interface ethernet 1/1/25 to 1/1/27
 link-aggregate configure timeout short
 link-aggregate configure key 10001
 link-aggregate active
exit
```

```sh
ip l add bond007 type bond mode 802.3ad lacp_rate fast lacp_active on
for i in eno2 eno3 eno4; do ip l set $i master bond007; ip l set $i up; done

ip l add link bond007 name bond007.100 type vlan id 100
```

Pinging on interface bond007.100 actually worked. I guess this switch really
does not like LACP with untagged ports, though perhaps I just set it up wrong.
This is fine, though; it's probably for the better, since the hypervisor should
be able to do this anyways.

In my NixOS configuration, I will set this up using systemd-networkd, though I
haven't done that yet.

## Decryption in initrd using SSH

I set up the root partition to be ZFS as I usually do. But, since this is a
server, I can't just walk up to the machine and type in my password, that would
be inconvenient. So, I needed to set up SSH during initrd.

[NixOS does have that option](https://search.nixos.org/options?channel=23.11&show=boot.initrd.network.ssh.enable).
However, getting it to work was quite annoying. Mostly, because they said that
[the host keys "are stored insecurely in the global Nix store"](https://search.nixos.org/options?channel=23.11&show=boot.initrd.network.ssh.hostKeys),
I thought that it would be fine to just have private keys in the repo if they're
going to be public anyways, something like this:

```nix
{
  boot.initrd.network.ssh = {
    enable = true;
    port = 2222; # because we are using a different host key
    hostKeys = [
      ./initrd/ssh_host_rsa_key
      ./initrd/ssh_host_ed25519_key
    ];
    authorizedKeys = inputs.self.lib.sshKeyDatabase.users.astrid;
  };
}
```

Apparently not, because those are actually filepaths, rather than derivations.
This would work, though:

```nix
{
  boot.initrd.network.ssh = {
    enable = true;
    port = 2222; # because we are using a different host key
    hostKeys = [
      (pkgs.writeText "ssh_host_rsa_key"
        (builtins.readFile ./initrd/ssh_host_rsa_key))
      (pkgs.writeText "ssh_host_ed25519_key"
        (builtins.readFile ./initrd/ssh_host_ed25519_key))
    ];
    authorizedKeys = inputs.self.lib.sshKeyDatabase.users.astrid;
  };
}
```

You would not believe how much pain I had to deal with because I _thought_
nixos-install worked successfully, but it actually just failed because it
couldn't find `/nix/store/blablabla-source/whatever/initrd/ssh_host_rsa_key` and
kept going through. It was late at night and I was not reading the logs very
carefully.

Of course, this is not ideal -- now the SSH keys are not merely in plaintext,
but now in my public git repo. Well, it also turns out that I misread the
documentation in my sleepiness _again_ -- that "stored insecurely in global Nix
store" part was qualified by "Unless your bootloader supports initrd secrets,"
and my bootloader appears to indeed support initrd secrets.

Probably I should just use what they use as an example and generate
machine-local keys:

```nix
[
  "/etc/secrets/initrd/ssh_host_rsa_key"
  "/etc/secrets/initrd/ssh_host_ed25519_key"
]
```

When I rebooted the machine... nothing happened. This is because I had to not
only `boot.initrd.ssh.network.enable`, but a couple other things:

```nix
{
  boot.initrd.network = {
    enable = true;
    udhcpc.enable = true;
  };
}
```

After I rebooted, udhcpcd was not able to find any network interfaces. Turns
out, that was because I needed to add the kernel module `tg3` to initrd, because
I'm using HP devices.

Rebooting again, I actually did get IP addresses and a shell!

```
> ssh root@192.168.69.206 -p 2222
~ # zfs list
NAME             USED  AVAIL  REFER  MOUNTPOINT
rpool           2.94G   920G    25K  none
rpool/enc       12.5M   920G   165K  none
rpool/enc/etc    668K   920G   668K  legacy
rpool/enc/home   854K   920G   854K  legacy
rpool/enc/tmp    145K   920G   145K  legacy
rpool/enc/var   10.7M   920G  10.7M  legacy
rpool/nix       2.92G   920G  2.92G  legacy
```

I just had to run `zfs load-key`.

```
~ # zfs load-key rpool/enc
Enter passphrase for 'rpool/enc':
~ #
```

... and nothing happened. Now what's happening?

The answer is, after running `zfs load-key` myself, the boot process was still
blocked, because it was calling `zfs load-key` too, just on the graphical
output.

```
~ # ps | grep zfs
  962 root      0:00 zfs load-key -a
  983 root      0:00 grep zfs
```

The solution to this is extremely easy :)

```
~ # kill 962
```

After this, I now have a blank system, open for me to set up whatever deranged
things I want!

## Conclusion

reading comprehension degrades when you are sleep-deprived
