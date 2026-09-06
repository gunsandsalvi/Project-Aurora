---
id: ADR-0009
title: The instrument row is 80 bytes with the schedule inline, and the schedule identity space does not exist
status: accepted
date: 2026-09-06
register-entry: 9
registers: PROJECT_AURORA.md §7.5, PROJECT_AURORA.md §5.2
claim-impact: A2
guard: aurora-tools sizing — both arms are summed from their field lists and priced against the census on every run, so a column added to either changes a published number; and the identifier census has no schedules row to sum
supersedes: §7.5's unresolved 44 B / 148 B choice and its deferral to Phase 2; §5.2's schedules identity space
cost: a schedule is a generating rule, not a list of dated rows, so an irregular schedule needs a rule that can express it or a second rule chained behind it
alternatives-rejected: 44 B with a schedule directory (8.5 MB dearer and 6.36 M identifiers for a thing nobody holds); 148 B inline (not a derived width — it is 80 B plus what an enumerated schedule costs, and an enumerated schedule is 2.0 GB); deferring to device measurement (the arm being measured is specified in §3.4.4, which does not exist)
re-derivations: §5.2's census drops from 15,732,835 to 9,372,835 ever issued; the directory from 60.0 MiB to 35.8 MiB; §7.5's family counts become usable for sizing
---

## Decision

**The instrument row is 80 bytes, with the schedule and both price epochs inline.** There is no
`scheduleFirst` column and **no schedule identity space**. A schedule is a generating rule — first
period, interval, count, amount, kind, rate basis — not a list of dated rows.

`optionsFirst` stays out of line: §7.6's seven typed option-terms tables are a different shape, held by
few instruments, and their absence is the common case.

## Why

**Neither published number is a derived width.** §7.5's eleven columns, at the widths the rest of the
specification forces, come to 33 B and pad to 40 — not 44. The inline row comes to 80 — not 148. The
68 bytes of difference are what an *enumerated* schedule costs, and 68 B buys five dated rows, which
cannot express a ten-year mortgage at weekly ticks. Enumerating properly is not a 148-byte row at all:
1,060,000 instruments × ~120 payments × 16 B is **2.0 GB**. So the 148 B arm was never a viable design;
it was a partial enumeration that would have run out.

**The comparison §7.5 defers could not have been made.** §7.5 says *"this table and §3.4.4 describe two
different rows"* and settles it "by measurement on the target device", as a Phase 2 entry criterion.
§3.4.4 is one of the eight dangling cross-references this milestone already recorded — **the arm being
measured is specified in a section that does not exist.** No device could have decided it.

**And memory is not what decides it.** Priced against §5.2's census:

| | rows | schedules | directory | total |
|---|---|---|---|---|
| **A** — 40 B row, schedule out of line | 42.4 MB | 25.4 MB | 25.4 MB | **93.3 MB** |
| **B** — 80 B row, schedule inline | 84.8 MB | — | — | **84.8 MB** |

8.5 MB apart, which is inside the error bar on the census's own owed mean lives. A choice this close is
not decided by the smaller number.

**What decides it is whether a schedule is a claim, and it is not.** Under A2 an instrument is data;
under R-1 a claim exists as its issuer's negative balance. **Nothing holds a schedule.** It has no
holder, no balance, and no counterparty — it is a field of the instrument describing when the
instrument pays. An identity space exists so that a thing can be *addressed*: by a holding, by a lien,
by a journal row, by a digest walk. A schedule is addressed by exactly one thing, the instrument that
owns it, which already has an identity.

§7.4's amendment mechanisms confirm it rather than contradict it. `prepayment` and `payment-holiday`
amend the *instrument* — ADR-0018 mints the handle per (mechanism, type), and the type is the
instrument's. Nothing amends a schedule without amending the instrument it belongs to.

So the 6,360,000 schedule identifiers were an identity space for something that is not a claim: **40%
of the entire identifier census**, sized, directoried, digest-walked and saved, for rows nobody can
address.

## What it costs

**A schedule must be expressible as a rule.** A level-payment loan, a bullet, an amortising ladder and
a floating coupon all are. An irregular schedule — a bespoke amortisation, a step-up with three
different steps — needs either a rule wide enough to carry it or a second rule chained behind the
first. If a type arrives that needs genuinely arbitrary dates, this decision is reopened, and the
`kind` field is where that shows up.

**The row doubles**, 40 B to 80 B, and 84.8 MB is a real line in N4. It buys back 25.4 MB of directory
and 25.4 MB of schedule rows, and removes an identity space from M1's schema entirely.

## Alternatives rejected

- **44 B with a schedule directory.** 8.5 MB dearer, and it is 40 B rather than 44 in any case.
- **148 B inline.** Not a width, a guess: 80 B of derived row plus five enumerated schedule rows.
- **Defer to device measurement, as §7.5 says.** The comparison names a section that does not exist,
  and the difference is inside the census's own error bar. A measurement cannot resolve a question
  whose two arms are not both specified.

## The guard

`aurora-tools sizing` sums both arms from their field lists and prices them against the census on every
run, so a column added to either moves a published number. And the census itself is the second guard:
after this decision it has **no schedules row**, so an identity space reappearing there is a diff.
