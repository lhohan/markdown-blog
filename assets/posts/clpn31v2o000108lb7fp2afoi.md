---
title: "Simplifying if-complexity in FizzBuzz"
datePublished: Fri Dec 01 2023 20:34:12 GMT+0000 (Coordinated Universal Time)
cuid: clpn31v2o000108lb7fp2afoi
slug: simplifying-if-complexity-in-fizzbuzz
tags: scala, functional-programming, scala3, scala-basics

---

In this series, I've mentioned that using an `if`\-expression in the FizzBuzz problem can be more error-prone and complex compared to functional approaches. In this brief article, I'll demonstrate why that's the case.

Let's start with a simple working implementation using `if`s:

```scala
// Commenting the Scala 3 version / Scala CLI directive
//> using scala 3.3.1

def fizzbuzz(s: Int): List[String] =

  def shout(i: Int): String =
    if i % 15 == 0 then "FizzBuzz"
    else if i % 3 == 0 then "Fizz"
    else if i % 5 == 0 then "Buzz"
    else
      i.toString

  val fb: LazyList[String] = LazyList.from(1).map(shout)

  (fb take s).toList
end fizzbuzz

@main def fizzbuzz(): Unit =
  fizzbuzz(40).foreach(fb => print(fb + ","))
end fizzbuzz
```

Suppose the requirement is now amended to include printing "Bazz" for every even number, such as printing "FizzBuzzBazz" at n = 30.

```scala
// Commenting the Scala 3 version / Scala CLI directive
//> using scala 3.3.1

def fizzbuzz(s: Int): List[String] =

  def shout(i: Int): String =
    if i % 30 == 0 then "FizzBuzzBazz"
    else if i % 15 == 0 then "FizzBuzz"
    else if i % 10 == 0 then "BuzzBazz"
    else if i % 6 == 0 then "FizzBazz"
    else if i % 5 == 0 then "Buzz"
    else if i % 3 == 0 then "Fizz"
    else if i % 2 == 0 then "Bazz"
    else
      i.toString

  val fb: LazyList[String] = LazyList.from(1).map(shout)
  (fb take s).toList
end fizzbuzz

@main def fizzbuzz(): Unit =
  fizzbuzz(40).foreach(fb => print(fb + ","))
end fizzbuzz
```

It is clear that adding more words to this type of if-expression will cause it to expand rapidly and become increasingly difficult to get right. This is what I meant when I said extending the `if`\-expression is error-prone.

Of course, this `if`\-expression is just the simplest and most commonly chosen solution by developers (I admit we've used this exercise as a hiring question as well), but that doesn't mean it can't be improved further, starting from its current state. Let's explore how to enhance it, beginning with using a mutable collection in the `shout` function:

```scala
def shout(i: Int): String =
  val sb = StringBuilder("")

  if i % 3 == 0 then sb.append("Fizz")
  if i % 5 == 0 then sb.append("Buzz")
  if i % 2 == 0 then sb.append("Bazz")

  if sb.isEmpty then i.toString
  else sb.toString()
end shout
```

Since the `StringBuilder` is local to the `shout` function, it doesn't break referential transparency, so I don't mind. However, what bothers me a bit more are the `if`\-statements that are no longer expressions, meaning they don't resolve to a value which is used further.

Let's improve by defining a method with an explicit `Unit` return type. The fact that a method returns `Unit` indicates it is performing a side effect.

```scala
def shout(i: Int): String =
  val sb = StringBuilder("")
  def maybeAppend(word: String, turn: Int): Unit =
    if i % turn == 0 then sb.append(word) else ()

  maybeAppend("Fizz", 3)
  maybeAppend("Buzz", 5)
  maybeAppend("Bazz", 2)

  if sb.isEmpty then i.toString
  else sb.toString()
end shout
```

Putting the word-turn combination in a `List` improves even more:

```scala
def shout(i: Int): String =
  val sb = StringBuilder("")
  def maybeAppend(word: String, turn: Int): Unit =
    if i % turn == 0 then sb.append(word) else ()

  List(("Fizz", 3), ("Buzz", 5), ("Bazz", 2)).foreach(maybeAppend)

  if sb.isEmpty then i.toString
  else sb.toString()
end shout
```

At this point inlining the `maybeAppend` method again is probably more clear:

```scala
def shout(i: Int): String =
  val sb = StringBuilder("")

  List(("Fizz", 3), ("Buzz", 5), ("Bazz", 2)).foreach:
    (word: String, turn: Int) =>
      if i % turn == 0 then sb.append(word) else ()

  if sb.isEmpty then i.toString
  else sb.toString()
end shout
```

As a final step in this article, we can then get rid of the mutable collection by using a `foldLeft`:

```scala
def shout(i: Int): String =
  val words = List(("Fizz", 3), ("Buzz", 5), ("Bazz", 2))
    .foldLeft(""):
      case (acc, (word: String, turn: Int)) =>
        if i % turn == 0 then acc + word else acc
  end words

  if words.isBlank then i.toString
  else words
end shout
```

The complete implementation becomes then:

```scala
// Commenting the Scala 3 version / Scala CLI directive
//> using scala 3.3.1

def fizzbuzz(s: Int): List[String] =

  def shout(i: Int): String =
    val words = List(("Fizz", 3), ("Buzz", 5), ("Bazz", 2))
      .foldLeft(""):
        case (acc, (word: String, turn: Int)) =>
          if i % turn == 0 then acc + word else acc
    end words

    if words.isBlank then i.toString
    else words
  end shout

  val fb: LazyList[String] = LazyList.from(1).map(shout)
  (fb take s).toList
end fizzbuzz

@main def fizzbuzz(): Unit =
  fizzbuzz(40).foreach(fb => print(fb + ","))
end fizzbuzz
```

Extending this solution to even more FizzBuzz words is now as simple as adding an element to a list while the implementation only uses simple functions and is relatively easy to understand.

We began this article by demonstrating that an initial if-expression-based solution for the FizzBuzz problem can be error-prone and complicated when expanded. Through step-by-step refactoring, we arrived at a solution using functional programming constructs. Our final solution is more future-proof and easily understandable, allowing for extensions with minimal effort.