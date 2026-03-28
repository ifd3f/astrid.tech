---
title: Linux is an interpreter
tagline: And why you would want to exec a cpio
tags:
  - linux
  - kexec
  - boot
  - recursion
  - quine
  - elf
slug:
  ordinal: 0
  name: linux-is-an-interpreter
date:
  created: 2026-03-27 12:30:00-07:00
  published: 2026-03-28 06:33:00-07:00
---

**This is a standalone addendum to an earlier four-part series.** Reading the
previous parts is not required.

Links to previous parts, if you are interested:

- [Part 0: curl > /dev/sda](https://astrid.tech/2026/03/24/0/curl-to-dev-sda/)
- [Part 1: Swap out the root before boot](https://astrid.tech/2026/03/24/1/swap-out-the-root-before-boot/)
- [Part 2: How to pass secrets between reboots](https://astrid.tech/2026/03/24/2/how-to-pass-secrets-between-reboots/)
- [The 3rd and final part: The little chicken shed that could](https://astrid.tech/2026/03/24/3/chicken-shed/)
- Part 5: you are here

---

[In a previous article](https://astrid.tech/2026/03/24/3/chicken-shed/#the-final-result),
I left you with this mysterious command.

```sh
curl https://astrid.tech/rkx.gz | gunzip | sudo sh
```

What does it do? This can't possibly be safe to run, can it? Am I distributing
malware to you?

Fine, fine, I'll open it up and show you what's inside.

## Reverse engineering `rkx.gz`

First, we download it.

```
astrid@chungus /tmp ❯ curl https://astrid.tech/rkx.gz | gunzip > rkx
  % Total    % Received % Xferd  Average Speed  Time    Time    Time   Current
                                 Dload  Upload  Total   Spent   Left   Speed
100 14.31M 100 14.31M   0      0 28.16M      0                              0
```

What kind of a file is it?

```
astrid@chungus /tmp ❯ file rkx
rkx: POSIX shell script, ASCII text executable
```

Well, I guess I tell you to pipe it to `sh`, so it's only expected it's a shell
script.

Let's see what it runs.

```
astrid@chungus /tmp ❯ cat rkx
...
Hr7vfOuMr610ygifa2yphI4pZCRAPHzf+dYZX1vplBE+19hSCR1TyECAePi+860zvrbSKSN8rrGl
EjqmkHEA8fB951tnfG2lU0b4XGNLJXRMIcMA4uH7zrfO+NpKp4zwucaWSuiYQkYBSimllFJKKaWU
vnXG11Y6ZYTPNbZUQscUMmYgHr7vfOuMr610ygifa2yphI4pZMhAPHzf+dYZX1vplBE+19hSCR1T
yIiBePi+860zvrbSKSN8rrGlEjqmkAED8fB951tnfG2lU0b4XGNLJXRMIeMFa6211lprrbXWWmut
KqFjChlIEA/fd751xtdWOmWEzzW2VELHFDKOIB6+73zrjK+tdMoIn2tsqYSOKWQYQTx83/nWGV9b
...
```

Oops, that's a lot of base64 junk! I guess that's only to be expected given that
it's a **20 megabyte shell script.**

```
astrid@chungus /tmp/r ❯ du -sh ../rkx.gz --apparent-size # i have a compressed disk so this flag tells you actual uncompressed size
20M	../rkx.gz
```

Well, if it's a shell script, it has to be legible. Let's just peek at its head
and tail.

```sh
astrid@chungus /tmp ❯ head rkx -n 15
astrid@chungus /tmp ❯ tail rkx
```

```sh
#!/bin/sh

set -x
if [ "$(id -u)" -ne 0 ]; then
  echo "Please ensure you are running as root/sudo"
  exit 1
fi

if ! command -v kexec && command -v base64 && command -v cpio 2>&1 >/dev/null ; then
  echo "Please ensure kexec-tools, base64, and cpio are installed"
  exit 1
fi

base64 -d <<912367yuiogrjklhsdijlslksdawuil234ui > r
MDcwNzAxMDAwQjI0MDkwMDAwNDE2RDAwMDBGRkZFMDAwMEZGRkUwMDAwMDAwMzAwMDAwMDAxMDAw
...
AAAAAAAAAAAAAA==
912367yuiogrjklhsdijlslksdawuil234ui

cpio -uidv < r "k" > k

kexec --load k --initrd r --reuse-cmdline
kexec --exec
```

Altogether, what does this do?

1. Ensure that the user is running as root
2. Ensure that the user has `kexec`, `base64`, and `cpio` installed
3. Turn 20MB of base64 into a cpio named "r"
4. Treat "r" as a cpio and pull out a file named "k"
5. Run `kexec` with "k" as a kernel and "r" as a ramdisk

So this piece of malware writes an OS to "k" and "r" and replaces the current OS
with that OS. Got it.

### carve out that base64 and decode his ass

```
astrid@chungus /tmp ❯ tail -n +15 < rkx | head -n -6 | base64 -d > r.cpio

astrid@chungus /tmp ❯ file r.cpio
r.cpio: ASCII cpio archive (SVR4 with no CRC)
```

Given that it treats this file as a ramdisk, it's no surprise that it's a valid
cpio.

What's it look like inside?

```
astrid@chungus /tmp/r ❯ cpio -i < ../r.cpio
30314 blocks

astrid@chungus /tmp/r ❯ tree --filelimit 10 -a
.
├── bin  [398 entries exceeds filelimit, not opening dir]
├── init
└── k

2 directories, 2 files
```

A /bin, and /init, and some file named `k`.

`k` is the kernel image it was extracting earlier:

```
astrid@chungus /tmp/r ❯ file k
k: Linux kernel x86 boot executable bzImage, version 6.18.18 (nixbld@localhost) #1-NixOS SMP PREEMPT_DYNAMIC Fri Mar 13 16:23:30 UTC 2026, RO-rootFS, swap_dev 0XC, Normal VGA
```

And `init` is a shell script, which is expectable:

```
astrid@chungus /tmp/r ❯ file init
init: POSIX shell script, ASCII text executable
```

### So, what's inside `/init`?

Well, first, it mounts /proc.

```sh
#!/bin/sh
mkdir -p /proc
mount -t proc proc /proc
```

Then, it makes a cpio at /r containing everything except /proc and /r.

```
find / | grep -v /r | grep -v /proc | cpio -vo -H newc > /r
```

And... well, it `kexec`s /k with /r.

```sh
kexec --load /k --initrd /r --reuse-cmdline
kexec --exec
```

Altogether now:

```sh
#!/bin/sh
mkdir -p /proc
mount -t proc proc /proc

find / | grep -v /r | grep -v /proc | cpio -vo -H newc > /r

kexec --load /k --initrd /r --reuse-cmdline
kexec --exec
```

And once you get there, the kernel be replaced with a new one, then run /init
inside /r, which happens to be this /init itself, which will replace the kernel
with a new one...

... so in other words, this is a Linux distro that recursively calls kexec on
itself! Isn't that so cute?

## So what really _is_ this thing?

In the previous article series,
[I made an initramfs that literally just runs `curl > /dev/sda` and reboots](https://astrid.tech/2026/03/24/0/curl-to-dev-sda/).
It's technically an OS, but it also might as well just be an executable file.

But think about how you run this executable file. You always have to pass it
into Linux, whether you're doing it from a bootloader, a VM, or `kexec`.

This feels strangely reminiscent of something that already exists. It feels like
the way you executed that initial payload with `curl | sh`, or `sh myscript.sh`,
or even `python3 myscript.py`.

In all of these cases, you're passing the program into another program that
actually interprets and runs the instructions inside.

Yes, that's right.

Initrds are programs, and Linux kernels are interpreter programs for initrds.

### Chasing your own tail

There's a weird thing happening in this malware, which is that using `kexec` for
recursion is a very strange form of recursion.

The CS 101 example of a recursive Fibonacci function is a very standard form of
recursion. You're taught not to do things like this because the professor will
put 1001 in and you'll hit Python's 1000-frame stack limit.

```py
def fib(n):
    match n:
        case 1: return 1
        case 2: return 1
        case n: return fib(n-1) + fib(n-2)
```

The program I gave you will never hit that stack limit.

There is no stack.

You're not nesting Linux kernels inside each other. You're replacing each Linux
kernel with a new one. But your new stack frame doesn't overwrite the old stack
frame whatsoever -- it builds up a new Linux interpreter stack frame in a
different part of memory, and executes that stack frame, leaving the old one
behind.

This initrd is a
[tail-call-optimized recursive function](https://en.wikipedia.org/wiki/Tail_call).
Stack frame replacement works by copying the program data to a new chunk of
memory, and executing that new chunk of memory. It's
[copy-on-write](https://en.wikipedia.org/wiki/Copy-on-write) by necessity
because the old program is actively executing while the new program is being
constructed.

> "Now, here, you see, it takes all the running you can do, to keep in the same
> place. If you want to get somewhere else, you must run at least twice as fast
> as that!" -- The Red Queen, from Alice in Wonderland

### Let me `fix` that for you

There's this concept of a
[Quine](<https://en.wikipedia.org/wiki/Quine_(computing)>), which is a
self-contained program that prints out a copy of itself.

As an example, here is this
[Python program (taken from the Wikipedia page)](<https://en.wikipedia.org/wiki/Quine_(computing)#Examples>):

```py
a: str = 'a: str = {}{}{}; print(a.format(chr(39), a, chr(39)))'; print(a.format(chr(39), a, chr(39)))
```

When piped into Python, it prints a copy of itself.

```sh
astrid@chungus ~  ❯ echo "a: str = 'a: str = {}{}{}; print(a.format(chr(39), a, chr(39)))'; print(a.format(chr(39), a, chr(39)))" | python3
a: str = 'a: str = {}{}{}; print(a.format(chr(39), a, chr(39)))'; print(a.format(chr(39), a, chr(39)))
```

Okay, so remember that init process from above?

If I had made it do something else at the end, maybe something like `cat /r`,
that would have made it spit out the cpio it's about to execute.

Which is the same exact cpio as itself.

**If the /init looked like this, I would have given you a quine of the Linux
initrd interpreter.**[^quine]

[^quine]:
    For the functional programmers among you, quines are often called the fixed
    points of their runtime environments. So this is a fixed point of the Linux
    interpreter as a program that executes initrds.

```sh
#!/bin/sh
mkdir -p /proc
mount -t proc proc /proc

find / | grep -v /r | grep -v /proc | cpio -vo -H newc > /r

cat /r
```

You may object that this program clearly performs I/O by reading files to
execute them.

No it doesn't.

These files are all in RAM.

Everything is a file, but these files are _variables_. No actual I/O is ever
being performed to read these files off a disk. When the script asks for them,
the kernel is just reading them off of a `tmpfs`!

This is a quine in the way as a C++ binary that scans through all its memory and
dumps out all of its in-memory program contents is.

**Exercise for the reader:** The malware I gave you had a 15M cpio. How small is
the smallest initrd that, when executed in a Linux kernel, outputs itself? In
other words, what is the smallest initrd quine?

## Interpret it a different way

If the Linux kernel is an interpreter, who interprets the Linux kernel
interpreter? What does it mean for a programming language to be interpreted?

Let's think about how conventional interpreters work first. In the example of
Python and Bash, you give them a string, they parse the string, and carry out
the instructions written in the string.

But those scripts aren't machine code! How is it possible for the Linux kernel
to execute my shell script when I run `./foo.sh`?

The shebang at the top (like `#!/bin/sh` or `#!/usr/bin/env python3`) basically
tells Linux "execute me by passing me to the thing after the `#!`.

Therefore, these two commands do exactly the same thing:

```sh
/bin/sh mything.sh
./mything.sh
```

Not all executable files have this header, though. For example, /bin/sh itself
actually has an ELF header, `\x7fELF`, indicating that it's a binary executable
file:

```
astrid@chungus rekexec ❯ cat /bin/sh | xxd | head -n 2
00000000: 7f45 4c46 0201 0100 0000 0000 0000 0000  .ELF............
00000010: 0300 3e00 0100 0000 c08f 0200 0000 0000  ..>.............
```

That makes a lot of sense. /bin/sh is a compiled binary. Even `file` on my NixOS
install confirms this.

```
astrid@chungus rekexec ❯ file -L /bin/sh
/bin/sh: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /nix/store/vr7ds8vwbl2fz7pr221d5y0f8n9a5wda-glibc-2.40-218/lib/ld-linux-x86-64.so.2, BuildID[sha1]=56923a72980631c2b23a5824f853b1c57b1f5f20, for GNU/Linux 3.10.0, not strid
```

Hang on.

> interpreter
> /nix/store/vr7ds8vwbl2fz7pr221d5y0f8n9a5wda-glibc-2.40-218/lib/ld-linux-x86-64.so.2

Interpreter? ELF files are interpreted too?

### Who interprets the interpreter?

[Well it turns out that from the kernel's perspective, the actual program being run is `ld-linux-x86-64.so.2`! The ELF doesn't do dynamic library management on its own, that `ld` program does!](https://stackoverflow.com/questions/71101779/what-is-the-role-of-program-interpreters-in-executable-files)

So the process is something like:

1. Execute ld.so
2. ld.so reads the /bin/sh ELF
3. ld.so finds the imported dynamic libraries of the ELF
4. ld.so loads them into memory
5. ld.so executes the program data from /bin/sh

You know what, I guess if you can imagine the existence of a Python or Bash
script that imports stuff and then runs raw machine code instructions, then ELFs
are kind of an interpreted language too!

And if you pass /bin/sh into that ld-linux-x86-64.so.2, does that work?

```sh
astrid@chungus rekexec ❯ /nix/store/vr7ds8vwbl2fz7pr221d5y0f8n9a5wda-glibc-2.40-218/lib/ld-linux-x86-64.so.2 /bin/sh
sh-5.3$
exit
```

Yes it does!

...

Hang on.

If `/bin/sh` interprets shell scripts, and `ld.so` interprets `/bin/sh`... who
interprets `ld.so`?

It can't possibly interpret itself, can it?

```sh
astrid@chungus astrid.tech-content ❯ file /nix/store/vr7ds8vwbl2fz7pr221d5y0f8n9a5wda-glibc-2.40-218/lib/ld-linux-x86-64.so.2
/nix/store/vr7ds8vwbl2fz7pr221d5y0f8n9a5wda-glibc-2.40-218/lib/ld-linux-x86-64.so.2: ELF 64-bit LSB shared object, x86-64, version 1 (GNU/Linux), static-pie linked, BuildID[sha1]=bd51a42f77a79acd5bd1c787dee61dbd1bbe1d58, not stripped
```

Phew! Turns out that it's statically linked, so the Linux kernel itself, which
has ELF interpretation facilities, can interpret it! Linux only needs to
delegate ELF files to `ld.so` when there's dynamic linking. We now have a base
case! No infinite recursion here!

Now, what happens when you `chmod +x` something that doesn't make sense to be
executed? Like this archive here?

```
astrid@chungus ~ ❯ chmod +x r
astrid@chungus rekexec ❯ file r
r: ASCII cpio archive (SVR4 with no CRC)
```

What's the result of `./r`?

```
astrid@chungus ~ ❯ ./r
zsh: exec format error: ./r
```

Okay, so this "ASCII cpio archive (SVR4 with no CRC)" has the magic string
`\x30\x37\x30\x37\x30\x31` at its head. Therefore, even if you `chmod +x`'d a
cpio file, you wouldn't be able to execute it!

After all, it wouldn't make any sense! How exactly does one execute a cpio file
as a program?

That would be ludicrous.

### it's executable, trust me bro

If you've ever installed Mono or Wine, you'll suddenly find yourself able to
execute EXE files. This is because they configure a kernel module called
`binfmt_misc`, which lets you tell the kernel "files with this magic string can
be interpreted using this interpreter."

And of course, if Linux can execute EXE files, it's definitely possible for it
to execute `cpio` files.

This QEMU command can act as an initrd interpreter by hosting a virtualized
Linux OS:

```sh
#!/bin/sh

set -x # print executed commands

exec qemu-system-x86_64 \
    -kernel /path/to/my/kernel \
    -initrd $1 \
    -append "console=ttyS0" \
    -nographic \
    -m 2G \
    -no-reboot
```

Put that script somewhere on your system, and you can register it in binfmt like
so:[^binfmt]

[^binfmt]:
    This statement approximately says "if you are asked to execute a file
    beginning with byte string `\x30\x37\x30\x37\x30\x31`, use
    `/path/to/my/script.sh` to handle it. For the other fields, see docs at
    https://docs.kernel.org/admin-guide/binfmt-misc.html for what they mean.

```sh
echo ':cpio:M::\x30\x37\x30\x37\x30\x31::/path/to/my/script.sh:' \
    > /proc/sys/fs/binfmt_misc/register
```

And with that, you can execute initrds that have the executable bit set.

```
astrid@chungus rekexec ❯ file i
i: ASCII cpio archive (SVR4 with no CRC)
astrid@chungus rekexec ❯ chmod +x i
astrid@chungus rekexec ❯ ./i
+ exec qemu-system-x86_64 -kernel /boot/kernels/5ngwg33rxpwc476b3bfixdqg4kx9qs62-linux-6.12.69-bzImage -initrd ./i -append console=ttyS0 -nographic -m 2G -no-reboot
SeaBIOS (version rel-1.17.0-0-gb52ca86e094d-prebuilt.qemu.org)


iPXE (http://ipxe.org) 00:03.0 CA00 PCI2.10 PnP PMM+7EFD1C90+7EF31C90 CA00



Booting from ROM...
Probing EDD (edd=off to disable)... o
[    0.000000] Linux version 6.12.69 (nixbld@localhost) (gcc (GCC) 14.3.0, GNU ld (GNU Binutils) 2.44) #1-NixOS SMP PREEMPT_DYNAMIC 6
[    0.000000] Command line: console=ttyS0
[    0.000000] BIOS-provided physical RAM map:
[    0.000000] BIOS-e820: [mem 0x0000000000000000-0x000000000009fbff] usable
[    0.000000] BIOS-e820: [mem 0x000000000009fc00-0x000000000009ffff] reserved
[    0.000000] BIOS-e820: [mem 0x00000000000f0000-0x00000000000fffff] reserved
[    0.000000] BIOS-e820: [mem 0x0000000000100000-0x000000007ffdffff] usable
[    0.000000] BIOS-e820: [mem 0x000000007ffe0000-0x000000007fffffff] reserved
[    0.000000] BIOS-e820: [mem 0x00000000fffc0000-0x00000000ffffffff] reserved
[    0.000000] BIOS-e820: [mem 0x000000fd00000000-0x000000ffffffffff] reserved
[    0.000000] NX (Execute Disable) protection: active
[    0.000000] APIC: Static calls initialized
[    0.000000] SMBIOS 2.8 present.
[    0.000000] DMI: QEMU Standard PC (i440FX + PIIX, 1996), BIOS rel-1.17.0-0-gb52ca86e094d-prebuilt.qemu.org 04/01/2014
[    0.000000] DMI: Memory slots populated: 1/1
[    0.000000] tsc: Fast TSC calibration using PIT
[    0.000000] tsc: Detected 3393.648 MHz processor
[    0.014905] last_pfn = 0x7ffe0 max_arch_pfn = 0x400000000
[    0.015525] MTRR map: 4 entries (3 fixed + 1 variable; max 19), built from 8 variable MTRRs
```

The interpreter for CPIO files is the kernel of a virtual OS on the OS you run
it from.

## The strangest loop

Our QEMU interpreter script can be thought of as creating a new stack frame of
the Linux environment. You're having your Linux distro calls another Linux
distro in a VM. This stack can tower infinitely large and make us hit the stack
frame limit of how big our memory is.

Let's apply a tail call optimization.

Here is a new interpreter:

```sh
#!/bin/sh

kexec --load /k --initrd $1 --reuse-cmdline
kexec --exec
```

Then let's put it in that piece of malware I gave you at
`/bin/cpio-interpreter`, and have `/init` register it as a binfmt handler:

```sh
#!/bin/sh
mkdir -p /proc
mount -t proc proc /proc
mount -t binfmt_misc none /proc/sys/fs/binfmt_misc

echo ':cpio:M::\x30\x37\x30\x37\x30\x31::/bin/cpio-interpreter:' \
  > /proc/sys/fs/binfmt_misc/register

find / | grep -v /r | grep -v /proc | cpio -vo -H newc > /r

chmod +x /r
exec /r
```

Now we have an initrd that derives the real init process, which is of cpio
format! Then it `exec`s it at the end, all in the nice and convenient
POSIXLY_CORRECT way that initramfses on most distros do!

### A last little thing

Besides the obvious, there's something else that's deeply wrong with what we've
done.

In most cases, using binfmt to execute scripts has to bottom out. Your
`#!/bin/sh` script has to be interpreted by `/bin/sh` which has to be
interpreted by `ld.so` which has to be interpreted directly by the kernel.

I've made a binfmt interpreter that runs another kernel and never bottoms out.

The interpreter for CPIO files on this system is the kernel of its next reboot.

## Conclusion

We learned what an initrd is in part 1 and used it to kill God in part 5.

Anyways, here's the [source code](https://github.com/ifd3f/rekexec/).[^nix]
kthxbai :3

[^nix]:
    Oh yeah, it was a Nix package. It turns out Nix is also capable of building
    OSes that _don't_ have a Nix store in them!
