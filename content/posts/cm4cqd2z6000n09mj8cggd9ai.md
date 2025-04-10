---
title: "Notes on Preparatory Refactoring"
datePublished: Fri Dec 06 2024 12:36:53 GMT+0000 (Coordinated Universal Time)
cuid: cm4cqd2z6000n09mj8cggd9ai
slug: notes-on-preparatory-refactoring
tags: refactoring, tdd

---

Notes inspired by Emily Bache's short Youtube video [Design Better Code with Preparatory Refactoring in TDD | Demo](https://youtu.be/kHwzVlXTOw8?si=4q5GpbGYioN1VE_4).

A preparatory refactoring is a refactoring to make future changes to accommodate new requirements easy. By definition, current behavior should not be changed. Instead of using tests to drive changes, you use them to verify you are not breaking existing behavior.

A practical tip: add a pending test as a mental note to remember the goal of your refactoring. Say you have a new requirement coming in. TDD's mantra Red-Green-Refactor would require you to write a failing test first. However, it turns out, the current design is not really well-suited to accommodate this change. This means you may be facing Red status for a while if you decide to keep the test in. In addition, when you start refactoring to change the design, you may accidentally break existing functionality. This means you will have tests that are failing expectedly *and* unexpectedly. When refactoring you would ideally only see the tests fail when you made a mistake. An in-between approach could be: add the test for the new behavior, see it fail, set it to 'pending' and then proceed with the refactoring.

Green-Refactor-Green instead of Red-Green-Refactor. Preparatory refactoring does not mean we cannot use small steps to do the refactoring. Make a small change to the code, see the test still pass, continue; if a test fails roll back or fix. Green-Refactor-Green, keep an eye on your pending test.

Two further resources which include an example of how one could approach such a refactoring:

* Video: [Design Better Code with Preparatory Refactoring in TDD | Demo](https://youtu.be/kHwzVlXTOw8?si=4q5GpbGYioN1VE_4) by Emily Bache.
    
* Blog post: [An example of preparatory refactoring](https://martinfowler.com/articles/preparatory-refactoring-example.html) by Martin Fowler.