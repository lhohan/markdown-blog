---
title: "Notes on 🚀 TDD, Where Did It All Go Wrong"
datePublished: Thu Nov 14 2024 09:26:28 GMT+0000 (Coordinated Universal Time)
cuid: cm3h3vg7w000c09l3aofng37t
slug: notes-on-tdd-where-did-it-all-go-wrong
tags: tdd, bdd, testing, gherkin, notes, tdd-vs-bdd

---

My short notes on [🚀 TDD, Where Did It All Go Wrong](https://youtube.com/watch?v=EZ05e7EMOLM&si=z4CYVriIZOOKwQNV)

Still a lot of testing wisdom in this talk (re-watched 6 years after publication).

## Focus on behaviour

Behaviour should be your primary focus when writing tests. The need for a new test should arise from new behaviour or requirements. While TDD is often contrasted with BDD, Kent Beck's early work already emphasised the importance of focusing on behaviour.

When writing tests, be mindful of coupling. Your tests should not all fail when you change your code. Write tests against public contracts where possible. In my experience, you can deviate from this rule if your unit under test has a stable internal contract.

This approach leads to better 'refactorability'. For a practical demonstration, I recommend watching [TDD & DDD From the Ground Up Live Coding by Chris Simon](https://youtube.com/watch?v=1WBIUYJVnok&si=ZiyE8Hvx3U5LsNHR).

## ATDD tools are not worth it if business is not actively participating

While I have promoted the use of ATDD (Acceptance Test Driven Development) tools like Gherkin, I must agree, the burden of translating between natural language and code can be 'horrible', to the point where internal frameworks are implemented to manage this complexity.

More importantly, the effort may not be worthwhile without an engaged business analyst or product owner. In my experience, business stakeholders rarely show interest in participating. While I've questioned whether I could have done more to encourage engagement, this talk confirms this is a common challenge. Though disappointing to acknowledge, as the practice appears promising on paper, this seems to be the reality we face.

That said, the consistent style these tools promote can still be valuable in test writing.

If you had success using Gherkin or similar tool I would be interested to learn how.

## Do not forget about Refactor

The TDD-cycle is Red-Green-**Refactor**. First fix the problem, then improve the design. The central idea is to decouple thinking about the problem from thinking about the design. The Refactor step is where the design is improved and is an integral part of the cycle.

This methodical approach leads to more maintainable code, contrasting with the approach of the 'Duct-tape Programmer' (this talk) or 'Tactical Tornado' ([Ousterhout](https://www.goodreads.com/book/show/39996759-a-philosophy-of-software-design?from_search=true&from_srp=true&qid=VdXFhGejkS&rank=1)) approach.

When you can improve the design without changing tests, you have achieved good decoupling.