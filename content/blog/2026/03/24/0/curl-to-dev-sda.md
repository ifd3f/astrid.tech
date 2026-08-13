---
title: curl > /dev/sda
tagline: How I made a Linux distro that runs `wget | dd`
tags:
  - linux
  - http
slug:
  ordinal: 0
  name: curl-to-dev-sda
date:
  created: 2026-03-16 18:11:00-07:00
  published: 2026-03-24 02:10:00-07:00
  updated: 2026-03-24 07:03:00-07:00
---

_This is part 0 of a four-part series about doing weird things in early Linux
boot._

To replace your Linux installation with a new OS, you can download a
pre-prepared disk image directly to your disk by running a command like this:

```sh
curl https://something.example/foo.img > /dev/sda # (or whatever your disk is called)
```

No need to save it anywhere first, just pipe it directly to the disk. Then you
can reboot into that OS, and congratulations, you've installed a new OS. This
works even for most EFI machines without any further `efibootmgr` commands
because
[EFI firmware automatically discovers the new EFI system partition!](https://superuser.com/questions/731362/how-does-efi-find-bootloaders)

## Why?

This command is possible because `/dev/sdX`, `/dev/nvmeX`, and other such disks
can be directly accessed from the filesystem, following the ancient Unix concept
of “everything is a file.” You can write to the `/dev/sdX` file, and that
directly writes to the disk!

## No, like, why would you want to do this instead of installing Linux any other normal way?

Okay, so the reason I initially did this was because I didn't want to pay
Contabo an extra $1.50/mo to have object storage just to be able to spawn VPSes
from premade disk images.

I thought it was a neat trick, a funny shitpost that riffs on the eternal
`curl | sh` debate. I could write a blog post about it, I tell you about how you
can do it yourself, one thousand words, I learn something, you learn something,
I get internet points, win win.

The problem is that it didn't stop there. I kept asking one more question. I
kept peeling one more layer off the onion. I kept turning one more page in the
grimoire… and before I knew it, I ended up with a four part blog series that
doesn't end where you expect it to end.

Why don't we start from the beginning?

## How to flash a Raspberry Pi the cool way

Nowadays, the Raspberry Pi Foundation gives you a piece of software that they
built that does everything automatically, but back in my day, you had to do it
this way.

There's a [Stack Exchange answer](https://raspberrypi.stackexchange.com/a/932)
that lists the exact series of steps.

1.  First, you go to the website in your web browser.
2.  Then, you click on the button to download the image.
3.  Then, quoting directly from that answer:

> Copy the contents of the image file onto the SD card by running
>
> `sudo dd bs=1M if=your_image_file_name.img of=/dev/sdx`

Yeah, they also said to `sha256sum` the image, but let's be honest, nobody
fucking does that, not even [Caligula](https://github.com/ifd3f/caligula/)
users.

### lazier?

Now, a logical first improvement to this step is, rather than navigating through
the web browser, you can just `wget` or `curl` that file from the command line
to begin with if you already know the URL. So, you can run:

```sh
wget -O disk.img https://www.raspberrypi.com/whatever
sudo dd bs=1M if=disk.img of=/dev/sdx
```

### lazier.

But of course, nothing says you have to download it to a file first. `dd` will
read from stdin if you don't give it an `if=` argument. So you can pipe into
`dd` and that achieves the same effect without writing any intermediate files.

```sh
curl https://www.raspberrypi.com/whatever | sudo dd bs=1M of=/dev/sdx
```

### lazier!

But of course, nothing says you have to use `dd`. That just makes your writes to
the disk more efficient because of block alignment and caching nonsense. You can
just redirect stdout like so:

```sh
sudo -i
curl https://www.raspberrypi.com/whatever > /dev/sdx
```

And congratulations, you have derived the initial shitpost premise from first
principles.

### it was compressed oops

I glossed over the fact that the Stack Exchange article also tells you that the
disk image comes as a zip file that you need to unzip first. But that's a nice
segue, because it turns out there are plenty of other variations on this:

```sh
# You may need to unzip your thing
curl https://something.example/foo.img.gz | gunzip | dd bs=1M of=/dev/sda status=progress
# You can use wget
wget -O- https://something.example/foo.iso | dd bs=1M of=/dev/sda status=progress
# You can upload via SSH
gzip -vc disk.img | ssh my-server.example -- sh -c 'gunzip -vc | dd bs=1M of=/dev/sda status=progress'
```

I mean frankly, there's an infinite number of ways to write directly from the
network to the disk![^1]

[^1]:
    Accomplishing this using other common file transfer protocols like gopher,
    BitTorrent, SMTP, magic-wormhole, NFS, RFC 1149, or FCC Part 97 are left as
    an exercise for the reader.

### sillier?

Okay, so now, let's say you have a VM running in Contabo, booted off of
`/dev/sda`, that you wanted to reimage with your own OS image. What do we need
to do to adapt this method to that?

## Making your own bootable OS image

Of course, you do need to figure out how to make such an OS image first.
Luckily, you can do this for any OS (even Windows!) by installing it in a VM
first, and then using the raw disk image that results from that.

To do that with QEMU, you need to first make a raw disk, preferably of a fairly
small size (you should be able to expand it once you've copied it).

```sh
truncate 10G disk.img
```

Then, you can run your OS with installer like so:

```sh
qemu-system-x86_64
	-hda ./disk.qcow2 \
	-m 2G \
	-enable-kvm \
	-nic user \
	-serial mon:stdio
```

Go through all the setup steps, and you're done. You can now send `disk.img` off
to your webserver.

Of course, I use NixOS btw so this entire process was automated! I just did
[`import <nixpkgs/nixos/lib/make-disk-image.nix>`](https://github.com/NixOS/nixpkgs/blob/9c2c1b470bcf6d22d2e8f1d8f922f122bd3b16b0/nixos/lib/make-disk-image.nix)
and that got me a disk image. If you use Nix as well, you can learn how to use
it by reading its documented source code.

## Unmounting the disk

Now that we've made the disk image, we need to unmount the victim disk. This is
a very easy process. You just type in `umount /dev/sdwhatever` or
`umount /dev/nvmewhatever`, like

```
root@localhost:~# umount /dev/sda1
umount: /: target is busy.
```

Oh. Right. The disk we're trying to replace is the OS's main disk. The one the
OS is running off of.

Well, what can we try instead?

### write to the mounted disk anyways. fuck you

The OS may stop you from unmounting `/dev/sda1`, but it won't stop you from
writing to `/dev/sda1` or `/dev/sda` even if there's something mounted! How do
you think `parted`, `gparted`, and `fdisk` work on live systems?

I ran the following command to upload, and typed in my password:

```
astrid@chungus infra ❯ gzip -vc result/nixos.img | ssh root@myhost.example -- bash -c 'gunzip -vc > /dev/sda'
root@myhost.example's password:
```

Now, I tried this out expecting that it probably wouldn't work. Sure, programs
have to get loaded into memory to run, so maybe this might work. But also, given
that a whole bunch of other things are happening on the machine at the same time
besides the rewrite, doing this may cause those other processes to trigger a
kernel panic.

But theory means nothing in the face of practice.

The command had no output, but `iftop` was indicating that _something_ was
happening:

```
                        191Mb                   381Mb                   572Mb                   763Mb              954Mb
└───────────────────────┴───────────────────────┴───────────────────────┴───────────────────────┴───────────────────────
chungus.lan                                    => myhost.example.                                32.9Mb  22.3Mb  16.6Mb
                                               <=                                                 424Kb   302Kb   228Kb
```

After waiting for a little while, the program terminated with the following
output:

```
astrid@chungus infra ❯ gzip -vc result/nixos.img | ssh root@myhost.example -- bash -c 'gunzip -vc > /dev/sda'
root@myhost.example's password:
 77.8% -- replaced with stdout
```

What happened here?

Well, we tried to overwrite the OS while it was in use, and that caused it to
crash 77.8% of the way through! A whole bunch of things _could_ have happened,
and the exact crash detail _could_ be interesting, but the sum of it is that we
did something stupid and caused something stupid.

Maybe it's a good idea to unmount the device before writing to it after all.

Still, though, we have to think about the implications here. How do you unmount
your OS's disk while keeping the OS running to be able to overwrite itself?

This may sound like some kind of paradoxical Buddhist meditation riddle, but the
solution is actually quite simple: just boot into a new OS where the old one
isn't mounted!

### Rescue images to the rescue

Most Linux distros' installers have all the requisite programs preinstalled,
along with networking. Arch and NixOS installers are perfect for this purpose!
Fedora installer can even be used for this too. You can piggyback off of them
without even running the actual installer included with them! As long as you
have networking and terminal access, you can run `curl > /dev/sda` to your
heart's content.

This was, in fact, the option I went with to overwrite my Contabo VMs. Contabo
lets you boot into a Debian-based rescue image instead of whatever's installed
on the disk they give you, so I booted into that, and managed to
`wget -O- > /dev/sda`[^2] and get my new OS installed!

[^2]:
    `-` can be passed into various Linux command flags as a means to say “please
    read from stdin” or “please write to stdout.” Now of course, this begs the
    question: how do you tell utilities like this “please literally refer to the
    file named `-` located in my current working directory?” The answer is quite
    silly, you just pass in `./-`.

I did this once, and I was happy.

Then I did it a second time, and thought, “isn't there a way to automate this?”

### Also, do we _really_ need a second disk?

No really, just think about it. All of the utilities needed to overwrite the
disk already exist on the disk before you overwrite it. In theory, you could
just copy all of those utilities into RAM, unmount everything else, and
overwrite the disk from RAM to achieve an in-place swap.

Disks and boot can't possibly be _that_ magical… can they?

_[Continued in part 1.](../../1/swap-out-the-root-before-boot)_
