---
id: ADR-0005
title: The surface computes nothing, and the shell is a different crate
status: accepted
date: 2026-09-06
register-entry: 16
claim-impact: none
guard: tools check-surface — no binary arithmetic punct in `surface`, tokenised; `shell` cannot name `world` (ADR-0003)
supersedes: none
cost: a reader needed by the interface must be named in `surface` before the interface can show it
alternatives-rejected: one crate with an exemption for layout arithmetic; a lint with an allow-list of files
re-derivations: §4's layer table gains `shell`
---

## Decision

`surface` holds named readers and the view model, and **contains no binary arithmetic operator**.
`shell` is a separate crate: it is the user interface, it may compute, and it **cannot name `world`**.

## Why

§4.4's rule — *every displayed number comes from a named reader* — is worth having because a quantity
computed twice is two implementations of one rule, and the first place anyone looks when they disagree.

**Without the split the rule dies in week two.** A user interface must compute: a label needs `width - 8`,
a chart needs a scale, a list needs an offset. One crate holding both means the first such line is either
a violation or an exemption, and §17 says the exemption list has none. Two crates means the line is
legal where it belongs and impossible where it does not.

**The check tokenises rather than greps**, so a `+` inside a comment, a doc comment or a string is not a
finding. The first draft of its sibling `check-lints` substring-matched and reported *itself* on its first
run, because its own needle appeared in its own source; both checks now use the lexer, and both have a
negative fixture proving they still fire on a real violation.

*Known limit, stated:* the check cannot distinguish a unary minus on a literal from a subtraction, and
does not try — `surface` has no reason to write either.
