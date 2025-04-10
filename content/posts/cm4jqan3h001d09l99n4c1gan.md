---
title: "Ground Your ADRs with a Verification Section"
datePublished: Wed Dec 11 2024 10:09:23 GMT+0000 (Coordinated Universal Time)
cuid: cm4jqan3h001d09l99n4c1gan
slug: ground-your-adrs-with-a-verification-section
tags: architecture, decision-making, adr

---

Making architectural decisions is one thing, but have you ever wondered how to make them more effective? Adding a Verification section to Architecture Decision Records (ADRs) can make the difference. This simple addition bridges the gap between theory and practice, making decisions actionable and measurable.

If you're new to ADRs, check out [my post on their benefits](https://hans.lhoest.eu/less-mentioned-benefits-of-architecture-decision-records) first.

## Common ADR Challenges

Architecture Decision Records (ADRs) are a valuable tool for capturing architectural choices. However, they can suffer from the following weaknesses:

1. Decisions can be made, or become, detached from implementation
    
2. Intent and success criteria remain vague or unmeasurable
    

These challenges can lead to:

* Architectural decisions that exist only on paper
    
* Difficulty in assessing whether decisions are being followed
    
* Missed opportunities for feedback and improvement
    

## Introducing Verification

Adding a 'Verification' section to ADRs helps ground these decisions in reality and improve their quality. This section details how decisions will be evaluated or implemented, creating a clear path from decision to implementation.

Writing an ADR is often only the beginning, not the end. Establishing feedback loops helps to crystallise our decisions in actual implementation.

### Grounding Architectural Decisions in Reality

It is a good idea to state goals in measurable and verifiable ways, and this applies equally to architecture and engineering. This is similar to writing requirements in parallel with the tests for these requirements. Or how writing code can put an analysis to the test. Each of these practices grounds our work in reality and provides feedback on our intentions.

Architecture decisions benefit from a similar grounding. Just as business analysts and developers can collaborate through behavioural tests, architects and developers can collaborate through verification criteria. More broadly, regardless of roles, the architecture function benefits from verification criteria. This creates feedback loops that improve quality.

Importantly, verification works in both directions. Architecture and implementation connect and as such developers engage with the architecture and in return the architect receives feedback on what is working and what is not. When decisions aren't being acted upon, it becomes visible, which is a sign to investigate why and adapt.

A key benefit of keeping verification in mind: ADRs become more actionable and effective.

## Verification Approaches

You might think this adds substantial work to your process. Not necessarily so. Verification methods can range from simple to sophisticated. Different architectural decisions require different verification approaches. Or you may, to get started, decide to go for a very light-weight approach.

Choose an approach that matches your context and needs. Here are some options:

1. Ask questions: the simplest approach uses straightforward questions for self-assessment.
    
2. Automated tests / fitness functions: integrate checks in your development pipeline, look for ways to assess architectural characteristics objectively.
    
3. Define metrics: what can you measure to determine if decisions are effective?
    
4. Architecture Hoisting: the most comprehensive and strict approach, as described by George Fairbanks:
    

> "Architecture hoisting is a stricter kind of architecture-focused design. When following an architecture hoisting approach, developers design the architecture with the intent of guaranteeing a goal or property of the system. Guarantees are difficult to come by in any kind of software design, but architecture hoisting strives to guarantee a goal or property through architecture choices. The idea is that once a goal or property has been hoisted into the architecture, developers should not need to write any additional code to achieve it."

Let's see some examples.

## Examples

### Simple

Suppose the recorded decision is to put all code in a mono-repo. Then the verification step could be as simple as:

```json
## Verification

Is your service in our mono-repo <include name>?
```

This looks too easy right? In some cases it simply can be. Especially if your ADRs apply to multiple teams, and verification is not handled centrally, your verification tends to be more questions or check-based than actual steering towards an implementation.

### Self-Assessment

For another example, suppose your cross-team decision is to adopt Continuous Delivery:

```json
## Verification

- Is the CI pipeline automated?
- Can you deploy to production without manual steps?
- If not, what manual gates are in place?
- Are feature flags used for in-progress work?
- Are branches short-lived?
```

