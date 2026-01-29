# The Ralph Wiggum Technique (Full Content)

Source: https://github.com/ghuntley/how-to-ralph-wiggum
Author: Geoff Huntley

---

## Overview

The Ralph Wiggum Technique represents an automated software development approach leveraging AI agents to reduce development costs significantly. Named humorously after the character, it emphasizes letting AI agents work autonomously within structured parameters.

---

## Core Architecture

### Three-Phase Workflow

The methodology operates through a funnel system:

**Phase 1 - Define Requirements** (LLM conversation)
- Discuss project ideas and identify Jobs to Be Done (JTBD)
- Break individual JTBD into specific topics of concern
- Use subagents to load information from URLs into context
- Generate specification files for each topic

**Phase 2 & 3 - Run Ralph Loop** (Two modes via prompt swapping)
- **Planning Mode**: Gap analysis, generates/updates implementation plans
- **Building Mode**: Implements from plan, commits changes, updates plan

---

## Loop Mechanics

### Outer Loop (Bash Script)

```bash
while :; do cat PROMPT.md | claude ; done
```

**Key features:**
- Simple infinite loop that restarts agent with fresh context
- IMPLEMENTATION_PLAN.md persists between iterations as shared state
- Each iteration represents one complete task with fresh 200K token context
- Iterations are isolated; context clears automatically

### Inner Loop (Task Execution)
- Scope discipline via prompt instructions ("one task" requirement)
- Backpressure through tests/builds that force corrections
- Natural completion after successful commit

---

## Key Principles

### Context Optimization

The technique recognizes that "approximately 176K tokens are truly usable" from advertised context windows, with only "40-60% in the smart zone."

Single tasks per loop iteration maximize this zone utilization to nearly 100%.

### Steering Ralph: Patterns + Backpressure

Two directional control mechanisms:

**Upstream Steering:**
- Deterministic setup with consistent files loaded each iteration
- Existing code patterns guide AI-generated code
- Add utilities and patterns to steer toward correct implementations

**Downstream Steering:**
- Tests, typechecks, lints create rejection mechanisms
- AGENTS.md specifies project-specific validation commands
- Can extend to subjective criteria via "LLM-as-judge" validation

### Let Ralph Ralph

The approach embraces:
- LLM self-identification and self-correction abilities
- Iterative convergence toward solutions
- Eventual consistency through looping
- Minimal human intervention during execution

### Security & Protection

Critical emphasis on sandboxing:
- `--dangerously-skip-permissions` flag enables autonomous operation
- Requires isolated environments with minimal viable access
- Only necessary API keys and deploy keys
- Docker, Fly Sprites, or E2B for containment
- Escape hatches: Ctrl+C stops loops; git reset reverts changes

---

## File Structure

```
project-root/
├── loop.sh                  # Ralph loop orchestration script
├── PROMPT_build.md          # Building mode instructions
├── PROMPT_plan.md           # Planning mode instructions
├── AGENTS.md                # Operational guide (loaded each iteration)
├── IMPLEMENTATION_PLAN.md   # Prioritized task list
├── specs/                   # Requirement specs (one per JTBD topic)
│   ├── [topic-a].md
│   └── [topic-b].md
├── src/                     # Application source code
└── src/lib/                 # Shared utilities & components
```

---

## Essential Concepts

| Term | Definition |
|------|------------|
| **Job to Be Done (JTBD)** | High-level user need or outcome |
| **Topic of Concern** | Distinct aspect/component within a JTBD |
| **Spec** | Requirements document for one topic of concern |
| **Task** | Unit of work derived from comparing specs to code |

**Topic Scope Test:** "One Sentence Without 'And'" — if describing a topic requires conjunction, it's likely multiple topics.

---

## Operational Guidelines

### The Plan is Disposable
- Regenerate when Ralph diverges from intended path
- Regenerate when plan feels stale or unclear
- Cheap cost (one planning loop) versus wasted effort

### Move Outside the Loop

The human operator's role shifts from implementation to:
- Engineering setup and environment
- Observing patterns and failures
- Tuning prompts reactively
- Adjusting signs and guardrails

### Key Language Patterns

Geoff's specific phrasing matters:
- "study" (not "read")
- "don't assume not implemented"
- "parallel subagents"
- "only 1 subagent for build/tests"
- "capture the why"
- "if functionality is missing then it's your job to add it"

