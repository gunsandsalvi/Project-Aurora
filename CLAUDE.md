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
