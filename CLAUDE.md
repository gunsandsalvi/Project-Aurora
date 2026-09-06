# Working agreements

## Workflow orchestration

- **Maximum 2 agents per workflow.** Never author a workflow script that spawns more than
  two `agent()` calls in total.
- **Launch many workflows at once instead.** Where work would naturally be a single large
  fan-out, split it into several independent 2-agent workflows and launch them in the same
  message so they run concurrently.
- Chain rounds across turns: launch a round of small workflows, wait for them to land, write
  their results to the scratchpad, then launch the next round pointed at those files (agents
  can read files; workflow scripts cannot).

## Working the plan

- **A milestone file lists remaining work, never completed work.** When an item is done it is
  **deleted** in the same commit that completes it. Git history is the record; the file is the
  state. A milestone is finished when its work-item tables are empty and its exit criteria pass.
- **A defect found mid-flight is never fixed opportunistically.** It is written up as a new
  numbered step and placed in the milestone that owns the code it affects, then resolved there.
  Three buckets, and the classification is explicit:
  - **blocks this milestone's exit** → a new step in the current milestone, at the right position
    in its dependency order;
  - **belongs to code a later milestone owns** → a new step in that milestone;
  - **is a defect in the specification rather than the build** → a row in `IMPLEMENTATION.md`'s
    open decisions register, assigned to the milestone that must settle it.
- **Every removal and every insertion is a diff a reviewer sees.** This is Appendix A's
  supersession rule applied to work rather than to values: work that disappeared without being
  named is the same failure as a value that changed without being named.
