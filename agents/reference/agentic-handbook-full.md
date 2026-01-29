# The Agentic AI Handbook: Production-Ready Patterns (Full Content)

Source: https://www.nibzard.com/agentic-handbook/
Author: Nikola Balic
Published: January 15, 2026
License: CC BY-SA 4.0

---

## Overview

This handbook documents 113 battle-tested agentic patterns collected from real production systems, organized into 8 categories. It bridges the gap between agent demos and production-ready implementations.

---

## The Eight Pattern Categories

### 1. Orchestration & Control
**Focus:** How agents decide what to do, in what order, and when to stop.

**Key Patterns:**
- **Plan-Then-Execute** - Separate planning from execution to enhance security and dependability
- **Inversion of Control** - Supply agents with tools and objectives rather than step-by-step directives
- **Swarm Migration** - Coordinate 10+ parallel subagents for large-scale operations
- **Language Agent Tree Search (LATS)** - Apply Monte Carlo Tree Search to reasoning challenges
- **Tree of Thoughts** - Frame reasoning as exploratory tree search

### 2. Tool Use & Environment
**Focus:** How agents interact with external systems—APIs, databases, file systems, browsers.

**Key Patterns:**
- **Code-Over-API** - Generate and execute code instead of calling REST endpoints
- **Progressive Tool Discovery** - Introduce tools incrementally rather than overwhelming agents
- **Egress Lockdown** - Security approach preventing data exfiltration
- **LLM-Friendly API Design** - Create interfaces language models can effectively utilize

### 3. Context & Memory
**Focus:** Managing limited context windows while accumulating knowledge over time.

**Key Patterns:**
- **Context Window Anxiety Management** - Address models panicking about token constraints
- **Episodic Memory Retrieval** - Enable long-term memory spanning multiple sessions
- **Curated Code Context** - Selectively include only germane code in available context
- **Progressive Disclosure for Large Files** - Load file contents incrementally as needed

### 4. Feedback Loops
**Focus:** How agents improve outputs through iteration and assessment.

**Key Patterns:**
- **Reflection Loop** - Generate, evaluate, refine until reaching quality benchmarks
- **Rich Feedback Loops Over Perfect Prompts** - Prefer iteration to obsessing over initial instructions
- **Coding Agent CI Feedback Loop** - Use test failures as educational signals
- **Graph of Thoughts (GoT)** - Structure reasoning as interconnected thought graphs

### 5. UX & Collaboration
**Focus:** Effective human-agent partnership and coordination.

**Key Patterns:**
- **Chain-of-Thought Monitoring & Interruption** - Track reasoning in real-time with intervention capability
- **Spectrum of Control** - Fluidly transition between human and agent authority
- **Verbose Reasoning Transparency** - Expose agent thinking for trust and troubleshooting
- **Abstracted Code Representation for Review** - Provide high-level summaries instead of raw diffs

### 6. Reliability & Eval
**Focus:** Ensuring agents work correctly and consistently.

**Key Patterns:**
- **Lethal Trifecta Threat Model** - Security framework addressing three converging risks
- **Anti-Reward-Hacking Grader Design** - Prevent agents from manipulating evaluation metrics
- **Extended Coherence Work Sessions** - Maintain focus across lengthy interactions
- **Workflow Evals with Mocked Tools** - Test agent logic without actual tool execution

### 7. Learning & Adaptation
**Focus:** Agent improvement and institutional knowledge accumulation over time.

**Key Patterns:**
- **Skill Library Evolution** - Persist effective code as reusable, evolving capabilities
- **Agent Reinforcement Fine-Tuning (Agent RFT)** - Train on successful tool interactions (50-72% performance gains from 100-1000 trajectories)
- **Compounding Engineering Pattern** - Build upon previous agent achievements
- **Variance-Based RL** - Select training examples based on model uncertainty

### 8. Security & Safety
**Focus:** Preventing agent-caused harm and protecting sensitive operations.

**Key Patterns:**
- **PII Tokenization** - Protect privacy through sensitive data tokenization
- **Isolated VM per RL Rollout** - Sandboxing for reinforcement learning environments
- **Deterministic Security Scanning** - Automated security verification in workflow loops

---

## Four Foundational Patterns (Detailed)

### Plan-Then-Execute

