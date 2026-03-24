---
title: The little chicken shed that could
tagline: The bikeshed of the bikeshed that became too important
tags:
  - linux
  - nixos
  - kexec
  - boot
slug:
  ordinal: 3
  name: chicken-shed
date:
  created: 2026-03-16 18:11:00-07:00
  published: 2026-03-24 02:10:01-07:00
---

_This is part 3 of an article series about how to reimage a disk in-place, how
to do weird things in initrd, and… well, now I guess it's about how to build
your own Linux distro. [Part 0 is located here.](../../0/curl-to-dev-sda)_

Some of you are about to angrily comment or email something like “grrrr!
[NixOS Anywhere](https://github.com/nix-community/nixos-anywhere) already does
[that kexec technique you were describing in part 2](../../2/how-to-persist-volatile-memory#introducing-curlingiron)
and you STOLE it from them! grrrrrrr!”

I would like to respond with a correction, which is that I did not just steal
their _technique_, but _I stole their entire system._

No, really -- where do you think that Nix store comes from?

## Dropshipping NixOS Anywhere

First, I wrote a bash script, `curlingiron.sh`:

```sh
#!/usr/bin/env bash

kernel_params="$(cat /proc/cmdline)"

src="$(echo "$kernel_params" | sed -n 's/.*curlingiron.src=\([^ ]*\).*/\1/p')"
dst="$(echo "$kernel_params" | sed -n 's/.*curlingiron.dst=\([^ ]*\).*/\1/p')"

curl -L "$src" | dd bs=1M of="$dst"
```

Then, with the attitude of someone building a plywood chicken shed onto the
first house they saw, I put this systemd service into a NixOS config that was
almost completely ripped off from
[theirs](https://github.com/nix-community/nixos-images/blob/main/nix/kexec-installer/module.nix):

```nix
# A systemd service that runs the actual curlingiron script
systemd.services.curlingiron = {
  script = builtins.readFile ./curlingiron.sh;
  serviceConfig = {
    # Need to set up restarting because network-online doesn't actually
    # mean "we can talk to the internet".
    # For more info, see https://systemd.io/NETWORK_ONLINE/
    Restart = "on-failure";
    RestartSec = 5;
  };
  requires = [ "network-online.target" ];
  wantedBy = [ "multi-user.target" ];
};
```

I hacked on it, got the kexec chain working, and I was very happy and did a
silly little dance.

The next day, I started working on the outlines of parts 1-3 and wrote down that
I should put in a “go download my funny initrd” section.

Now, most initrds are fairly small. Debian's is 35MB, my NixOS daily driver's is
19MB. Wanna know how big this one was?

```
astrid@chungus curlsda ❯ ls -lhLF result/curlingiron/
total 490M
-r--r--r-- 2 root root  14M Dec 31  1969 bzImage
-r--r--r-- 2 root root 477M Dec 31  1969 initrd.gz
-r-xr-xr-x 2 root root  585 Dec 31  1969 kexec-boot*
```

Horror dawned on me as I realized that the house I had built my chicken shed
onto wasn't just a house, it was a 477MB nuclear power plant! Oh god, I can't
just go around handing those out! People would be angry, and worse, GitHub Pages
would throttle me!

## Desperately trimming the fat

I used [nix-tree](https://github.com/utdemir/nix-tree) to browse curlingiron's
dependency closure, and found some obvious things to axe. For example, we don't
need ntfs3g, ZFS, sshfs, NetworkManager, `nixos-install`, `nixos-enter`, or
`nixos-generate-config`!

If you're thinking “hey wait a minute, these sound like things that installers
need,” that's because NixOS Anywhere is a fully-featured unattended installer!

I tore a bunch of those out and got… **441MB** (-36MB). Not great, not terrible.

## Desperately trimming the walls

I proceeded to madly run around its
[nix-tree](https://github.com/utdemir/nix-tree) with a sledgehammer, searching
for literally dependencies to pull off, and found some really funny ones.

### Delete Nix

Did you know that you can disable Nix on the OS famous for using Nix as its
package manager and configuration language?

```nix
nix.enable = false;
```

Doing so took out a whopping **121MB,** bringing me down to **320MB.**

### Delete bloat

sudo, vim, nano, and other useless things like those seem to be included in
`environment.systemPackages` by default. How do you remove them?

Thank god for `lib.mkForce`.

```nix
environment.systemPackages = with pkgs; lib.mkForce [ bash ];
```

Doing this took out **28MB** down to **292MB.** So much progress, yet still not
enough!

## Desperately trimming the nuclear reactor

No matter what I did, I kept running into the 170MB elephant in the room --
systemd. [Jade (of .fyi fame)](https://jade.fyi/) suggested I try using the
systemd minimal variant packaged with nixpkgs.

What's the size delta? ~100MB?

```
astrid@chungus curlsda ❯ nix path-info -Sh nixpkgs#systemd nixpkgs#systemdMinimal
/nix/store/1vs3gbz4w3wrqs76z8iay5cidwrv2hy6-systemd-258.3                171.8M
/nix/store/83ihzry3x75f469s4gj44nlqn94fncc3-systemd-minimal-258.3         66.9M
/nix/store/hksy4h7mxnbc66a5g3kb9pdh4ggjiqm1-systemd-minimal-258.3-man      1.2M
/nix/store/iz73qi2sbgg8yfnm4r10xy4k89mpbc86-systemd-258.3-man              1.8M
```

Eh, why not.

### Let's get rid of some control rods!

By convention, most NixOS config nodes provide a `.package` option that you can
use to override the package. This is useful if, say, you want to compile a
package in a certain way, or use one of its drop-in replacements. Conveniently
enough, `systemd` provides one too, so I tried dropping it in.

```nix
systemd.package = pkgs.systemdMinimal;
```

`nix build` resulted in an error.

```
astrid@🌐 chungus curlsda ❯ nix build .#nixosConfigurations.curlingiron.config.system.build.kexecTree
warning: Git tree '/home/astrid/Documents/curlsda' is dirty
error: builder for '/nix/store/5hl4z4j7zkl411d8g2li7hy7vhfjxcwx-initrd-units.drv' failed with exit code 1;
       last 1 log lines:
       > missing /nix/store/g5kvz35643fp9yhxnkwcnwkkrhyj6d71-systemd-minimal-259/example/systemd/system/systemd-bsod.service
...
```

So I proceeded to go on a mad get-error-search-nixpkgs-disable-option-new-error
loop.

```nix
# that bsod thing from above
boot.initrd.systemd.suppressedUnits = [
  "systemd-bsod.service"
];
boot.initrd.systemd.suppressedStorePaths = [
  "${pkgs.systemdMinimal}/lib/systemd/systemd-bsod"
];
```

I was making progress! I was pulling out the control rods! It was finally coming
down!

```nix
services.timesyncd.enable = false;
systemd.oomd.enable = false;
services.udev.enable = false;
systemd.coredump.enable = false;
```

Then I got to systemd-logind and readied my sledgehammer. I opened up its
[module](https://github.com/NixOS/nixpkgs/blob/b963f9244dfb52f5970f08e356bb7a4114dca976/nixos/modules/system/boot/systemd/logind.nix),
excitedly searching around for the off switch... and found that it does not have
one.

## Bargaining with the Nuclear Regulatory Commission

Okay. It's hardcoded, but that doesn't have to be the end, right? I could submit
a patch, or at least make a private fork of nixpkgs. It could be as easy as
adding a `systemd.logind.enable` flag!

…but it could also spiral out of control if it turns out to be more involved
than that…

I could try somehow manually removing all 10-ish of those config nodes that it
defines! I'm sure there's ways!

…though they probably all involve arcane `lib` functions that I don't feel like
searching for…

I took a step back and saw that what I had left wasn't a chicken shed hacked
onto an nuclear elephant -- it had already exploded, and I was left with just
the elephant's foot.

Remember that `curl > /dev/sda` thing, that initial chicken shed that I was
talking about way back in part 1? It really only needs to read from the internet
and write to a disk, and I didn't need NixOS to do that.

## Building a standalone chicken shed

When people want to make tiny Linux distros, the tools that usually come to mind
are Buildroot and Yocto. I did some cursory research, and then remembered
something – I don't need to learn another toolchain for this stuff, Nix already
has all the components I need!

It can't be too hard to build your own initramfs from scratch.

Right?

### Building the frame

While spelunking NixOS, I encountered a function called `pkgs.makeInitrdNG` that
does exactly what you think it does, which is build an initramfs, not an initrd.
You can learn how to use it by reading
[its documented source code](https://github.com/NixOS/nixpkgs/blob/127473ff3102f1d1c4804b54dc557a6a01d26a68/pkgs/build-support/kernel/make-initrd-ng.nix).

Of course, that only gave me the frame, which I kept kicking at angrily because
I kept running into this kernel panic.

```
[    1.908619] List of all partitions:
[    1.908854] No filesystem could mount root, tried:
[    1.908869]
[    1.909479] Kernel panic - not syncing: VFS: Unable to mount root fs on "" or unknown-block(0,0)
[    1.910078] CPU: 0 UID: 0 PID: 1 Comm: swapper/0 Not tainted 6.18.15 #1-NixOS PREEMPT(voluntary)
[    1.910410] Hardware name: QEMU Standard PC (i440FX + PIIX, 1996), BIOS rel-1.17.0-0-gb52ca86e094d-prebuilt.qemu.org 04/01/2014
[    1.910878] Call Trace:
[    1.911022]  <TASK>
[    1.911336]  dump_stack_lvl+0x5d/0x80
[    1.911681]  vpanic+0xdb/0x2d0
[    1.911774]  panic+0x6b/0x6b
[    1.911854]  mount_root_generic+0x293/0x2b0
[    1.911984]  prepare_namespace+0x1dc/0x230
[    1.912086]  kernel_init_freeable+0x27c/0x290
[    1.912186]  ? __pfx_kernel_init+0x10/0x10
[    1.912300]  kernel_init+0x1a/0x130
[    1.912394]  ret_from_fork+0x1cb/0x200
[    1.912500]  ? __pfx_kernel_init+0x10/0x10
[    1.912618]  ret_from_fork_asm+0x1a/0x30
[    1.912775]  </TASK>
[    1.913296] Kernel Offset: 0x21400000 from 0xffffffff81000000 (relocation range: 0xffffffff80000000-0xffffffffbfffffff)
[    1.913803] ---[ end Kernel panic - not syncing: VFS: Unable to mount root fs on "" or unknown-block(0,0) ]---
QEMU: Terminated
```

Can you figure out what's wrong here, and what
`Unable to mount root fs on "" or unknown-block(0,0)` means?

Obviously, it has nothing to do with mounting anything. It meant my Nix config
accidentally pointed `/init` to a directory.

### Special filesystems

You may be familiar with `/dev`, `/proc`, and `/sys` being magic directories
that contain information about the OS itself.

Well, did you know that they're not there by default, and that the init process
has to actually mount them itself? I was reminded of that the hard way.

```sh
# Mount the very important directories
mkdir -p /dev /proc /sys
mount -t devtmpfs devtmpfs /dev
mount -t proc procfs /proc
mount -t sysfs sysfs /sys
```

### Always thank your bus drivers

On most normal Linux systems, `udev` will find all of your peripherals and run
`modprobe` with the correct kernel driver. We don't have that luxury, and I was
too lazy to figure out how to stick `udev` rules into this thing, so I had to
manually run `modprobe` myself.

```sh
# load core bus drivers first
modprobe pci_hotplug 2>/dev/null
modprobe usbcore 2>/dev/null
modprobe xhci_hcd 2>/dev/null
modprobe ehci_hcd 2>/dev/null

# then scan all devices
for f in /sys/bus/pci/devices/*/modalias; do
    modprobe -b "$(cat $f)" 2>/dev/null
done

# USB devices won't appear until USB host loaded, rescan
for f in /sys/bus/usb/devices/*/modalias; do
    modprobe -b "$(cat $f)" 2>/dev/null
done
```

What's in those `modalias`es? Fuck if I know!

```
pci:v00008086d000024DBsv0000103Csd0000006Abc01sc01i8A
```

I don't know or care how `modprobe` turns this incomprehensible string into a
kernel module, but if you do,
[Arch Wiki has an explanation](https://wiki.archlinux.org/title/Modalias).

### who need they SCSI ate

I learned that the sd in `/dev/sda` stands “SCSI Disk” after several rounds of
failing to find `/dev/sda`.[^1]

[^1]:
    Similarly, CD-ROMs are named things like `/dev/sr0`, and that's because `sr`
    stands for SCSI ROM.

```sh
# SCSI disk layer (doesn't come from modalias)
modprobe sd_mod 2>/dev/null
modprobe sr_mod 2>/dev/null
```

### Networking

Do you think the Linux kernel just gives you internet access? Of course not.
DHCP is a userspace thing, but luckily
[Busybox gives that to you](https://udhcp.busybox.net/).

```sh
# Turn on network interface and DHCP clients
ip link set eth0 up
udhcpc --script /etc/dhcpevent.sh &
udhcpc6 --script /etc/dhcpevent.sh &
```

### she af on my \_ till i packet

Except actually, I had to add this line before the DHCP shit because the Linux
kernel in Nixpkgs doesn't have it enabled by default???

```sh
# The kernel packaged with Nix apparently doesn't do this???
modprobe af_packet 2>/dev/null
```

### ugh

```sh
# Wait until we have any amount of connectivity. This is because
# DHCP might not be up by the time we get here, so we have to wait.
until wget --spider "$src"; do
  echo "Connection failed, waiting until we have connectivity"
  sleep 5
done
```

### fuck my life

```sh
reboot -f # needs -f because Init Shenanigans
```

## The final result

There is a parable about a committee arguing about bikeshed at a nuclear power
plant. Even though the nuclear power plant is clearly the more important thing,
the committee spends more time talking about the bike shed than the power plant.

The solution to bikeshedding is usually focus on the nuclear power plant and
drop the bike shed.

I guess you could call what I did "chickenshedding": I dropped the nuclear power
plant and only shipped the chicken shed.

```
astrid@🌐 chungus ~  ❯ du -sh /tmp/curlingiron.initrd
6.1M	/tmp/curlingiron.initrd
```

From a 292MB initramfs, we now have a **6.1MB** initramfs, made entirely to run
`busybox wget | dd`, and smaller than almost every other distro's initramfs.

It doesn't check SSL certs because the default Busybox `wget` provided by
Nixpkgs doesn't check SSL certs.

And it's a full OS built in Nix, but because I don't use the NixOS module system
whatsoever, it's _not_ a NixOS.[^2]

Oh, and remember when I hinted that writing about the `curl > /dev/sda` trick
would lead me to build a Linux distro?

This is it. This is the distro.[^3]

The distro isn't the _argument_ to `curl > /dev/sda`. _It's the fucking
operator_.

The source code is [here](https://github.com/ifd3f/curlsda), and I leave you
with one more mystery:

```sh
curl https://astrid.tech/rkx.gz | gunzip | sudo sh
```

What function this serves is left as an exercise for the reader.

[^2]:
    And unlike NixOS, this one follows
    [FHS](https://en.wikipedia.org/wiki/Filesystem_Hierarchy_Standard)!
    [Just look at the tree output in part 2.](../../2/persist-volatile-memory-in-ram/#header-and-now-for-the-gore)
    /bin and /lib are symlinks, but they're also symlinks on Debian due to
    [UsrMerge](https://systemd.io/THE_CASE_FOR_THE_USR_MERGE/). You can't win.
    Nobody can win.

[^3]:
    It comes with a kernel, userspace, and package manager! You just run the
    package manager _before_ boot, not after.