---

## Enhanced Loop Example

The script supports mode selection and iteration limits:

```bash
./loop.sh              # Build mode, unlimited
./loop.sh 20           # Build mode, max 20 iterations
./loop.sh plan         # Plan mode, unlimited
./loop.sh plan 5       # Plan mode, max 5 iterations
```

Claude CLI flags used:
- `-p`: Headless mode
- `--dangerously-skip-permissions`: Auto-approve tool calls
- `--output-format=stream-json`: Structured output
- `--model opus`: Complex reasoning agent
- `--verbose`: Detailed logging

---

## Implementation Best Practices

1. **Context Loading**: Each iteration deterministically loads PROMPT.md + AGENTS.md + specs/*
2. **Backpressure Control**: 1 subagent for validation; up to 500 for searches/reads
3. **Documentation**: Update AGENTS.md only with operational learnings; keep it brief
4. **Plan Updates**: Maintain IMPLEMENTATION_PLAN.md as single source of truth
5. **Git Hygiene**: Commit after tests pass; one task per commit
6. **Spec Consistency**: Use Opus with "ultrathink" for spec conflicts
7. **Issue Resolution**: Document or resolve all discovered bugs immediately

---

## Steering Techniques

### Non-Deterministic Backpressure

For subjective criteria (aesthetics, UX feel), LLM-as-judge tests provide binary pass/fail validation aligned with specifications.

### Discovery-Based Learning

When Ralph discovers issues, immediately update IMPLEMENTATION_PLAN.md to prevent future duplication.

### Operational Recording

AGENTS.md captures how to correctly run the application based on discovered patterns.

---

## Prompt Structure

### PROMPT_plan.md Template

```markdown
# Planning Mode

## Context
Load and study:
- AGENTS.md (operational guidelines)
- specs/* (all specification files)
- IMPLEMENTATION_PLAN.md (current state)

## Task
Perform gap analysis between specifications and current implementation.
Update IMPLEMENTATION_PLAN.md with prioritized tasks.

## Constraints
- Study existing code before proposing changes
- Don't assume features are not implemented
- One planning session per iteration
- Capture the WHY for each task
```

### PROMPT_build.md Template

```markdown
# Building Mode

## Context
Load and study:
- AGENTS.md (operational guidelines)
- IMPLEMENTATION_PLAN.md (task list)
- Relevant source files

## Task
Complete ONE task from the implementation plan.
Run tests after changes.
Commit on success.
Update plan to mark task complete.

## Constraints
- Only 1 task per iteration
- Only 1 subagent for build/test validation
- If functionality is missing, it's your job to add it
- Parallel subagents allowed for file searches/reads
```

---

## AGENTS.md Template

```markdown
# Project: [Name]

## Build & Test Commands
- Build: `[command]`
- Test: `[command]`
- Lint: `[command]`

## Project Structure
[Brief directory overview]

## Key Patterns
[Patterns the agent should follow]

## Operational Notes
[Learnings discovered during development]

## Boundaries
### Always Safe
- [safe actions]

### Ask First
- [actions needing approval]

### Never Do
- [forbidden actions]
```

---

## Troubleshooting

### Ralph Keeps Failing the Same Task
- Check if spec is unclear or contradictory
- Verify backpressure (tests) are correctly configured
- Regenerate the plan

### Ralph is Stuck in a Loop
- Ctrl+C to stop
- Review IMPLEMENTATION_PLAN.md for circular dependencies
- Simplify the current task

### Quality Issues
- Add more specific tests as backpressure
- Update AGENTS.md with patterns to follow
- Consider LLM-as-judge for subjective criteria

### Ralph Goes Off Track
- Stop and regenerate plan
- Update specs with clearer requirements
- Add guardrails to AGENTS.md

---

## Key Insights

1. **Context is King**: Fresh context each iteration prevents accumulation of errors
2. **Plans are Cheap**: Regenerate rather than fight a bad plan
3. **Tests are Steering**: Backpressure shapes behavior more than instructions
4. **Patterns Compound**: Good utilities guide future code generation
5. **Human Role Shifts**: From coding to environment engineering
6. **Eventual Consistency**: Looping converges on solutions
7. **Isolation is Essential**: Sandbox everything for safety
