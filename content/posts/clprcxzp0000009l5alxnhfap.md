---
title: "FizzBuzz fun: Exploring Functional Programming Design Patterns : Monoids"
datePublished: Mon Dec 04 2023 20:22:12 GMT+0000 (Coordinated Universal Time)
cuid: clprcxzp0000009l5alxnhfap
slug: fizzbuzz-exploring-functional-programming-design-patterns-monoids
tags: software-development, scala, scala3, scala-basics

---

In this post, we will continue from where we left off in a previous article in this series, "FizzBuzz Fun in Scala: Combining Functions," and explore where further abstraction leads us in terms of functional programming design patterns. Our goal is to show that, despite their intimidating name, Monoids are a logical, reusable design pattern that emerges when reducing list-like structures to a single value—a task quite common in everyday development. As for Scala 3, we will also utilize contextual abstractions instead of Scala 2 implicits, to combine more complex structures from smaller ones.

## Combining values using a single abstraction

Below is the code we will start with. If you would like to understand this code better please see the [FizzBuzz fun in Scala: Combining functions](https://hashnode.com/edit/clpkz8w32000309l92w9nhd82) post.

```scala
// Commenting the Scala 3 version / Scala CLI directive
//> using scala 3.3.1

def combine[A](
    f1: A => Option[String],
    f2: A => Option[String]
): A => Option[String] =
  a =>
    (f1(a), f2(a)) match
      case (Some(s1), Some(s2)) => Some(s1 + s2)
      case (None, Some(s2))     => Some(s2)
      case (Some(s1), None)     => Some(s1)
      case (None, None)         => None

val fizzbuzzAt: Int => String =
  extension (word: String)
    def every(n: Int): Int => Option[String] = i =>
      if i % n == 0 then Some(word) else None

  val wordShouts = List(
    "Fizz".every(3),
    "Buzz".every(5)
  )
  val combined = wordShouts.fold(_ => None)(combine)
  val fizzbuzz: Int => String = i =>
    combined(i).getOrElse(i.toString)
  fizzbuzz
end fizzbuzzAt

def fizzbuzz(n: Int): List[String] =
  LazyList
    .from(1)
    .map(fizzbuzzAt)
    .take(n)
    .toList
end fizzbuzz

@main def fizzbuzz(): Unit =
  fizzbuzz(20).foreach(fb => print(fb + ","))
end fizzbuzz
```

<div data-node-type="callout">
<div data-node-type="callout-emoji">💡</div>
<div data-node-type="callout-text">Code can be run using <a target="_blank" rel="noopener noreferrer nofollow" href="" style="pointer-events: none"><strong>Scala CLI</strong></a>. Save a copy of a complete example in e.g. <code>FizzBuzz.scala</code> and run it like: <code>scala-cli run FizzBuzz.scala</code>. In this article, a complete runnable example can be recognised by a <code>//&gt; using scala 3.3.1</code> directive near the top of the file.</div>
</div>

We will begin by zooming in on how we 'combined' multiple functions with the type signature `Int => Option[String]` into a single one:

```scala
def combine[A](
    f1: A => Option[String],
    f2: A => Option[String]
): A => Option[String] =
  a =>
    (f1(a), f2(a)) match
      case (Some(s1), Some(s2)) => Some(s1 + s2)
      case (None, Some(s2))     => Some(s2)
      case (Some(s1), None)     => Some(s1)
      case (None, None)         => None
```

which is used as:

```scala
val combined = wordShouts.fold(_ => None)(combine)
```

In the \``fold`\` we seem to have two related functions so let's define them together:

```scala
trait Combiner[B]:
  def zero: B
  def combine(b1: B, b2: B): B
```

The `zero` function enables us to define an element that 'does not count' when combining elements. The `combine` function allows us to merge elements of the same type into another element of the same type. For example, consider integers: we could combine them using addition, in which case the zero element would be 0; alternatively, we could combine them using multiplication, in which case the zero element would be 1.

Looking at the type signature of `combine`:

```scala
def combine[A](
    f1: A => Option[String],
    f2: A => Option[String]
): A => Option[String]
```

We can see that `B` in `Combiner[B]` in this case would be of type `A => Option[String]`, so the instance for our type of `Combiner` would look like this:

```scala
new Combiner[A => Option[String]]:
  override def zero: A => Option[String] = ???
  override def combine(
        a1: A => Option[String],
        a2: A => Option[String]
    ): A => Option[String] = ???
end Combiner
```

Declaring the instance in an object (later we will group more `Combiner` instances together) we get:

```scala
object Combiner:
  def myCombiner[A] = new Combiner[A => Option[String]]:
    override def zero: A => Option[String] = ???
    override def combine(
        a1: A => Option[String],
        a2: A => Option[String]
    ): A => Option[String] = ???
end Combiner
```

To implement `zero` we can have a look at the `fold` usage above. For the `combine` method we can have a look at our original `combine` function:

```scala
object Combiner:
  def myCombiner[A] = new Combiner[A => Option[String]]:
    override def zero: A => Option[String] =
      _ => None
    override def combine(
        a1: A => Option[String],
        a2: A => Option[String]
    ): A => Option[String] =
      a =>
        (a1(a), a2(a)) match
          case (Some(s1), Some(s2)) => Some(s1 + s2)
          case (None, Some(s2)) => Some(s2)
          case (Some(s1), None) => Some(s1)
          case (None, None) => None
end Combiner
```

This `Combiner` we can then use to implement our original `combine` function:

```scala
import Combiner.*
def combine[A](
    f1: A => Option[String],
    f2: A => Option[String]
): A => Option[String] =
  myCombiner.combine(f1, f2)
```

At this point, our solution works again and we can remove our `combine` function entirely. Our \``fold`\` then becomes:

```scala
  val combined =
    wordShouts.fold(_ => None)(myCombiner.combine)
```

Or:

```scala
  val combined =
    wordShouts.fold(myCombiner.zero)(myCombiner.combine)
```

Since the `Combiner` type has all the behaviour required for `fold` ideally we would write:

```scala
val combined = wordShouts.fold(myCombiner)
```

Which we can do using an extension method:

```scala
extension[A] (as: List[A])
  def foldWith(c: Combiner[A]): A =
    as.fold(c.zero)(c.combine)

val combined = wordShouts.foldWith(myCombiner)
```

Note: using `fold` as a name for the extension method sadly does not seem to work so I renamed it to `foldWith`. I did delve deeper into this on why this is.

## Combine all the things!

At this point, you might be wondering, "Why go through all this trouble?". The answer lies in the fact that the two functions we defined in `Combiner` are precisely the arguments we provide to a `fold`. A fold represents the operation of combining elements within a collection while accounting for cases where the collection has no elements. This is something we do frequently. In functional programming, this type is commonly referred to by its mathematical name: a monoid.

If we rename our definition of `Combiner` to `Monoid`, then this is precisely how a Monoid is defined:

```scala
trait Monoid[A]:
  def zero: A
  def combine(b1: A, b2: A): A
```

Let's stick with `Combiner` for now and I'll revisit monoids further down in this post.

Let's have a closer look at our current instance of `Combiner`:

```scala
  def myCombiner[A] = new Combiner[A => Option[String]]:
    override def zero: A => Option[String] =
      _ => None
    override def combine(
        a1: A => Option[String],
        a2: A => Option[String]
    ): A => Option[String] =
      a =>
        (a1(a), a2(a)) match
          case (Some(s1), Some(s2)) => Some(s1 + s2)
          case (None, Some(s2))     => Some(s2)
          case (Some(s1), None)     => Some(s1)
          case (None, None)         => None
```

The code accomplishes several tasks:

1\. The pattern matching on the `Option` type essentially combines Options, which is not directly related to combining functions.

2\. The addition of `String`s, s1 + s2, can also be regarded as 'combining'.

3\. The `Combiner` is generic in its argument, but its return type is specific to `Option[String]`.

Currently, our `Combiner` instance appears to be combining functions, Options, and Strings simultaneously. Let's attempt to separate each of these into its own Combiner:

```scala
  def stringCombiner = new Combiner[String]:
    override def zero: String = ???
    override def combine(a1: String, a2: String): String = ???
    
  def optionCombiner[A] = new Combiner[Option[A]]:
    override def zero: Option[A] = ???
    override def combine(a1: Option[A], a2: Option[A]): Option[A] = ???
    
  def functionCombiner[A, B] = new Combiner[A => B]:
    override def zero: A => B = ???
    override def combine(f1: A => B, f2: A => B): A => B = ???
```

Since we started with combining functions, let's start with `functionCombiner`:

```scala
def functionCombiner[A, B] =
  new Combiner[A => B]:
    override def zero: A => B = a => ???
    override def combine(f1: A => B, f2: A => B): A => B =
      a =>
        val b1 = f1(a)
        val b2 = f2(a)
        ??? // combine b1 and b2
```

We need a way to combine `B`s, otherwise we cannot possibly continu. Do we have a way of combining Bs? Certainly, if pass one:

```scala
def functionCombiner[A, B](bc: Combiner[B]) =
  new Combiner[A => B]:
    override def zero: A => B = a => bc.zero
    override def combine(f1: A => B, f2: A => B): A => B =
      a =>
        val b1 = f1(a)
        val b2 = f2(a)
        bc.combine(b1, b2)
    end combine
end functionCombiner
```

In our `myCombiner` `B` is an `Option[String]`:

```scala
def myCombiner[A] = new Combiner[A => Option[String]]: ...
```

To implement our combiner, we need a way of combining Options. Let's start with `Option[String]` first:

```scala
  def optionStringCombiner = new Combiner[Option[String]]:
    override def zero: Option[String] = None
    override def combine(
        a1: Option[String],
        a2: Option[String]
    ): Option[String] =
      (a1, a2) match
        case (Some(s1), Some(s2)) => Some(s1 + s2)
        case (None, Some(s2))     => Some(s2)
        case (Some(s1), None)     => Some(s1)
        case (None, None)         => None
```

Here, similar to what we did for the `functionCombiner`, we can generalize by passing a `Combiner` for a generic parameter:

```scala
def optionCombiner[A](ac: Combiner[A]) =
  new Combiner[Option[A]]:
    override def zero: Option[A] = None
    override def combine(
        maybeA1: Option[A],
        maybeA2: Option[A]
    ): Option[A] =
      (maybeA1, maybeA2) match
        case (Some(a1), Some(a2)) => Some(ac.combine(a1, a2))
        case (None, Some(a2))     => Some(a2)
        case (Some(a1), None)     => Some(a1)
        case (None, None)         => None
```

The final piece of the puzzle involves combining strings. The `stringCombiner` is easy compared to the other combiners we have already implemented:

```scala
  def stringCombiner = new Combiner[String]:
    override def zero: String = ""
    override def combine(a1: String, a2: String): String = a1 + a2
```

If we now define the `optionsStringCombiner` in terms of the `optionCombiner` and `stringCombiner` we get:

```scala
def optionStringCombiner = optionCombiner(stringCombiner)
```

Our original `myCombiner`, of type `Combiner[A => Option[String]]` becomes:

```scala
def myCombiner[A] =
  functionCombiner[A, Option[String]](
    optionCombiner(
      stringCombiner
    )
  )
```

To recap what we have done:

* We defined a trait called `Combiner` to combine elements.
    
* We created an instance of this class that enables us to combine 'word shouts'. We modelled shouting words as functions: `Int => Option[String]`, meaning that given a number, we may or may not shout a word.
    
* We defined this instance using three more atomic instances: one that knows how to combine Strings, one that knows how to combine Options, and one that knows how to combine functions.
    
* We also defined an extension method for the `List` class that takes this `Combiner` instance and performs the actual combining.
    

All together:

```scala
// Commenting the Scala 3 version / Scala CLI directive
//> using scala 3.3.1

trait Combiner[A]:
  def zero: A
  def combine(a1: A, a2: A): A

object Combiner:
  def myCombiner[A] =
    functionCombiner[A, Option[String]](
      optionCombiner(
        stringCombiner
      )
    )

  def stringCombiner = new Combiner[String]:
    override def zero: String = ""
    override def combine(a1: String, a2: String): String =
      a1 + a2

  def optionCombiner[A](ac: Combiner[A]) =
    new Combiner[Option[A]]:
      override def zero: Option[A] = None
      override def combine(
          maybeA1: Option[A],
          maybeA2: Option[A]
      ): Option[A] =
        (maybeA1, maybeA2) match
          case (Some(a1), Some(a2)) =>
            Some(ac.combine(a1, a2))
          case (None, Some(a2)) => Some(a2)
          case (Some(a1), None) => Some(a1)
          case (None, None)     => None

  def functionCombiner[A, B](bc: Combiner[B]) =
    new Combiner[A => B]:
      override def zero: A => B = a => bc.zero
      override def combine(f1: A => B, f2: A => B): A => B =
        a =>
          val b1 = f1(a)
          val b2 = f2(a)
          bc.combine(b1, b2)
      end combine
  end functionCombiner

end Combiner

import Combiner.*

val fizzbuzzAt: Int => String =
  extension (word: String)
    def every(n: Int): Int => Option[String] = i =>
      if i % n == 0 then Some(word) else None

  extension [A](as: List[A])
    def foldWith(c: Combiner[A]): A =
      as.fold(c.zero)(c.combine)

  val wordShouts = List(
    "Fizz".every(3),
    "Buzz".every(5)
  )
  val combined = wordShouts.foldWith(myCombiner)
  val fizzbuzz: Int => String = i =>
    combined(i).getOrElse(i.toString)
  fizzbuzz
end fizzbuzzAt

def fizzbuzz(n: Int): List[String] =
  LazyList
    .from(1)
    .map(fizzbuzzAt)
    .take(n)
    .toList
end fizzbuzz

@main def fizzbuzz(): Unit =
  fizzbuzz(20).foreach(fb => print(fb + ","))
end fizzbuzz
```

## Contextual abstraction using given instances and using clauses

Scala 2's implicits enabled removing all the explicit wiring of our `Combiner`s. In Scala 3 this is now done using `given` instances and `using` clauses. All our Combiner instances can be declared with the `given` keyword, and wherever we need an instance of `Combiner` to be available we use the `using` keyword.

Here is the complete implementation; afterwards, I will highlight the most important parts.

```scala
// Commenting the Scala 3 version / Scala CLI directive
//> using scala 3.3.1

trait Combiner[A]:
  def zero: A
  def combine(a1: A, a2: A): A

object Combiner:

  given stringCombiner: Combiner[String] with
    override def zero: String = ""
    override def combine(a1: String, a2: String): String =
      a1 + a2

  given optionCombiner[A](using ac: Combiner[A]): Combiner[Option[A]] with
      override def zero: Option[A] = None
      override def combine(
          maybeA1: Option[A],
          maybeA2: Option[A]
      ): Option[A] =
        (maybeA1, maybeA2) match
          case (Some(a1), Some(a2)) =>
            Some(ac.combine(a1, a2))
          case (None, Some(a2)) => Some(a2)
          case (Some(a1), None) => Some(a1)
          case (None, None)     => None

  given functionCombiner[A, B] (using
                                cb: Combiner[B]
                               ): Combiner[A => B] with
      override def zero: A => B = a => cb.zero
      override def combine(f1: A => B, f2: A => B): A => B =
        a =>
          val b1 = f1(a)
          val b2 = f2(a)
          cb.combine(b1, b2)
      end combine
  end functionCombiner
end Combiner

import Combiner.given

val fizzbuzzAt: Int => String =
  extension (word: String)
    def every(n: Int): Int => Option[String] = i =>
      if i % n == 0 then Some(word) else None

  extension [A](as: List[A])
    def foldWith(using c: Combiner[A]): A =
      as.fold(c.zero)(c.combine)

  val wordShouts = List(
    "Fizz".every(3),
    "Buzz".every(5)
  )
  val combined = wordShouts.foldWith
  val fizzbuzz: Int => String = i =>
    combined(i).getOrElse(i.toString)
  fizzbuzz
end fizzbuzzAt

def fizzbuzz(n: Int): List[String] =
  LazyList
    .from(1)
    .map(fizzbuzzAt)
    .take(n)
    .toList
end fizzbuzz

@main def fizzbuzz(): Unit =
  fizzbuzz(20).foreach(fb => print(fb + ","))
end fizzbuzz
```

Highlights:

* The `Combiner` object contains all given instances of `Combiner` :
    

```scala

object Combiner:
  given stringCombiner: Combiner[String] with
  ...
  given optionCombiner[A](using ac: Combiner[A]): Combiner[Option[A]] with
  ...
  given functionCombiner[A, B] (using cb: Combiner[B]): Combiner[A => B] with
  ...
end Combiner
```

* given instances must be explicitly imported ('`import Combiner.*`' does *not* import them!):
    

```scala
import Combiner.given
```

* All elements can be folded into a single element, if a `Combiner` instance for A is given:
    

```scala
extension [A](as: List[A])
  def foldWith(using c: Combiner[A]): A =
    as.fold(c.zero)(c.combine)
```

* This fold needs a `Combiner` for `Int => Option[String]`:
    
    * `functionCombiner`'s type matches but needs a `Combiner` for `Option[String]`
        
    * `optionCombiner`'s type matches but needs a `Combiner` for `String`
        
    * `stringCombiner`'s type matches and does not need anything more
        

As such a 'complete' function combiner instance is given and will be used to fold all shouts into a single function:

```scala
val combined = wordShouts.foldWith
```

## Reuse with cats

in Scala 3 type classes are traits with one or more parameters whose implementations are provided as `given`instances. So `Combiner` is now a type class. As I mentioned earlier, the `Combiner` type class is known in category theory as a `Monoid` (which does come with some 'laws' which allow these structures to be well suited for parallelization but this is out of the scope of this article and for details I'll refer to the references at the end):

```scala
trait Monoid[A]:
  def zero: A
  def combine(b1: A, b2: A): A
```

The [*cats* library](https://typelevel.org/cats/) also defines this type class.

Let's now try to reuse the `Monoid` type class from cats.

Note the changes below:

* The cats lib dependency is declared and can be used: `//> using lib org.typelevel::cats-core:2.10.0`
    
* The cats `Monoid` type class is now used instead of `Combiner`
    
* The `zero` function is renamed to `empty` to match *cats'* names.
    
* The `Combiner` object is renamed to `CombinerInstances`
    
* The `foldWith` extension method is replaced with the `Monoid.combineAll` method on (a superclass of) `List` so. This method looks for a monoid that can combine the elements in the list of type `List[Int => Option[String]]`. This line is where most of the 'action' happens.
    

```scala
// Commenting the Scala 3 version / Scala CLI directive
//> using scala 3.3.1
//> using lib org.typelevel::cats-core:2.10.0
import cats.Monoid

object CombinerInstances:

  given stringCombiner: Monoid[String] with
    override def empty: String = ""
    override def combine(a1: String, a2: String): String =
      a1 + a2

  given optionCombiner[A](using ac: Monoid[A]): Monoid[Option[A]] with
      override def empty: Option[A] = None
      override def combine(
          maybeA1: Option[A],
          maybeA2: Option[A]
      ): Option[A] =
        (maybeA1, maybeA2) match
          case (Some(a1), Some(a2)) =>
            Some(ac.combine(a1, a2))
          case (None, Some(a2)) => Some(a2)
          case (Some(a1), None) => Some(a1)
          case (None, None)     => None

  given functionCombiner[A, B] (using
                                cb: Monoid[B]
                               ): Monoid[A => B] with
      override def empty: A => B = _ => cb.empty
      override def combine(f1: A => B, f2: A => B): A => B =
        a =>
          val b1 = f1(a)
          val b2 = f2(a)
          cb.combine(b1, b2)
      end combine
  end functionCombiner

end CombinerInstances

import CombinerInstances.given

val fizzbuzzAt: Int => String =
  extension (word: String)
    def every(n: Int): Int => Option[String] = i =>
      if i % n == 0 then Some(word) else None

  val wordShouts = List(
    "Fizz".every(3),
    "Buzz".every(5)
  )
  val combined = Monoid.combineAll(wordShouts)
  val fizzbuzz: Int => String = i =>
    combined(i).getOrElse(i.toString)
  fizzbuzz
end fizzbuzzAt

def fizzbuzz(n: Int): List[String] =
  LazyList
    .from(1)
    .map(fizzbuzzAt)
    .take(n)
    .toList
end fizzbuzz

@main def fizzbuzz(): Unit =
  fizzbuzz(20).foreach(fb => print(fb + ","))
end fizzbuzz
```

It turns out that the instances we defined ourselves are quite common, and cats also provides these monoid instances. For example, the monoid instance we defined for `String` can be imported using:

```scala
import cats.instances.string.given
```

Once this is done we do not need our instance in `MonoidInstances` anymore. The same can be done for the Option and function instances. So in the end we can delete all of our custom instances including the `CombinerInstances` object because it has become empty:

```scala
// Commenting the Scala 3 version / Scala CLI directive
//> using scala 3.3.1
//> using lib org.typelevel::cats-core:2.10.0
import cats.Monoid

import cats.instances.string.given
import cats.instances.option.given
import cats.instances.function.given

val fizzbuzzAt: Int => String =
  extension (word: String)
    def every(n: Int): Int => Option[String] = i =>
      if i % n == 0 then Some(word) else None

  val wordShouts = List(
    "Fizz".every(3),
    "Buzz".every(5)
  )
  val combined = Monoid.combineAll(wordShouts)
  val fizzbuzz: Int => String = i =>
    combined(i).getOrElse(i.toString)
  fizzbuzz
end fizzbuzzAt

def fizzbuzz(n: Int): List[String] =
  LazyList
    .from(1)
    .map(fizzbuzzAt)
    .take(n)
    .toList
end fizzbuzz

@main def fizzbuzz(): Unit =
  fizzbuzz(20).foreach(fb => print(fb + ","))
end fizzbuzz
```

This is the final version of Fizzbuzz in this article. As a side bar, note how we ended with 10 lines less than what we started with when rolling our own monoids manually.

## Conclusion

In conclusion, this article demonstrates how monoids emerge as a reusable design pattern when reducing list-like structures to a single value. By building upon the FizzBuzz example, we explored the process of defining and assembling reusable monoid instances, and eventually transitioned to using the Cats library for simplification and reuse.

Finally, here are some references if you would like to delve deeper into Monoids or functional programming in Scala:

* The [Scala red book](https://www.goodreads.com/book/show/13541678-functional-programming-in-scala) (note there is also a [second edition for Scala 3](https://www.goodreads.com/book/show/60509157-functional-programming-in-scala-second-edition?from_search=true&from_srp=true&qid=FLuOEyInel&rank=4))
    
* [FP Tower](https://www.fp-tower.com) has a very elegant and natural way of introducing Monoids in its course materials. As far as courses go this is one of the most engaging courses I ever took online. (Not free. Scala 2 at the time of writing.)
    
* [Monoid documentation of the cats library](https://typelevel.org/cats/typeclasses/monoid.html)
    
* Another explanation of what a Monoid is using FizzBuzz [on reddit](https://www.reddit.com/r/scala/comments/45gqpd/comment/czy732k/)