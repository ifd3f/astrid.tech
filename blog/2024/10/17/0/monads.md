---
title: In Defenestration of Monads
tagline: A Praguematic Approach
tags:
  - functional-programming
  - haskell
  - monads
slug:
  ordinal: 0
  name: monads
date:
  created: 2024-10-17 11:38:08-07:00
  published: 2024-10-17 11:38:08-07:00
---

```irc
<simplicius> what's a monad?
<ast3r> a monyad is a monyoid in the categowy of endofunctors!
<simplicius> not helpful.
```

A monad is a big fancy word that means something really abstract. A monad is also a
kind of functor. A functor is also a fancy words that means something abstract.
These words are commonly used by functional programmers because they want to
sound fancy and write abstract code.

It is a universal law of the universe that everyone who learns Haskell ends up writing a blog post about monads. I have been trying to buck that trend, but unfortunately, my hand has been forced.

So, in this essay I will defenestrate the concept of Haskell monads off the ivory tower of functional programming.

## Motivation

An object oriented programmer looks at a system and sees objects that interact with each other, calling each others functions, a complex web of dependencies...

Functional programmers look at a system and think that one part of it is eerily similar to other systems.

Isn't it weird

## Functors

This is what a functor is:

```haskell
class Functor f where
  fmap :: (a -> b) -> f a -> f b
```

```irc
<ast3r> class isn't the type that OOP people are familiar with!!! it's more like an interface!!!
```

If you aren't familiar with Haskell's syntax, you may be more comfortable with a definition in pseudo-Rust (where `trait` is kinda like an interface too):

```rust
trait Functor<A> {
  fn fmap<A, B>(original: Self<A>, mapper: Fn(A) -> B) -> Self<B>;
}
```

Here's a translation into English:

> In order for a type `f` to be a functor, it must be generic over 1 type (so stuff like Lists, which are generic over a value type are in, but Maps, which need a key and value type, are out). It must also implement a function called `fmap`, which:
> - takes in a function turning `a` into `b`
> - takes in a functor in `a`
> - returns a functor in `b`

Let's think about what `fmap` means in the case of something concrete, like a list.

```haskell
fmap :: (a -> b) -> [a] -> [b]

fmap (\x -> x * 3) [1, 2, 3, 4]  -- [3, 6, 9, 12]
```

Doesn't this sound like the `map` function? In fact, this is exactly what it is. In fact, [it is literally defined
in the Haskell source code as so](https://hackage.haskell.org/package/base-4.19.0.0/docs/src/GHC.Base.html#Functor):

```
instance Functor [] where
    fmap = map
```
So, a functor can be thought of as a container of some results that lets you turn it into a container of some other results.

If your list is empty, nothing happens. Just like in `map`. You just get an empty list, except instead of having 0 `a`'s, now you have 0 `b`'s.

### Other things that are functors

#### Containers

Sets, queues, heaps, and arrays are functors in a similar way -- you can map over every element in the container to make a new container.

#### Maps

```irc
<simplicius> Wait, didn't you say maps aren't functors because they take 2 types instead of 1?
```

That's true, `Map` is not a functor. However, if you already have a key type `k`, then `Map k` is -- thus, `fmap` is defined over the values of the map, and we completely ignore the `k` type.

#### Maybe

`Maybe` is a functor. In fact, you can define it pretty trivially like so:

```haskell
instance Functor Maybe where
  fmap f (Just x) = Just (f x)
  fmap f Nothing = Nothing
```

This is how most functor definitions end up looking.

```irc
<ast3r> it's like a list that can only be 1 element at most!
```

## Monads

```irc
<simplicius> wait, why are we skipping applicatives?
<ast3r> shhhhhhhh we'll get to it later!
```

## Applicative functors

Applicative functor is a fancy word for "I have 2 functors and I would like to be able to merge those 2 functors together into a new spicy functor." One of the ways to define it in Haskell is this:

```haskell
class Functor f => Applicative f where
    pure :: a -> f a
    liftA2 :: (a -> b -> c) -> f a -> f b -> f c
```

We have a function called `pure`, which you can think of as wrapping