Note the self-assessment tone. This style can be adopted when architecture is an enabling and supporting function.

### Automated

An example requiring effort but providing more guarantees to compliance:

Suppose we are all working in the previously mentioned mono-repo and discover that teams are not respecting service boundaries. We decide to establish naming conventions to be able to enforce no inappropriate, accidental dependencies are created. We could again simply ask a question in the Verification section if naming conventions are followed, but automating this checking is relatively easy, so we decide to automate this check.

Here we make compliance with our ADR automatic by hoisting the decision in our way of working.

The verification section in the initial ADR may then look like this:

```json
## Verification

A dependency checker will be written and integrated in our build tooling for each service.

- Are all of your services' build tooling integrated with the dependency checker?
```

**Roles and organizational structures can heavily influence the verification section.** If you have, say, a Developer Experience Team they may likely work with the teams and makes sure this gets integrated in their CI-pipeline. Other times seniors in the teams may take this upon them. Ideally these responsibilities are made clear in the Verification section as well.

More examples of self-assessment questions can be found at [John Lewis' Software Engineering Principles Self-Assessment](https://engineering-principles.jlp.engineering/self-assessment/).

## Implementing the Verification section

`Verification` is my recommended term because of this definition:

> \[!quote\] "VERIFY implies the establishing of correspondence of actual facts or details with those proposed or guessed at." (Merriam-Webster)

However, depending on your environment a different name could be a pragmatic choice. `Compliance` often resonates better in regulated environments. I have used 'Validation' before. While regularly validating or evaluating our decisions is good practice, here we are aiming to put a system in place to check if our decisions are implemented.

Here is an example of a `Verification` section you could include in your ADR template:

```json
## Verification

How will we ensure compliance with this decision?

Consider:

- Questions for self-assessment
- Specific metrics
- Verifications in the form of tests or fitness functions
- Implementation guidance
- Making it easy to adopt
```

Based on this, a more extended example for a Continuous Delivery ADR:

```json
## Verification

Questions:
- Is the CI pipeline automated?
- Can you deploy to production without manual steps?
- Are feature flags used for in-progress work?

Metrics:
- Deployment frequency (target: daily)
- Lead time for changes (target: < 1 day)
- Change failure rate (target: < 15%)

Implementation:
- Teams must implement automated deployment, Delivery Platform Team will assist
- Regular metrics reporting (data can be collected manually)
```

## Considerations

Won't adding this section complicate writing these ADRs?

Well yes, there is another section to write and you need to think about how to make your ADRs more effective. But I hope by now I have shown you this does not need to be complex. Making them more effective and actionable is something you would want regardless, so the thinking part about this should not be skipped. Instead, incorporate verification thinking while writing the ADR: state decisions in measurable and verifiable ways.

Now if you ask me, is it better to have a record of *a* decision without this section than not having one at all? Then my answer would be yes. Don't let perfection stop you from adopting or recording your decisions - use verification to improve them over time.

## Making It Work in Practice

Here are some tips that helped me in the past and may inspire you:

### Build Assessment Lists

If you're generating documentation from your ADRs in markdown (perhaps as a static site), consider extracting the verification sections to create a comprehensive assessment list. This provides teams with a single reference point for alignment checking, rather than requiring them to wade through individual ADRs.

### Conduct Regular Reviews

Conduct annual team reviews using the verification sections:

* Hold two-way discussions to identify outdated or unfit decisions
    
* Create prioritised action lists from review findings
    
* Plan top-priorities with stakeholders as needed
    

This is often also a good time to evaluate the ADR process itself:

* What's working well?
    
* What needs improvement?
    
* How can we enhance the process?
    

## Key Takeaways

Architecture benefits from being grounded in implementation. Adding a Verification section to your ADRs creates feedback loops and improves their quality.

The verification section serves as more than a checklist. It:

* Connects architecture with implementation
    
* Provides a mechanism for feedback in both directions
    
* Reveals when decisions aren't being acted upon
    
* Makes ADRs more actionable and effective
    

This grounding helps prevent architectural decisions from remaining theoretical or becoming shelfware.