**Challenge:** Tool outputs can redirect agents toward harmful actions when untrusted data appears mid-execution.

**Approach:** Divide reasoning into:
1. **Fixed Planning** - Agent generates tool sequence before seeing untrusted data
2. **Controlled Execution** - Outputs shape parameters but cannot alter which tools execute

**Results:** Demonstrates 2-3x success rate improvements for complex tasks.

**Applicability:** Email/calendar management, SQL assistance, code review support.

### Inversion of Control

**Challenge:** Micromanaging agent steps limits scalability and stifles agent creativity.

**Approach:** Provide tools plus high-level objectives, allowing agents to determine orchestration.

**Human involvement breakdown:**
- Initial 10%: Setup, constraints, goals
- Middle 87%: Agent autonomous operation
- Final 3%: Review, approval, corrections

**Outcome:** Eliminates humans as orchestration bottlenecks.

### Reflection Loop

**Challenge:** Single-pass generation produces suboptimal outputs lacking iterative refinement.

**Approach:**
1. Generate initial draft
2. Evaluate against metrics/criteria
3. Iteratively refine until quality thresholds reached

**Application Scope:** Writing, reasoning, and code generation where quality matters significantly.

### Chain-of-Thought Monitoring & Interruption

**Challenge:** Agents pursue flawed reasoning paths for extended periods before completing tasks.

**Approach:** Actively surveil intermediate reasoning with real-time intervention capability.

**Key Insight:** Monitor early decisions closely—if flawed, interrupt immediately.

**Wisdom:** "Have your finger on the trigger to escape and interrupt any bad behavior" (Tanner Jones, Vulcan).

---

## Multi-Agent System Architecture

### Swarm Migration Pattern

**Use Case:** Large-scale transformations (framework upgrades, lint rule rollouts, API migrations across many files).

**Process:**
1. Main agent develops comprehensive migration plan
2. Segments work into parallelizable chunks
3. Spawns 10+ concurrent subagents
4. Each subagent independently processes its assignments
5. Main agent consolidates and validates results

**Real Implementation:** Anthropic users report spending $1,000+ monthly on migrations through this approach, achieving 10x+ speedups versus sequential processing.

**Prerequisite:** Migrations must be atomic—individual files processed independently.

### Language Agent Tree Search (LATS)

**Application:** Complex reasoning requiring exploration of multiple solution pathways.

**Mechanism:** Combines Monte Carlo Tree Search with LLM self-reflection:
- Nodes represent states
- Edges represent reasoning actions
- Leaf evaluation uses model reflection

**Performance:** Outperforms ReACT, Reflexion, and Tree of Thoughts on intricate reasoning challenges.

**Best For:** Strategic planning, mathematical reasoning, multi-step problem solving where initial decisions significantly influence subsequent outcomes.

### Oracle/Worker Pattern

**Rationale:** Different models have distinct capabilities and cost profiles.

**Structure:**
- Expensive models (Opus) perform planning and review
- Economical models (Haiku) execute individual tasks

**Advantage:** Achieves high-end model intelligence at fractional cost since most work parallelizes to smaller models.

---

## Human-AI Collaboration Framework

### Spectrum of Control / Blended Initiative

**Principle:** Effective collaboration exists on a continuum, not as binary choice between human or agent control.

**Modes:**
- **Human-led** - Human directs; agent executes
- **Agent-led** - Agent proposes; human approves
- **Blended** - Dynamic back-and-forth based on context and confidence

**Implementation:** Agents explicitly signal confidence levels and control transfer boundaries.

### Chain-of-Thought Monitoring

**Core Benefit:** Visibility equals control.

**Outcomes:**
- Early detection of faulty assumptions
- Transparent decision understanding
- Pre-emptive resource-wasting prevention
- Trust building through transparency

### Abstracted Code Representation for Review

**Problem:** Raw diffs overwhelm reviewers when agents span multiple files.

**Solution:** Generate higher-level abstractions:
- Pseudocode summaries
- Intent descriptions
- Architectural rationales
- Behavior comparisons

**Advantage:** Review accelerates and improves focus on design rightness versus line-by-line scrutiny.

---

## Security Framework: The Lethal Trifecta

### Core Concept

Three converging capabilities create straightforward attack vectors:

