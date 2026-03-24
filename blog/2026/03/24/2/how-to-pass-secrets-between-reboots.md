---
title: How to pass secrets in RAM between reboots
tagline: This happens in production all the time
tags:
  - linux
  - boot
slug:
  ordinal: 2
  name: how-to-pass-secrets-between-reboots
date:
  created: 2026-03-16 18:11:00-07:00
  published: 2026-03-24 02:10:02-07:00
  updated: 2026-03-24 07:03:00-07:00
---

_This is part 2 of a four-part article series about how to reimage a disk
in-place, and how to do weird things in initrd.
[Part 0 is located here](../../0/curl-to-dev-sda)._

At a high level, when a Linux machine reboots, the following things end up
happening:

1.  Init process shuts down all other processes and all writes are flushed to
    disk
2.  The init process requests that the kernel reboot, so the kernel sends an
    ACPI reset to the machine
3.  The firmware on the machine needs to POST (power-on self test), and it also
    needs to clear the RAM so that the next OS doesn't steal data from the
    previous OS
4.  Once POST is over, the firmware executes the bootloader
5.  The bootloader executes the kernel
6.  Kernel and initrd will then meow meow meow meow meow meow go read
    [the last part](../../1/swap-out-the-root-before-boot/#header-putting-it-all-together)
7.  Successfully rebooted, yay!

Step 3 is absolutely terrible for you if you are a server owner and have to
apply kernel updates, because servers can sometimes take an entire fucking hour
to POST.[^1]

[^1]:
    This is due to having to do a lot more checks during POST. I unfortunately
    couldn't find any good authoritative sources listing all of the things they
    do, but just ask anyone who owns or operates server hardware and they will
    complain about this to you (see ServerFault threads
    [here](https://serverfault.com/questions/889855/why-do-servers-boot-up-so-slow-in-general)
    and
    [here](https://serverfault.com/questions/454706/how-to-make-my-hp-server-boot-faster)).

What's the solution to this ridiculous problem? Obviously, something more
ridiculous.

## Enter `kexec`

`kexec` stands for Kernel Execute. This is a command that tells the current
kernel to replace itself with a new kernel.

`kexec` is used in two phases:

1.  Load the kernel, initrd, and kernel command line args into memory
2.  Execute it

Here's step 1:

```sh
kexec \
    --load=/path/to/vmlinuz/or/whatever \
    --initrd=/path/to/initrd/or/whatever \
    --command-line="arg=foo bar=spam"
```

And here's step 2:[^2]

[^2]:
    If you actually use `kexec` in production you should run `systemctl kexec`
    or equivalent for your init system to properly shut down your services
    first. Just running it raw is the equivalent of resetting your system
    without hitting ACPI reset.

```sh
kexec --exec
```

What does it look like when you use `kexec` to accelerate your reboots? It looks
something like this:

1.  All other processes are shut down by the init process and all writes are
    flushed to disk
2.  The init process calls `kexec` to request the kernel to load and execute the
    new kernel
3.  Kernel and initrd will then meow meow meow meow meow meow go read
    [the last part](../../1/swap-out-the-root-before-boot/#header-putting-it-all-together)
4.  Successfully rebooted, yay!

By using `kexec`, you are able to shave an entire hour off of your downtime
every time you apply kernel updates! It's trusted by Google and a whole bunch of
other big companies to dodge long POSTs!

### What is it _really_ doing?

No, really. Think about what's actually happening for a second. Isn't a little
_too_ convenient that you can skip all those steps and everything works?

When you ran `kexec --load`, you packaged a couple of things from the current
system into RAM, in the form of an initramfs.

And then when you ran `kexec --exec`, you didn't send any ACPI signals.

You didn't go back into the bootloader, you didn't even go back into the
_firmware_, you didn't have the firmware POST...

... and you didn't have the firmware clear RAM.

Instead, you replaced your current kernel with a new kernel, and told it "please
execute these things that I left in my RAM here."

You're using RAM to transmit data between boots, without ever letting the
firmware know that you rebooted.

"Reboot" is a social construct that the firmware never needed to be involved in.
You never left “OS mode” on your computer.

`kexec`ing an initramfs is a way that you can pass `curl` to the next system
without the firmware clearing it.

## Introducing: curlingiron

If you'll recall from the last part, initramfs is literally just a bunch of
binaries, libraries, and an init script shoved inside a cpio archive. You might
be wondering, “how hard could it be to make my own?”

That's what I wondered, and as it turns out, it's really not that hard.

You download both
[https://astrid.tech/curlingiron.initrd](https://astrid.tech/curlingiron.initrd)
and
[https://astrid.tech/curlingiron.vmlinuz](https://astrid.tech/curlingiron.vmlinuz).

Then, you run the following command:

```sh
kexec \
	--load /path/to/curlingiron.vmlinuz \
	--initrd /path/to/curlingiron.initrd \
	--command-line "console=ttyS0 curlingiron.src=https://something.example/foobar.img curlingiron.dst=/dev/sda"
kexec --exec
```

and this will send the data inside the initramfs into the next boot, depositing
12 bitcoins into your wallet.

Look, here's me using it to make a Raspberry Pi flash itself!

// TODO record a video[^3]

[^3]:
    Imagining what this would look like is an exercise left for the reader. Also
    it wouldn't work anyway because I compiled these for x86.

### What's actually inside?

It's an extremely straightforward program! curlingiron is just a wrapper around
`wget | dd`:

```sh
# Read src and dst from kernel params
kernel_params="$(cat /proc/cmdline)"
src="$(echo "$kernel_params" | sed -n 's/.*curlingiron.src=\([^ ]*\).*/\1/p')"
dst="$(echo "$kernel_params" | sed -n 's/.*curlingiron.dst=\([^ ]*\).*/\1/p')"

echo "Writing $src to $dst"

# Actually download it
wget -O- "$src" | dd bs=1M of="$dst"

echo "Done, rebooting!"
sync
reboot -f
```

(The `wget` implementation doesn't actually check SSL certs. Whatever, who
cares, proof of concept, we'll get to that later!)

You pass in the following kernel command line args:

- `curlingiron.src=` is an HTTP URL (i.e.
  `https://somewhere.example/foobar.img`)
- `curlingiron.dst=` is the destination to write to (i.e. `/dev/sda`)

and it does the writing, then it syncs changes to disk, and reboots, and boom,
new system!

Simple, right?

### And now for the boilerplate

Okay, so that was the meat of of my script. Here's the entire thing. Turns out
getting this to work was _just_ a bit more complicated than that.

```sh
#!/bin/sh

# Mount the very important directories
mkdir -p /dev /proc /sys
mount -t devtmpfs devtmpfs /dev
mount -t proc procfs /proc
mount -t sysfs sysfs /sys

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

# SCSI disk layer (doesn't come from modalias)
modprobe sd_mod 2>/dev/null
modprobe sr_mod 2>/dev/null

# The kernel packaged with Nix apparently doesn't do this???
modprobe af_packet 2>/dev/null

# Turn on network interface and DHCP clients
ip link set eth0 up
udhcpc --background --script /etc/dhcpevent.sh &
udhcpc6 --background --script /etc/dhcpevent.sh &

# Read src and dst from kernel params
kernel_params="$(cat /proc/cmdline)"
src="$(echo "$kernel_params" | sed -n 's/.*curlingiron.src=\([^ ]*\).*/\1/p')"
dst="$(echo "$kernel_params" | sed -n 's/.*curlingiron.dst=\([^ ]*\).*/\1/p')"

echo "Writing $src to $dst"

# Wait until we have any amount of connectivity. This is because
# DHCP might not be up by the time we get here, so we have to wait.
until wget --spider "$src"; do
  echo "Connection failed, waiting until we have connectivity"
  sleep 5
done

# Actually download it
wget -O- "$src" | dd bs=1M of="$dst"

echo "Done, rebooting!"
sync
reboot -f
```

### And now for the gore

Okay, that's not the entire thing either.

If that's a shell script, it has to run a bunch of binaries and load a bunch of
kernel modules. Where does it load _those_ from?

Well, it's just an initramfs, which is just a cpio, so why don't we unpack it
and see what's inside?

```
astrid@🌐 chungus /tmp/curlingiron ❯ unzstd < ../../curlingiron.initrd | cpio -i
```

<div style="height: 200px"></div>

**CW: nix jumpscare**

<div style="height: 200px"></div>

```
astrid@🌐 chungus /tmp/curlingiron ❯ tree -L 3 -F
./
├── bin -> /nix/store/2qzl4hvvsgl0l39wi34w66j0kwmrwprc-installed/bin/
├── etc/
│   └── dhcpevent.sh -> /nix/store/l9np9nak71bmzx8s04npk58sy2x6hr14-dhcpevent.sh*
├── init -> /nix/store/0yfym6sgyyw02ycmx0xyrlmm189krn1j-init*
├── lib -> /nix/store/5i01v0wrpwxzy1kck92p72g1q16ydpda-linux-6.18.15-shrunk/lib/
├── nix/
│   └── store/
│       ├── 0yfym6sgyyw02ycmx0xyrlmm189krn1j-init*
│       ├── 2qzl4hvvsgl0l39wi34w66j0kwmrwprc-installed/
│       ├── 4dd2451gimsjq0v2i1s8zrh5w2b72qrw-busybox-1.37.0/
│       ├── 5i01v0wrpwxzy1kck92p72g1q16ydpda-linux-6.18.15-shrunk/
│       ├── l0l2ll1lmylczj1ihqn351af2kyp5x19-glibc-2.42-51/
│       └── l9np9nak71bmzx8s04npk58sy2x6hr14-dhcpevent.sh*
├── run/
├── sbin -> /nix/store/2qzl4hvvsgl0l39wi34w66j0kwmrwprc-installed/sbin/
├── tmp/
└── var/
    ├── empty/
    └── run -> ../../run/

16 directories, 4 files
```

Yes, that's a Nix store.

I can explain!

_[Continued in part 3.](../../3/chicken-shed)_
