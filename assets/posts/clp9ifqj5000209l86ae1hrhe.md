---
title: "FizzBuzz Functional Fun in Scala"
datePublished: Sun Feb 23 2014 00:00:00 GMT+0000 (Coordinated Universal Time)
cuid: clp9ifqj5000209l86ae1hrhe
slug: fizzbuzz-functional-fun-in-scala
tags: scala, functional-scala

---

*Updated with a more functional implementation of FizzBuzz November 2015*

*Updated with a link to an implementation using* `Monoids` July 2016

[FizzBuzz](http://en.wikipedia.org/wiki/Fizz_buzz) fun in Scala using Scala [Streams](http://www.scala-lang.org/api/current/index.html#scala.collection.immutable.Stream).

The straightforward implementation of FizzBuzz usually involves defining a list or array of fixed size 100, populating it using a loop over the index and setting the value in the list according to our Fizz Buzz specification.

[Streams](http://www.scala-lang.org/api/current/index.html#scala.collection.immutable.Stream) are lazy lists in which elements are only evaluated when they are needed. This allows for a seemingly infinite recursive loop-looking definition as shown in the Fibonacci example below. The first line builds a sequence of sums, the sum of `a` and `fibFrom` looks as it if will never end if not for the lazy evaluation.

```scala
def fibFrom(a: Int, b: Int): Stream[Int] = a #:: fibFrom(b, a + b)

def fibonacciStream(n: Int) = fibFrom(1, 1).take(n).toList.last
```

The 2nd line is there for clarity to show how to calculate the nth Fibonacci number based on our `Stream` definition. The `take` method will return a Stream, so nothing is being evaluated until `toList` is called. (Note that in case of an infinite collection `toList` may never terminate.)

Here is an alternate implementation of FizzBuzz using the [Stream](http://www.scala-lang.org/api/current/index.html#scala.collection.immutable.Stream) class in the standard Scala API.

```scala
def fizzbuzz(s: Int): List[String] = {
  
  def shout(i: Int): String =
    if (i % 15 == 0) "FizzBuzz" else
    if (i % 3 == 0) "Fizz" else
    if (i % 5 == 0) "Buzz" else
      i.toString
  
  val fb: Stream[String] = Stream.from(1).map(i => shout(i))
  
  (fb take s).toList
}
```

[`Stream.from(start:Int)`](http://www.scala-lang.org/api/current/index.html#scala.collection.immutable.Stream$) creates an infinite stream of integers and we map them using a function according to the FizzBuzz specification. We then take the first `n` of this infinite list and convert it to a 'normal' Scala List.

To print the first 100:

```scala
fizzbuzz(100).foreach{ println(_) }
```

As a little extra, this is how we could test our FizzBuzz implementation using [ScalaCheck](http://www.scalacheck.org/):

```scala
import org.scalacheck.Gen
import org.scalatest.FunSuite
import org.scalacheck.Prop.forAll
import org.scalatest.prop.Checkers

class FizzBuzzTest extends FunSuite with Checkers {

  test("fizzbuzz") {
    val fb = fizzbuzz(100)
    val range = Gen.choose[Int](1, 100)
    val fizzbuzzProperty = forAll(range) {
      n =>
        if (n % 15 == 0) fb(n - 1) == "FizzBuzz" else
        if (n % 5 == 0) fb(n - 1) == "Buzz" else
        if (n % 3 == 0) fb(n - 1) == "Fizz" else
          fb(n - 1) == n.toString
  
    }
    check(fizzbuzzProperty)
  }
}
```

*A more functional implementation of FizzBuzz (added November 2015)*

Thanks to [Dierk König's implementation in Frege](https://dierk.gitbooks.io/fregegoodness/content/src/docs/asciidoc/fizzbuzz.html):

```scala
  def fizzbuzz(s: Int): List[String] = {

    // helper method inspired by haskell, cycle a list infinitely,
    def cycle(xs: List[String]): Stream[String] = Stream.continually(xs).flatten

    val numbers = Stream.from(1)

    // a infinite cycle of "", "", "Fizz"
    val fizzes = cycle(List("", "", "Fizz"))
    // a infinite cycle of "", "", "", "", "Buzz"
    val buzzes = cycle(List("", "", "", "", "Buzz"))

    // zip the fizzes and buzzes, and concatenate them, result is "", "", "Fizz", "", "Buzz", "Fizz", ...
    val pattern = fizzes zip buzzes map { case (f, b) => f + b }
    // zip numbers with the pattern, if the pattern is empty keep the number, otherwise keep the pattern
    val numbersAndPattern = numbers zip pattern map {
      case (n, p) => if (p.isEmpty) n.toString else p
    }

    numbersAndPattern take s toList
  }
```

What I like about this implementation:

* Much fewer conditionals. There is only 1 compared to 4 in the first implementation. And the order matters in the latter, if I do not put the `i % 15 == 0` check first the logic will fall apart. So if changes need to be made it is easier to introduce bugs in the one containing more conditionals.
    
* The code can be more easily built incrementally using the REPL. It just *flows* nicer. The `shout` method needs to be entered in one go, all or nothing.
    
* Closer to the specification. Consider the `i % 15 == 0` check in the first implementation a bit more: to fit the FizzBuzz specification better it would be more accurate to write it as `i % 3 == 0 || i % 5 == 0`. This would up the conditional checks to 5 though. At first sight, the concatenation in the functional implementation felt like a lucky shot, but in fact, it fits exactly the specification of FizzBuzz: I suppose it is no coincidence we are shouting 'FizzBuzz', a concatenation, instead of 'Boo!'.
    

Note there is also `zipWithIndex` in Scala but I maintained the same "`zip` and `map`"-logic to keep the code more consistent and hence more simple.

*A functional implementation using* `Monoids` (Added July 2016)

Monoids also provide a way of implementing FizzBuzz in a more functional way, check it out [here](https://www.reddit.com/r/scala/comments/45gqpd/whats_a_monoid/czy732k). At the moment I do not have the time to go into detail but it is a nice example of practical use of monoids.