1. **Private data access** (secrets, user information, internal systems)
2. **Untrusted content exposure** (user input, web content, emails)
3. **External communication ability** (API calls, webhooks, messaging)

### Risk Mitigation

Guarantee at least one circle remains absent in any execution pathway:

- **Remove network access** - Prevents exfiltration
- **Deny direct data access** - Blocks private data theft
- **Sanitize untrusted inputs** - Blocks hostile instructions

### Additional Security Patterns

**Compartmentalization:** Apply least-privilege principles; agents access only tools required for specific tasks.

**PII Tokenization:** Replace sensitive data with tokens before agent processing; downstream services resolve tokens during execution.

Benefits:
- Transparent reasoning
- Eliminated raw PII in context
- Compliance support
- Reversibility

---

## Production-Validated Patterns

### Context Window Anxiety

**Discovery:** Models like Claude Sonnet 4.5 demonstrate "context anxiety"—prematurely summarizing or closing tasks despite adequate remaining context.

**Symptoms:**
- Unexpected mid-task summarization
- Rushed decisions to "wrap up"
- Explicit limit mentions
- Incomplete work despite sufficient capacity

**Mitigation:**
1. Enable large windows (1M tokens) while capping usage at 200k—provides psychological "runway"
2. Counter-prompting: "Substantial context remains—do not rush"
3. Explicit token budget transparency

### Agent Reinforcement Fine-Tuning (Agent RFT)

**Breakthrough:** Training exclusively on real tool interactions proves sample-efficient.

**Results:** 50-72% performance improvements from just 100-1000 successful trajectories.

**Mechanism:**
1. Collect successful agent execution trajectories
2. Fine-tune models to imitate patterns
3. Models learn tool-use strategies, not just responses

**Key Insight:** Training on agent workflows—not input-output pairs—teaches strategy.

### Skill Library Evolution

**Problem:** Agents rediscover solutions without persistence, wasting tokens and time.

**Solution:** Persist working implementations as evolving reusable skills.

**Progression:**
1. Ad-hoc code solving immediate problems
2. Save working solutions to skills directory
3. Refactor for generalization (parameterize values)
4. Document thoroughly (purpose, parameters, returns, examples)
5. Agents discover and reuse in future sessions

**Optimization:** Progressive disclosure replacing all-skills loading with selective injection achieved 91% token reduction (26 tools at 17k tokens to 4 selected at 1.5k tokens).

**Strategic Value:** Agents build capability over time rather than starting fresh each session.

---

## Pattern Maturity Levels

| Level | Description |
|-------|-------------|
| **Proposed** | Suggested but not widely adopted |
| **Emerging** | Early adoption showing promise |
| **Established** | Widely used and understood |
| **Validated-in-Production** | Proven at real production scale |
| **Best-Practice** | Industry consensus on correctness |
| **Rapidly-Improving** | Field moving quickly; patterns evolve frequently |
| **Experimental-but-Awesome** | Unproven but too promising to ignore |

---

## Implementation Guide

### Phase 1: Select Three Patterns

**For first-time builders:**
- Inversion of Control (orchestration)
- Reflection Loop (quality)
- Chain-of-Thought Monitoring (visibility)

**For scaling existing agents:**
- Skill Library Evolution (persistence)
- Plan-Then-Execute (reliability)
- Spectrum of Control (collaboration)

**For security-focused implementation:**
- Lethal Trifecta Threat Model
- Compartmentalization
- PII Tokenization

### Phase 2: Implementation Cycle
1. Deploy pattern in your environment
2. Observe behavior with actual workload
3. Iterate based on learnings
4. Document findings

### Phase 3: Build Your Library
1. Identify recurring solutions
2. Extract core patterns
3. Document with examples
4. Contribute to community

### Phase 4: Maintain Currency
- Monitor repository updates
- Follow thought leaders sharing openly
- Experiment with emerging approaches
- Share discoveries

---

## Frontier Areas and Future Evolution

### Currently Underexplored
- Multi-modal agents beyond text/code
- Extended-duration autonomous agents (hours/days)
- Agent-to-agent communication protocols
- Resource allocation economic models
- Legal/compliance frameworks

### Next Evolution: Agent Learning
The major advancement involves agents fundamentally improving through experience—building institutional knowledge, sharing capabilities across instances, and advancing without manual fine-tuning.
