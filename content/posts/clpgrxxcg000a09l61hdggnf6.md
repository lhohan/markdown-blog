---
title: "FizzBuzz Functional Fun in Scala 3"
datePublished: Mon Nov 27 2023 10:36:35 GMT+0000 (Coordinated Universal Time)
cuid: clpgrxxcg000a09l61hdggnf6
slug: fizzbuzz-functional-fun-in-scala-3
cover: https://cdn.hashnode.com/res/hashnode/image/stock/unsplash/FGYv8CDQBmg/upload/3df8c3248d3a98dba0f04a6fcdefaa10.jpeg
tags: scala, scala3, functional-scala

---

Nearly a decade ago, I wrote a post about [implementing FizzBuzz in a more functional manner](https://hans.lhoest.eu/fizzbuzz-functional-fun-in-scala), which also happened to be the final entry on a blog I started that year. Now, I want to dedicate more time to writing, and rebooting my blog seems to be a good way to achieve this. What better way to begin than by revisiting my previous post and picking up where I left off?

I believe the article is still fine, but reading it feels like looking at old code I wrote: slightly awkward. Now, I would change some parts which I may do in upcoming posts but let's start simple and rewrite in Scala 3. This is the code I left with last time:

```scala
def fizzbuzz(s: Int): List[String] = {

    // helper method inspired by haskell, cycle a list infinitely,
    def cycle(xs: List[String]): Stream[String] = Stream.continually(xs).flatten

    // a infinite cycle of "", "", "Fizz"
    val fizzes = cycle(List("", "", "Fizz"))
    // a infinite cycle of "", "", "", "", "Buzz"
    val buzzes = cycle(List("", "", "", "", "Buzz"))

    // zip the fizzes and buzzes, and concatenate them, result is "", "", "Fizz", "", "Buzz", "Fizz", ...
    val pattern = fizzes zip buzzes map { case (f, b) => f + b }
    // zip numbers with the pattern, if the pattern is empty keep the number, otherwise keep the pattern
    val numbers = Stream.from(1)
    val numbersAndPattern = numbers zip pattern map {
      case (n, p) => if (p.isEmpty) n.toString else p
    }

    numbersAndPattern take s toList
  }
```

Below is a Scala 3 version in which I comment on the changes. Note that I added a Scala 3 version of a \`main\` function which can be run using [Scala CLI.](https://scala-cli.virtuslab.org)

```scala
// Commenting the Scala 3 version / Scala CLI directive
//> using scala 3.3.1

// optional significant white spacing, we can omit all curly braces
@main def fizzbuzz(): Unit =
  def fizzbuzz(s: Int): List[String] =

    // `Stream` is replaced with `LazyList`, the latter being fully lazy.
    def cycle(xs: List[String]): LazyList[String] = LazyList.continually(xs).flatten

    val fizzes = cycle(List("", "", "Fizz"))
    val buzzes = cycle(List("", "", "", "", "Buzz"))

    // case matches on tuples are not required anymore
    val pattern = fizzes zip buzzes map(_ + _)
    val numbers = LazyList.from(1)
    // again case match on tuples removed
    val numbersAndPattern = numbers zip pattern map:
      (n, p) =>
        // if-then-else statement
        if p.isEmpty then n.toString else p
    
    // here extra braces are required
    (numbersAndPattern take s).toList
  // not required, explicit end clause for larger functions
  end fizzbuzz

  fizzbuzz(20).foreach(fb => print(fb + "," ))
```

This will print `1,2,Fizz,4,Buzz,Fizz,7,8,Fizz,Buzz,11,Fizz,13,14,FizzBuzz,16,17,Fizz,19,Buzz,`.

Most notable is the significant white spacing. I understand it is not to everybody's taste but I quite like it. I admit sometimes I get unexpected compiler errors because of it but nothing big not frequently. Another nice improvement which always was weird is that now the \`case\` matches on tuples can be removed when mapping.

Cleaning up a little a removing superfluous comments I leave it at:

```scala
// Commenting the Scala 3 version / Scala CLI directive
//> using scala 3.3.1

@main def fizzbuzz(): Unit =
  def fizzbuzz(s: Int): List[String] =

    def cycle(xs: List[String]): LazyList[String] = LazyList.continually(xs).flatten

    val fizzes = cycle(List("", "", "Fizz"))
    val buzzes = cycle(List("", "", "", "", "Buzz"))

    val pattern = fizzes zip buzzes map (_ + _)
    val numbers = LazyList.from(1)
    val numbersAndPattern = numbers zip pattern map: (n, p) =>
      if p.isEmpty then n.toString else p

    (numbersAndPattern take s).toList
  end fizzbuzz

  fizzbuzz(20).foreach(fb => print(fb + ","))
end fizzbuzz
```