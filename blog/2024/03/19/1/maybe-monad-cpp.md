---
title: Fixing a bug with C++'s >>= operator
tagline:
  This is very important and I don't know why it has been broken for so long
tags:
  - cpp
  - functional-programming
  - haskell
  - monads
slug:
  ordinal: 1
  name: maybe-monad-cpp
date:
  created: 2024-03-19 23:57:45-07:00
  published: 2024-03-19 23:57:45-07:00
---

As someone who has done a bit of functional programming, I have to say that C++
has really strange design choices, like being eagerly evaluated and allowing the
user to cause all sorts of weird memory safety issues. But that's excusable --
not all languages can be as good as Haskell.

However, there is one thing that cannot be excused -- C++ has this weird bug
where the >>= operator represents right-bitshift-and-assign, and not what it
should be, which is the
[monadic bind operator](https://hackage.haskell.org/package/base-4.19.1.0/docs/Control-Monad.html#v:-62--62--61-)
like in Haskell.

Well, thanks to C++'s amazing operator overload system, I was able to write some
code that fixes the problem!

Here's some Haskell code:
```haskell
module Main where

main :: IO ()
main =
  let obj1 = Just 10
      obj2 = Nothing :: Maybe Int
      action x = Just (x + 10)
      newthing1 = obj1 >>= action
      newthing2 = obj2 >>= action
      composeTwice = obj1 >>= action >>= action
   in do
        putStrLn "Hello World!"
        putStrLn $ show obj1 ++ " becomes " ++ show newthing1
        putStrLn $ show obj2 ++ " becomes " ++ show newthing2
        putStrLn $ "Composed twice: " ++ show composeTwice
```

And here's the equivalent C++ code with my custom `Maybe` type:

```cpp
int main() {
  Maybe<int> obj1 = Maybe<int>::Just(10);
  Maybe<int> obj2 = Maybe<int>::Nothing();

  std::function<Maybe<int>(const int &)> action =
      ([](const int &a) { return Maybe<int>::Just(a + 10); });
  Maybe<int> newthing1 = (obj1 >>= action);
  Maybe<int> newthing2 = (obj2 >>= action);
  Maybe<int> compose_twice = ((obj1 >>= action) >>= action);

  std::cout << "Hello World!\n"
            << obj1 << " becomes " << newthing1 << "\n"
            << obj2 << " becomes " << newthing2 << "\n"
            << "Composed twice: " << compose_twice;
}
```

Here’s the output:

```
Hello World!
Just(10) becomes Just(20)
Nothing becomes Nothing
Composed twice: Just(30)
```

This kind of thing is extremely useful, and I would like C++ to fix this bug as
soon as possible.

## Full source

[The full source of the mockup is available as a gist](https://gist.github.com/ifd3f/be508885746961ed7ef2dae3b6487eaf),
but also mirrored here:

```cpp
#include <functional>
#include <iostream>
#include <memory>
#include <optional>
#include <sstream>
#include <string>

template <class A> class Maybe {
  std::unique_ptr<A> contents;
  Maybe(std::unique_ptr<A> contents) : contents(std::move(contents)) {}
  Maybe() {}

public:
  static Maybe<A> Just(A a) { return Maybe(std::make_unique<A>(a)); }
  static Maybe<A> Nothing() { return Maybe(); }

  bool is_just() const { return this->contents != nullptr; }

  const A &unwrap() const {
    if (this->contents) {
      return *this->contents;
    }
    throw "failed to unwrap nothing";
  }

  template <class B> auto operator>>=(std::function<Maybe<B>(const A &)> f) {
    if (this->is_just()) {
      return f(this->unwrap());
    }
    return Maybe<B>::Nothing();
  }
};

template <class A>
std::ostream &operator<<(std::ostream &os, const Maybe<A> &obj) {
  if (obj.is_just()) {
    return os << "Just(" << obj.unwrap() << ")";
  } else {
    return os << "Nothing";
  }
}

int main() {
  Maybe<int> obj1 = Maybe<int>::Just(10);
  Maybe<int> obj2 = Maybe<int>::Nothing();

  std::function<Maybe<int>(const int &)> action =
      ([](const int &a) { return Maybe<int>::Just(a + 10); });
  Maybe<int> newthing1 = (obj1 >>= action);
  Maybe<int> newthing2 = (obj2 >>= action);
  Maybe<int> compose_twice = ((obj1 >>= action) >>= action);

  std::cout << "Hello World!\n"
            << obj1 << " becomes " << newthing1 << "\n"
            << obj2 << " becomes " << newthing2 << "\n"
            << "Composed twice: " << compose_twice;
}
```

## `#undef SHITPOST`

Okay, you must be wondering how the hell this works. Well, the meat of the code
is here.

```cpp
template <class B> auto operator>>=(std::function<Maybe<B>(const A &)> f) {
  if (this->is_just()) {
    return f(this->unwrap());
  }
  return Maybe<B>::Nothing();
}
```

Let's break this down.

### Assignments are expressions

Assignments in C++ are expressions. Even =. Usually, they return the newly
assigned value. For example, if you write something like `a = b = c` that
assigns b to the value of c, and then a to the value.

How do add-and-assign operators work?

```cpp
#include <iostream>

int main() {
  int a = 2;
  int b = 3;
  int c = 7;
  int d = a += b += c;

  std::cout << a << " " << b << " " << c << " " << d;
}
```

has output `12 10 7 12`. What's happening here is:

1. `b += c` makes `c` stay the same and `b = b + c = 3 + 7 = 10`
2. `a += b` makes `b` stay the same and `a = a + b = 2 + 10 = 12`
3. `d = a` makes `d = a = 12`.

Other operate-and-assign operators have basically the same rules.

Notice that it's right-associative (i.e. this is `a += (b += c)`) whereas
Haskell's `>>=` is left-associative (as in, `a >>= b >>= c` is
`(a >>= b) >>= c`). That's why I have to write it like this:

```cpp
  Maybe<int> compose_twice = ((obj1 >>= action) >>= action);
```

### `auto` return type

I tried a type signature like this:

```cpp
template <class B> Maybe<B> operator>>=(std::function<Maybe<B>(const A &)> f);
```

This will error at the very first time it's used, even if you explicitly specify
the return type like so:

```auto
  Maybe<int> newthing1 = (obj1 >>= action);
```

This is because even though we did constrain `B` in the arguments, C++ seems too
stupid to guess what the return will be.

## C++23

I know `std::optional` got a `.and_then()` method added to it that's basically
this but less cursed. I have not tried a C++23 compiler, but I suspect you might
be able to generalize to anything that has a `.and_then()` method, although I
haven't tried that yet.
