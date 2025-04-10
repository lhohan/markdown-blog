---
title: "Fizzbuzz fun in Scala: A straightforward implementation"
datePublished: Wed Nov 29 2023 10:41:13 GMT+0000 (Coordinated Universal Time)
cuid: clpjmzkrh000108ldcsbze4s8
slug: fizzbuzz-fun-in-scala-a-straightforward-implementation
tags: scala3, scala-basics

---

In previous articles of this series, I examined various implementations and meanwhile experimented with others at different levels of abstraction. However, before delving into those, I wanted to present what I believe to be the most straightforward approach, which allows for easy reading and extension.

<div data-node-type="callout">
<div data-node-type="callout-emoji">💡</div>
<div data-node-type="callout-text">Code can be run using <a target="_blank" rel="noopener noreferrer nofollow" href="" style="pointer-events: none">Scala CLI</a>. Save a copy of a complete example in e.g. `FizzBuzz.scala` and run it like: `scala-cli run FizzBuzz.scala`.</div>
</div>

The below example makes use of simple functions:

```scala
// Commenting the Scala 3 version / Scala CLI directive:
//> using scala 3.3.1

@main def fizzbuzz(): Unit =

  def fizzbuzz(n: Int): List[String] =
    // a lazy list's elements or only evaluated when needed
    LazyList
      // create a lazy list counting infinitely from 1
      .from(1)
      // map to a tuple where first element will hold the result
      .map(i => ("", i))
      // implement FizzBuzz the logic using easy to read functions
      .map((s, i) => if i % 3 == 0 then (s + "Fizz", i) else (s, i))
      .map((s, i) => if i % 5 == 0 then (s + "Buzz", i) else (s, i))
//    easy to extend:
//      .map((s, i) => if i % 2 == 0 then (s + "Bazz", i) else (s,i))
      // convert the tuples to a list of Strings
      // empty strings are turned into the index number
      .map((s, i) => if s.isBlank then i.toString else s)
      // take the request number of elements
      // remove this line and the next line will never terminate
      .take(n)
      // here we turn the lazy list into an eager, evaluated list
      .toList
  end fizzbuzz

  fizzbuzz(20).foreach(fb => print(fb + ","))
end fizzbuzz
```

Note how much easier it is to extend than if you would these \`if-else\` statements:

```scala
def shout(i: Int): String =
    if (i % 15 == 0) "FizzBuzz" else
    if (i % 3 == 0) "Fizz" else
    if (i % 5 == 0) "Buzz" else
      i.toString
```