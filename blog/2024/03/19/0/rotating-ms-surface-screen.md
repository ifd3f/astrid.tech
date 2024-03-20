---
title: Rotating my Linux Microsoft Surface's screen
tags:
  - linux
  - surface-pro
  - x11
slug:
  ordinal: 0
  name: rotating-ms-surface-string
date:
  created: 2024-03-17 22:35:45-07:00
  published: 2024-03-19 21:50:25-07:00
---

Suppose you have a Microsoft Surface Pro running Linux with all the
[Linux Surface](https://github.com/linux-surface/linux-surface) patches and
[IPTS daemon](https://github.com/linux-surface/iptsd) running for your stylus.

To rotate the display left, first find your screen by executing
`xrandr --listmonitors`, which will have output looking like this:

```
Monitors: 1
 0: +*eDP-1 2736/260x1824/173+0+0  eDP-1
```

Then, you execute the following (in my case I would use `display_name=eDP-1`):

```sh
xrandr --output $display_name --rotate left
```

However, this isn't enough, because the stylus's coordinate system did not
rotate. Your pen's coordinates will look like they've been reflected across a
diagonal line.

Find your stylus by running `xinput`:

```
⎡ Virtual core pointer                    	id=2	[master pointer  (3)]
⎜   ↳ Virtual core XTEST pointer              	id=4	[slave  pointer  (2)]
⎜   ↳ IPTS 045E:001F Touchscreen              	id=7	[slave  pointer  (2)]
⎜   ↳ Microsoft Surface Type Cover Mouse      	id=9	[slave  pointer  (2)]
⎜   ↳ Microsoft Surface Type Cover Keyboard   	id=10	[slave  pointer  (2)]
⎜   ↳ Microsoft Surface Type Cover Touchpad   	id=15	[slave  pointer  (2)]
⎜   ↳ IPTS Touch                              	id=12	[slave  pointer  (2)]
⎜   ↳ IPTS Stylus Pen (0)                     	id=13	[slave  pointer  (2)]
⎜   ↳ IPTS Stylus Eraser (0)                  	id=14	[slave  pointer  (2)]
⎣ Virtual core keyboard                   	id=3	[master keyboard (2)]
    ↳ Virtual core XTEST keyboard             	id=5	[slave  keyboard (3)]
    ↳ Video Bus                               	id=6	[slave  keyboard (3)]
    ↳ Microsoft Surface Type Cover Keyboard   	id=11	[slave  keyboard (3)]
    ↳ IPTS Stylus                             	id=8	[slave  keyboard (3)]
```

Notice that there may be multiple IDs associated with the stylus. You will have
to execute the following command for all of them:

```sh
xinput set-prop $xinput_id 191 0 -1 1 1 0 0 0 0 1
```

To reverse the process, you run the following (again, over all `$xinput_id`s):

```sh
xrandr --output $display_name --rotate left
xinput set-prop $xinput_id 191 1 0 0 0 1 0 0 0 1
```

## A wrapper script

Instead of calling that directly, here's a fairly janky but working Python
script I wrote that wraps those commands:

```python
#!/usr/bin/env python3

import os
import re
import subprocess
import sys


MATRICES = {
    'normal': '1 0 0 0 1 0 0 0 1',
    'left': '0 -1 1 1 0 0 0 0 1',
}

COORDINATE_TRANFORM_ATTR = 191


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in MATRICES:
        display_help()
        exit(1)

    for input_id in find_ipts_ids():
        rotate_input_by_id(input_id, sys.argv[1])
    rotate_primary_screen(sys.argv[1])


def rotate_input_by_id(xinput_id: int, rotation: str):
    matrix = MATRICES[rotation]
    cmd = f"xinput set-prop {xinput_id} {COORDINATE_TRANFORM_ATTR} {matrix}"
    os.system(cmd)


def rotate_primary_screen(rotation: str):
    os.system(f'xrandr --output eDP-1 --primary --mode 2736x1824 --pos 0x0 --rotate {rotation}')


def find_ipts_ids():
    xinput_result = subprocess.check_output('xinput', shell=True).decode()
    for l in xinput_result.splitlines():
        if 'IPTS' in l and 'pointer' in l:
            yield int(re.search(r'id=(\d+)', l).group(1))


def display_help():
    print("simple script for rotating the surface's screen")
    print(f"usage: {sys.argv[0]} <normal | left>")


if __name__ == '__main__':
    main()
```

Ignore the inconsistency between `os.system` and `subprocess.run` and the whole
"running code in a shell" thing we're doing, it was hacked together on a plane,
give me some slack :)

## Explanation

What's going on with these values?

```python
MATRICES = {
    'normal': '1 0 0 0 1 0 0 0 1',
    'left': '0 -1 1 1 0 0 0 0 1',
}
```

These are transformation matrices. Your touchscreen digitizer puts inputs into a
specific coordinate space, and these matrices can be applied to that coordinate
space to tweak it. For more explanation,
[see this page from the Ubuntu wiki](https://wiki.ubuntu.com/X/InputCoordinateTransformation).
