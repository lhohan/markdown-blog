---
title: "Software Architecture Note: On Negotiation and Limiting Accidental Complexity"
datePublished: Tue Nov 28 2023 14:26:15 GMT+0000 (Coordinated Universal Time)
cuid: clpifl4cz000f09l681v7aj70
slug: software-architecture-note-on-negotiation-and-limiting-accidental-complexity
cover: https://cdn.hashnode.com/res/hashnode/image/stock/unsplash/Ty4nyrV4PJI/upload/f9062c0600e99289ae4890c13d547fd9.jpeg
tags: software-architecture

---

In this brief article, I will discuss two insights from the book [**Fundamentals of Software Architecture: An Engineering Approach**](https://www.goodreads.com/book/show/44144493-fundamentals-of-software-architecture) by Mark Richards and Neal Ford: the importance of negotiation in an architect's job and an effective communication approach. I will then offer a way on how these aspects can be incorporated into architectural processes.

### Negotiation and complexity in the architect role

Negotiation is an integral part of a software architect's role, as they will be challenged by developers who may possess greater technical knowledge of the proposed solution, by other architects within the organisation who may question their ideas or approach to the problem, and by business stakeholders who will demand justification for the cost and time invested. While this may seem logical when read, seeing it explicitly stated provides comfort in knowing that disagreements are normal and that working together often requires adjusting ideas along the way.

Another challenge as an architect, and for a developer as well, is to avoid accidental complexity ("we made a problem hard") and focus on the essential complexity ("we have a hard problem"). An effective way to avoid accidental complexity is to use what the book calls the '4C's: **C**lear and **C**oncise **C**ommunication, and **C**ollaborating with all stakeholders are essential for a software architect to limit the chances of accidentally making things harder.

While these leadership traits are important for the *role* of an architect, they can also be incorporated into the architectural *process*.

### Negotiation and complexity in the architecture process

One way to achieve the 4C's and facilitate negotiation, which I have implemented as an improvement over other processes (e.g. an 'Architecture Board') not mentioned in the book but extensively described in the article ["Scaling the Practice of Architecture, Conversationally"](https://martinfowler.com/articles/scaling-architecture-conversationally.html), is by introducing an *Architecture Forum*. Such a forum leads to artefacts in the form of principles, lightweight Architecture Decision Records (ADR), and a 'Tech Radar'. The Forum helps share new ideas and decide the partners to collaborate with on different topics. People involved will then write down their choices clearly and concisely, aligning with the intent of ADRs.

Negotiation happens while documenting the decisions. Collaboration on written text leaves less room for interpretation than a meeting does and you are directly working on what will become the final result. (Of course, meetings can still support converging on the final decision.) The records of these decisions will then help others, and ourselves, in the future remember the often-forgotten aspect of how things are: the why.

---

A lot more can be written on the topic of an Architecture Forum and its artefacts, especially ADRs, but I leave it at this for now. If you would like to know more, do not hesitate to show interest and I may go deeper in the next post.