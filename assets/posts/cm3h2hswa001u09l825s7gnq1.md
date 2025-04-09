---
title: "Notes on TDD & DDD From the Ground Up Live Coding"
datePublished: Thu Nov 14 2024 08:47:51 GMT+0000 (Coordinated Universal Time)
cuid: cm3h2hswa001u09l825s7gnq1
slug: notes-on-tdd-ddd-from-the-ground-up-live-coding
tags: tdd, testing, ddd, notes

---

Some short notes from Chris Simon's Talk [TDD & DDD From the Ground Up Live Coding](https://youtube.com/watch?v=1WBIUYJVnok&si=ZiyE8Hvx3U5LsNHR)

When choosing the right testing level, developers face an important trade-off. Higher-level tests provide better coverage for refactoring, but make it harder to pinpoint the exact location of failures. Finding the right balance is crucial for maintainable tests.

Domain modelling through traditional entity diagrams can lead to hidden assumptions. Event storming, by contrast, helps create more explicit models that better reflect how systems change over time. This approach brings us closer to the actual domain and helps reveal important details we might otherwise miss.