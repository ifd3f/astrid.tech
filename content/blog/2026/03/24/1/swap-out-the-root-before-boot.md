---
title: Swap out the root before boot
tagline: Learn about the Linux boot process by making it boot something else
tags:
  - linux
  - boot
  - ctf
  - debian
slug:
  ordinal: 1
  name: swap-out-the-root-before-boot
date:
  created: 2026-03-16 18:11:00-07:00
  published: 2026-03-24 02:10:01-07:00
  updated: 2026-03-24 07:03:00-07:00
---

_This is part 1 of a four-part article series about how to reimage disks
in-place. [Part 0 is located here](../../0/curl-to-dev-sda)._

Did you know that when Linux boots, it doesn't actually mount your root disk at
first? It actually runs a mini-me OS that finds the real root disk and becomes
it.

The official name for this mini-me is the **initramfs** (**Init**ial **RAM**
**F**ile **S**ystem), which some people still call the **initrd** (**Init**ial
**R**AM **D**isk). I'll be using both terms interchangeably to mean the same
thing. As the name suggests, it's a filesystem that lives purely in RAM.

If this all seems very abstract, that's because it is. Let's look inside a
Debian VM and see how it fits together in practice!

## Crack open a cold one

For this article, I'm using QEMU to boot into
[a premade Debian 13 nocloud VM image](https://cloud.debian.org/images/cloud/trixie/latest/).
You can follow along at home by running the VM like so:

```sh
# If you don't use x86, you can probably figure out how to adapt this.
qemu-system-x86_64 \
	-hda ./disk.qcow2 \
	-m 2G \
	-enable-kvm \
	-nic user \
	-serial mon:stdio
```

If you're running a Linux system, though, you can actually inspect your system's
real initrd!

Usually, the kernel image, bootloader, and initrd are all located in the `/boot`
folder on your machine, on their own FAT partition. Here's how it looks inside a
Debian VM:

```
root@localhost:~# ls -l /boot
total 47040
-rw-r--r-- 1 root root       83 Feb 17 05:47 System.map-6.12.73+deb13-amd64
-rw-r--r-- 1 root root   283277 Feb 17 05:47 config-6.12.73+deb13-amd64
drwx------ 3 root root    16384 Jan  1  1970 efi
drwxr-xr-x 6 root root     4096 Feb 20 05:37 grub
-rw-r--r-- 1 root root 35742063 Feb 20 05:37 initrd.img-6.12.73+deb13-amd64
-rw-r--r-- 1 root root 12109760 Feb 17 05:47 vmlinuz-6.12.73+deb13-amd64
```

`grub` is GRUB, the bootloader; `vmlinuz` is your Linux kernel image; and
`initrd` is your initramfs, because we still call it that despite having moved
on decades ago.

Now, how do we open it up? The initramfs, it turns out, is just a cpio archive,
which the kernel will unpack into a RAM filesystem during boot. (And if you're
not familiar with cpio archives, they're a bit like tarballs in that they
contain a big bundle of files stored at various paths.)

You can actually inspect what's on the initrd by opening it up![^1]

[^1]:
    I lied very slightly when I said that it's “just” a cpio. The reason the
    extraction is weird like this is because of a thing called _microcode
    prepending_. Google it if you're interested.

```
root@localhost:~# mkdir initrd
root@localhost:~# cd initrd
root@localhost:~/initrd# cpio -idm < /boot/initrd.img-6.12.73+deb13-amd64
50564 blocks
root@localhost:~/initrd# dd skip=50564 if=/boot/initrd.img-6.12.73+deb13-amd64 | unzstd | cpio -idm
19244+1 records in
19244+1 records out
9853295 bytes (9.9 MB, 9.4 MiB) copied, 0.1218 s, 80.9 MB/s
68807 blocks
```

```
root@localhost:~/initrd# ls -lF
total 28
lrwxrwxrwx 1 root root    7 Feb 20 05:37 bin -> usr/bin/
drwxr-xr-x 3 root root 4096 Mar 17 15:50 conf/
drwxr-xr-x 5 root root 4096 Mar 17 15:50 etc/
-rwxr-xr-x 1 root root 6787 May 13  2025 init*
lrwxrwxrwx 1 root root    7 Feb 20 05:37 lib -> usr/lib/
lrwxrwxrwx 1 root root    9 Feb 20 05:37 lib64 -> usr/lib64/
drwxr-xr-x 2 root root 4096 Feb 20 05:37 run/
lrwxrwxrwx 1 root root    8 Feb 20 05:37 sbin -> usr/sbin/
drwxr-xr-x 5 root root 4096 Mar 17 15:50 scripts/
drwxr-xr-x 6 root root 4096 Mar 17 15:50 usr/
```

You might think this looks a bit like a normal Linux root filesystem. In fact,
it is! But there is one special file that makes everything work: `/init`.

This /init script is executed by the kernel after all the environmental stuff is
set up and the initramfs is unpacked into a filesystem. It is the first, and
only process that the kernel will ever execute on its own. All other processes
will be spawned by /init.

If we peek at that `/init` file, you'll notice that it's a shell script.

```
root@localhost:~/initrd# head init
#!/bin/sh

# Default PATH differs between shells, and is not automatically exported
# by klibc dash.  Make it consistent.
export PATH=/sbin:/usr/sbin:/bin:/usr/bin

[ -d /dev ] || mkdir -m 0755 /dev
[ -d /root ] || mkdir -m 0700 /root
[ -d /sys ] || mkdir /sys
[ -d /proc ] || mkdir /proc
```

There's a whole bunch of logic in this script that you don't have to worry
about, but at the bottom, it eventually `exec`s the init process that actually
lives on the actual disk.

```
root@localhost:~/initrd# grep -n exec init
...
340:exec run-init ${drop_caps} "${rootmnt}" "${init}" "$@" <"${rootmnt}/dev/console" >"${rootmnt}/dev/console" 2>&1
```

## Putting it all together

With all of this together, the process from bootloader to init process looks
something like this:

1.  Bootloader loads kernel and initramfs into memory
2.  Bootloader executes the kernel, passing the initramfs in as an argument
3.  Kernel sets up environmental things like interrupts and memory management
4.  Kernel unpacks the initramfs into a tmpfs, then executes the init process at
    /init with PID 1
5.  The init process loads drivers needed to mount filesystems
6.  The init process mounts an actual disk, and changes its root directory into
    that disk[^2]
7.  The init process uses `exec` to become the actual, full init process located
    on the disk

[^2]:
    Technically, this process uses `switch_root`, which is like if `chroot` also
    deleted the initramfs.

If you think this is stupid and convoluted, wait until you hear what they used
to do.[^3]

[^3]:
    Rather than having an initrd tell it to mount stuff,
    [the kernel actually used to mount the root FS itself by having hardcoded major and minor device numbers](https://unix.stackexchange.com/a/18055)!
    There was a program, `rdev`, that let you change those hardcoded values
    without recompiling!

Armed with this knowledge, let's stop Debian's bog standard default boot process
at **step 4** to swap out the disk under the OS's feet.

## Let's _not_ mount the root filesystem, actually

Still inside that Debian 13 VM, I rebooted, and went into GRUB.

```
                            GNU GRUB  version 2.12-9

 +----------------------------------------------------------------------------+
 |d-2e12f75e0e9d                                                              |^
 |                else                                                        |
 |                  search --no-floppy --fs-uuid --set=root 92cad97f-ed1a-4b2\|
 |d-86ad-2e12f75e0e9d                                                         |
 |                fi                                                          |
 |                echo        'Loading Linux 6.12.73+deb13-amd64 ...'         |
 |                linux        /boot/vmlinuz-6.12.73+deb13-amd64 root=PARTUUI\|
 |D=ee7feeef-aee7-4010-9c05-1cdbb4f8cc1b ro single dis_ucode_ldr console=tty0\|
 | console=ttyS0,115200 earlyprintk=ttyS0,115200 consoleblank=0               |
 |                echo        'Loading initial ramdisk ...'                   |
 |                initrd        /boot/initrd.img-6.12.73+deb13-amd64          |
 |                                                                            |
 +----------------------------------------------------------------------------+

      Minimum Emacs-like screen editing is supported. TAB lists
      completions. Press Ctrl-x or F10 to boot, Ctrl-c or F2 for
      a command-line or ESC to discard edits and return to the GRUB menu.
```

My hunch was `root=` seems to tell the initramfs “find the partition that looks
like this and mount that as your eventual root.” So if you delete it, it won't
have a root to mount, so it will just drop you into a shell.

I did that, then I hit Ctrl-x to boot, and it successfully booted into this
shell in initramfs where nothing is mounted.

```
[    1.588813] sr 1:0:0:0: [sr0] scsi3-mmc drive: 4x/4x cd/rw xa/form2 tray
...
Begin: Loading essential drivers ... done.
Begin: Running /scripts/init-premount ... done.
Begin: Mounting root file system ... Begin: Running /scripts/local-top ... done.
Begin: Running /scripts/local-premount ... done.
No root device specified. Boot arguments must include a root= parameter.
(initramfs)
```

But unfortunately, this initramfs doesn't have any of the required utilities. It
lacks `curl` and `wget` and almost all other networking things… though it does
seem to have `ipconfig`.

Yes, `ipconfig`, not `ifconfig`. Google was confused by that too.

It clearly does not work the same way as the Windows one:

```
(initramfs) ipconfig
ipconfig: no devices to configure
```

`ip=dhcp` supposedly makes it do DHCP-ing. I tried it… and it seems to have
worked?[^4]

[^4]:
    I'll be completely honest, I don't know what this `ip=dhcp` construct is. It
    comes from an AI summary, which in turn cites
    [this Medium article](https://medium.com/@hasancansert/from-local-disks-to-network-roots-mastering-nfs-booting-for-modern-linux-systems-3cf544c934fa#:~:text=Configuring%20the%20Kernel%20Boot%20Parameters%20root=/dev/nfs%20%E2%80%94,board%20to%20obtain%20an%20IP%20address%20automatically.)
    and
    [this Arch wiki article](https://wiki.archlinux.org/title/Diskless_system#NFS_2),
    neither of which actually explain why that works. I only figured out that
    `ipconfig` comes from the `mkinitcpio-nfs-utils` package from searching for
    it on
    [search.nixos.org](https://search.nixos.org/packages?channel=25.11&query=ipconfig&show=mkinitcpio-nfs-utils).
    There is no website for that package, and there are no manpages. The only
    actual official documentation I've found has been the READMEs in the literal
    sourcecode itself, which is available from
    [sources.archlinux.org](https://sources.archlinux.org/other/mkinitcpio/).
    Both READMEs appear to have been hastily compiled in the 2010s, and neither
    even mention this `ip=dhcp` construct as a possibility. By the time you read
    this, this post may end up becoming the biggest hit for `ip=dhcp`!

```
(initramfs) ipconfig "ip=dhcp"
IP-Config: ens3 hardware address 52:54:00:12:34:56 mtu 1500 DHCP RARP
[ 1295.952616] e1000: ens3 NIC Link is Up 1000 Mbps Full Duplex, Flow Control: RX
IP-Config: ens3 guessed broadcast address 10.0.2.255
IP-Config: ens3 complete (dhcp from 10.0.2.2):
 address: 10.0.2.15        broadcast: 10.0.2.255       netmask: 255.255.255.0
 gateway: 10.0.2.2         dns0     : 10.0.2.3         dns1   : 0.0.0.0
 rootserver: 10.0.2.2 rootpath:
 filename  :
```

The `nfsmount` command exists in this initramfs as well. So I guess nothing
stops you from acquiring your image from NFS! That, however, is left as an
exercise for the reader.

## The self-induced CTF

I was originally going to just end this investigation thread here, and let some
other nerd reading my article figure out the NFS thing.

But then I was watching TV, specifically
[Romance of the Three Kingdoms (1994)](<https://en.wikipedia.org/wiki/Romance_of_the_Three_Kingdoms_(TV_series)>),[^5] which
adapts the Chinese epic novel to the screen. I was thinking of an episode I had
seen earlier, in which
[Zhuge Liang “borrows” a hundred thousand arrows from his enemy Cao Cao](https://en.wikipedia.org/wiki/List_of_fictitious_stories_in_Romance_of_the_Three_Kingdoms#Borrowing_arrows_with_straw_boats)
by baiting Cao Cao's men into firing arrows at his thatched boats. He then went
home and pulled them all out, and a few days later, his armies would use those
arrows on Cao Cao to great success.

[^5]:
    Despite its age (and length), it's a really good show. It has epic subject
    matter and insanely deep scheming while also having very cheesy fight
    choreography and acting. There's a good English fansub you can get here:
    [https://gentlemenofthehan.wordpress.com/](https://gentlemenofthehan.wordpress.com/)

![A painting of the Zhuge Liang arrow-borrowing scene from the Summer Palace in Beijing.](https://s3.us-west-000.backblazeb2.com/nyaabucket/ddb6011c861ee7801b3d79961ee153a6c015992e159caaa5d84db8595c053fcf/straw-boats.jpg)

That was when it hit me – the disk I'm overwriting _does_ in fact have `curl` on
it. If there's a `curl` on the disk, why can't I “borrow” it into the initramfs
as well?

### Copying `curl` into RAM

I first mounted the victim disk.

```
(initramfs) mkdir /mnt
(initramfs) mount -t ext4 /dev/sda1 /mnt
[   91.218815] EXT4-fs (sda1): mounted filesystem 92cad97f-ed1a-4b2d-86ad-2e12f75e0e9d r/w with ordered data mode. Quota mode: none.
```

Of course, because `curl` expects its libraries to be located in /lib, you can't
execute it directly.

```
(initramfs) mnt/bin/curl
mnt/bin/curl: error while loading shared libraries: libcurl.so.4: cannot open shared object file: No such file or directory
```

That's fine. Let's try copying `curl` and its libraries into initramfs.

```
(initramfs) cp mnt/bin/curl bin/curl
sh: 18: cp: not found
```

Oh. I guess we don't even have that. Let's try a different thing.

```
(initramfs) cat mnt/bin/curl > /bin/curl
(initramfs) /bin/curl
sh: 20: /bin/curl: Permission denied
(initramfs) chmod +x /bin/curl
sh: 21: chmod: not found
```

Okay, well this fucking sucks. The initramfs is so minimal we don't even have
`cp` or `chmod`! How do you borrow `curl` without those?

### Desperate times from stupid games

I tried a number of different things after this.

```
(initramfs) ln -s /bin /mnt/test # can we cp stuff via symlink?
(initramfs) chroot mnt bash
bash: cannot set terminal process group (-1): Inappropriate ioctl for device
bash: no job control in this shell
root@(none):/# ls test
'['				      nsenter
 aa-enabled			      nstat
 aa-exec			      numfmt
 aa-features-abi		      od
... # nope -- it was disk's /bin, not initramfs's /bin!
```

```
(initramfs) cpio
Usage: cpio [-V] -i [< archive]
(initramfs) chroot mnt bash
root@(none):/# # time for an absolutely gnarly one-liner that cpios curl and all of its deps
root@(none):/# (echo /usr/bin/curl; ldd /usr/bin/curl | awk '{print $3}' | awk 'NF' | while read -r l; do echo $l; readlink -f $l; done) | cpio -o > curl.cpio
63251 blocks
root@(none):/# exit
exit
(initramfs) cpio -i < /mnt/curl.cpio
cpio: premature end of file
```

Nothing worked![^6]

[^6]:
    It was only later, after writing all this up and recreating the process,
    that I realized I needed to append `-H newc` to the disk's `cpio` command,
    because the initramfs `cpio` only supports SVR4-format archives.

I tried brainstorming other things, but I couldn't think past a certain wall:
how do I transmit things past that annoying `chroot` boundary? Now, in
retrospect, there were so many options available to me that I hadn't thought of
at the time. To name a few:

- I could have actually run `mnt/bin/curl` and overriden the `LD_LIBRARY_PATH`
  environment variable.
- I could have either bind-mounted or remounted the initramfs to be accessible
  inside /mnt.
- I could have just tried mounting `/dev` inside `/mnt` anyways.

Instead, a stupid idea struck me, and I felt stupider after realizing it.

`chroot`'s usage looks like this:

```
(initramfs) chroot
Usage: chroot newroot command...
```

It's just a normal command. And the command argument is also just a normal
command. I've been putting shells in the command position this whole time.
Nothing about it requires that you _only_ put shells there.

### Desperate measures for stupid prizes

I first mounted the victim disk as read-only (so that ext4 doesn't accidentally
commit any changes after we finish overwriting it).

```
(initramfs) mkdir /mnt
(initramfs) mount -t ext4 -o ro /dev/sda1 /mnt
[   91.218815] EXT4-fs (sda1): mounted filesystem 92cad97f-ed1a-4b2d-86ad-2e12f75e0e9d r/w with ordered data mode. Quota mode: none.
```

Then, I ran this command.

```
(initramfs) chroot /mnt curl -v http://10.0.2.2:8000/result/idiot.img > /dev/sda
```

`curl` is being run in `chroot`, but the `> /dev/sda` is not, allowing us to
traverse the `chroot` boundary.

And it… all… works.[^7]

[^7]:
    Full disclosure: I was unable to get DNS working here. Got
    `curl: (6) Could not resolve host: google.com`. Still, the fact that
    everything else worked is impressive.

```
(initramfs) chroot /mnt curl -v http://10.0.2.2:8000/result/idiot.img > /dev/sda
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
  0     0    0     0    0     0      0      0 --:--:-- --:--:-- --:--:--     0*   Trying 10.0.2.2:8000...
* Connected to 10.0.2.2 (10.0.2.2) port 8000
* using HTTP/1.x

...

<
{ [47520 bytes data]
* Request completely sent off
{ [102400 bytes data]
[  144.301186] random: crng init done
100 3275M  100 3275M    0     0  82.4M      0  0:00:39  0:00:39 --:--:--  179M
* shutting down connection #0
```

I rebooted, and ended up in a brand new system, and became the first and only
winner of the CTF known as Debian 13 default nocloud VM initramfs.

```
(initramfs) reboot
[  434.452133] sd 0:0:0:0: [sda] Synchronizing SCSI cache
[  434.538469] ACPI: PM: Preparing to enter system sleep state S5
[  434.539977] reboot: Restarting system
[  434.540715] reboot: machine restart

BdsDxe: loading Boot0002 "UEFI QEMU HARDDISK QM00001 " from PciRoot(0x0)/Pci(0x1,0x1)/Ata(Primary,Master,0x0)
BdsDxe: starting Boot0002 "UEFI QEMU HARDDISK QM00001 " from PciRoot(0x0)/Pci(0x1,0x1)/Ata(Primary,Master,0x0)

Booting initrd of NixOS 26.05 (Yarara) (Initrd).
[  OK  ] Created slice Slice /system/modprobe.
[  OK  ] Started Dispatch Password Requests to Console Directory Watch.
         Expecting device /dev/disk/by-uuid/67f14184-9305-4de6-accb-ad89928a5d99...
[  OK  ] Reached target Path Units.
[  OK  ] Reached target Slice Units.
[  OK  ] Reached target Swaps.
...
```

## Wait, you overwrote it while it was mounted? Why did it work _this_ time?

As mentioned in the last article, when you execute code, it has to get copied
into memory first. So even though we overwrote the `curl` on disk, the `curl` in
memory could still go on and destroy the original.

… Well, okay, except there's even a little wrinkle there.

Linux actually loads code into memory _lazily_ – only grab the chunks of code
(pages) from the disk when someone asks for them. The technical term for this is
[Demand Paging](https://en.wikipedia.org/wiki/Demand_paging). It's likely that,
by the time `curl` is in its “download bytes/write bytes” loop, all of the
requisite pages have been paged in, so we don't run into issues.

Another reason it may have worked this time is probably because nothing else was
using that disk. In the last article, when I tried it, the OS was probably
running a lot of services. This time, the only thing using it is `curl` before
it gets fully paged in.

## Conclusion

In conclusion, I have successfully demonstrated how easy it is to trick you into
learning how actual initramfses work in real life.

I didn't actually do this. I went straight to the stupider option.

When your computer reboots, the firmware clears the RAM so that the new OS can't
read secrets out of the old OS. I figured out a way to make an initramfs dodge
that.

_[Continued in part 2.](../../2/how-to-pass-secrets-between-reboots)_
