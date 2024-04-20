---
title: That One Time Hacker News Killed My Website
tagline: And how I fixed it
tags:
  - vercel
  - devops
  - project:astrid-tech
slug:
  ordinal: 0
  name: that-one-time-hacker-news-killed-my-website
date:
  created: 2024-02-26 18:50:56-08:00
  published: 2024-02-26 19:55:36-08:00
  updated: 2024-04-19 19:43:17-07:00
---

Hey, remember how I used to blog regularly? Remember how astrid.tech used to get
a new post every few weeks? Remember when the last post wasn't over a year ago?
[Remember that Blink Mini reverse-engineering series I did 2 years ago that I never ended up finishing](https://astrid.tech/2022/07/07/0/blink-mini-disassembly/)?
I thought I would milk a post out of my own technical ineptitude during that
era.

~~Please note that I _would_ have placed more images in this post, but the fact
that it's been over a year means that most of the data has gone out of
retention.~~ EDIT: i found the screenshots!!!

## Day of the incident

It's 2022, the day right before Thanksgiving. I'm with my partner's family, and
I open my phone. I find a Discord message from a friend:

> Did your website go down

I blink. No way it could have gone down. It's dead simple and I hadn't written a
post in a while.

I went to my site, and I was presented with a Vercel error page.

![This deployment has been disabled. 402 Payment Required](https://s3.us-west-000.backblazeb2.com/nyaabucket/19e0f2f009d1a26e69e135b50b749f218ac6235f36d3a0ff3fb29d74f81dee51.png)

Oh no.

I checked my Vercel dashboard. They say that they've blocked my website because
it's exceeded the quota. "Nonsense," I said. "There's no way my website could
exceed my quota. Nobody reads it!"

But the graphs said otherwise. They said that my quota was very, very, very
exceeded. In fact, I was 3.13 times above the quota, at 313G of egress when I
should have had a max of 100G of egress!

![egress graphs showing 313% over thee limit](https://s3.us-west-000.backblazeb2.com/nyaabucket/789cc0924bfced20becc77834794f23047a837c69604bc943afecc0c4511784b.png)

### Spying on my users and selling their data

So I consulted the good ol' trusty Google Analytics, and they said that all the
referrers for that page came from `ycombinator.com`. I don't remember how I
figured out where my HN link was -- possibly, an online friend told me. Either
way, I figured out that it was
[this link](https://news.ycombinator.com/item?id=33683122).

It turns out that I was wrong -- people _were_ reading my website. Too many
people. Though I suppose that for clout-chasing purposes, that's not the worst
reason to get my website killed.

### Finding the thing that sucked the data

But still, that didn't explain why I was making so much egress. I'm only serving
a bit of HTML, right? Going back to Vercel, I looked at an egress breakdown by
link... and it was entirely out of the images.

![all my valuable egress is being eaten by jpgs and pngs](https://s3.us-west-000.backblazeb2.com/nyaabucket/166c2ef6d05b06b65866a2f8ab73f0f1cb4072c35be8f8ba8b7a776e033363a4.png)

In other words, I had to move images out of Vercel and onto something else.

## Fixing the problem

This incident forced me do what I was meaning to do for a long time: go and
actually upload my images to object storage.

### Putting the images in a better place

[Backblaze](https://www.backblaze.com/) is the object storage provider I use
because I'm not an organization with compliance or uptime requirements or tons
of money.

I wrote an uploading script and a codemod script, both in Python, using the
[boto3](https://pypi.org/project/boto3/) library.

- The upload script recursed through my directories and uploaded images to
  Backblaze, while saving their URLs into a CSV.
- The codemod script recursed through my directories for Markdown files, and
  relinked images based on that CSV. This script contained awful hacks like the
  regex `\!\[.*\]\((.*)\)` for finding links to images.

There was a little caveat with the upload script -- I had to specify the
content-type and content-encoding or else they would not be recognized as images
by the thing. Luckily, Python has a built-in library called
[mimetypes](https://docs.python.org/3/library/mimetypes.html).

Once that was all done, I published my blog, and then all the images were linked
to files hosted in Backblaze.

### Begging Vercel to please lift my block 🥺

The last thing to do was go and ask Vercel to lift my quota. Honestly, I'm
surprised they didn't just cut off my website as soon as I hit the limit, and
that they were actually generous enough to let it go to 313% before blocking me.

I filed a support issue in Vercel:

> Sent: 2022-11-23 22:44
>
> Subject: Usage block
>
> ---
>
> Hello, my personal blog has been blocked due to an unforseen surge in traffic.
> This was caused by the site hosting multiple large images, an issue that I
> have just fixed by moving the images to external object hosting on Backblaze,
> which will prevent this from happening again. Could you unblock it now?

I hit send, and I waited for the response. The next morning:

> Sent: 2022-11-24 05:59
>
> Subject: Re: Usage block
>
> ---
>
> Hi there,
>
> Thanks for reaching out to Vercel Support!
>
> We appreciate your co-operation and appreciate that you've made changes to
> reduce the usage. We have unblocked your account for you now.
>
> Please do consider our Terms of Service and Fair Use Policy whilst using our
> service.
>
> Kind regards, Rob
>
> Rob Peters
>
> ▲ Senior Customer Support Engineer at Vercel

and astrid.tech was back in business!

## Conclusion

just use object storage for your images, y'all

EDIT: i found the images again and put them in
