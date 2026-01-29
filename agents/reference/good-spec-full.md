# How to Write a Good Spec for AI Agents (Full Content)

Source: https://addyosmani.com/blog/good-spec/
Author: Addy Osmani

---

## Overview

This guide addresses a common developer frustration: simply providing massive specifications to AI agents doesn't work due to context window limitations and attention constraints. Instead, developers need to craft "smart specs"—documents that guide agents clearly while remaining within practical context sizes and evolving throughout projects.

---

## Five Core Principles

### 1. Start with High-Level Vision, Let AI Expand Details

Begin projects with concise goal statements and core requirements, treating this as a product brief. Allow the agent to generate elaborate specifications from this foundation. This leverages AI strengths in elaboration while maintaining directional control.

**Key techniques:**
- Use Plan Mode (read-only operations) to refine specifications before code generation
- Have agents explore existing codebases and draft detailed plans
- Ask agents to clarify ambiguities and review for architectural risks
- Save specs as persistent references (e.g., SPEC.md files)

The spec becomes a shared artifact between human and AI, remaining the "source of truth" that guides all subsequent work.

### 2. Structure Specs Like Professional Requirements Documents

Treat AI specs as formal Product Requirements Documents or System Requirements Specifications. GitHub's analysis of 2,500+ agent configuration files revealed the most effective specs address six core areas:

**Six Core Areas:**

1. **Commands** - Full executable commands with flags (e.g., `npm test`, `pytest -v`)
2. **Testing** - Testing frameworks, file locations, coverage expectations
3. **Project Structure** - Explicit directory organization for source, tests, docs
4. **Code Style** - Real examples surpass lengthy descriptions
5. **Git Workflow** - Branch naming, commit formats, PR requirements
6. **Boundaries** - What agents should never touch (secrets, vendor directories, production configs)

**Additional recommendations:**
- Specify exact stack versions and dependencies
- Use consistent formatting (Markdown, XML-like tags)
- Treat specs as executable artifacts tied to version control
- Integrate specs into CI/CD pipelines

### 3. Break Tasks Into Modular Prompts, Not Monolithic Ones

Research confirms that excessive instructions cause performance degradation—the "curse of instructions." As requirements accumulate, model adherence to each decreases significantly.

**Strategies:**
- Divide specs into phases or components (backend vs. frontend sections)
- Create extended table of contents with summaries for large specs
- Use subagents or "skills" for different expertise domains
- Employ parallel agents for non-overlapping work
- Focus each prompt on one focused task/section
- Refresh context for major task transitions

This modular approach prevents context overload while maintaining quality and manageable error rates.

### 4. Build In Self-Checks, Constraints, and Human Expertise

Effective specs anticipate where agents might fail and establish guardrails. Incorporate domain knowledge and edge cases to prevent agents from operating in a vacuum.

**Three-tier boundary system:**
- **Always do** - Actions agents should take without asking
- **Ask first** - Actions requiring human approval (schema changes, new dependencies)
- **Never do** - Hard stops (no secrets, no vendor directories)

**Quality control measures:**
- Encourage self-verification against specs
- Use "LLM-as-a-Judge" for subjective evaluation
- Implement conformance testing
- Incorporate test plans within specs
- Inject domain expertise and common pitfalls

### 5. Test, Iterate, and Evolve Specs Continuously

Spec-writing and agent work form iterative cycles. Continuously verify outputs against specifications and refine accordingly.

**Best practices:**
- Test after each milestone, not just at completion
- Update specs when discovering new requirements or misunderstandings
- Use context management tools (RAG, MCP protocols)
- Maintain Git history of spec evolution
- Monitor and log agent actions
- Parallelize carefully with clear task separation
- Select appropriate models for different tasks

---

## Common Pitfalls to Avoid

- **Vague prompts** - Specificity matters; clearly define inputs, outputs, and constraints
- **Oversized contexts** - Hierarchical summaries prevent context bloat
- **Skipped reviews** - Always examine critical code paths despite passing tests
- **Conflating rapid prototyping with production engineering** - Different modes require different rigor
- **Ignoring the "lethal trifecta"** - Speed, non-determinism, and cost create dangerous combinations
- **Missing core spec areas** - Use the six-area checklist as verification

---

## Key Takeaway

Effective AI agent specifications require balancing clarity with conciseness. The best outcomes emerge from treating specs as living documents that guide agents through structured stages: specification, planning, task breakdown, and implementation. This staged approach—combined with human oversight and continuous refinement—produces reliable, maintainable AI-assisted code.

---

## Six Core Areas Checklist

Use this checklist to verify spec completeness:

| Area | Covered? | Notes |
|------|----------|-------|
| Commands | | Full executable commands with flags |
| Testing | | Frameworks, locations, coverage |
| Project Structure | | Directory organization |
| Code Style | | Examples, not descriptions |
| Git Workflow | | Branches, commits, PRs |
| Boundaries | | Always/Ask/Never tiers |

---

## Three-Tier Boundary Template

```markdown
## Boundaries

### Always Safe
- [Actions agents can take without asking]

### Ask First
- [Actions requiring human approval]

### Never Do
- [Hard stops - forbidden actions]
```
