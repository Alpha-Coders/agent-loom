# Awesome Agentic Patterns (Full Content)

Source: https://github.com/nibzard/awesome-agentic-patterns
Repository: nibzard/awesome-agentic-patterns

---

## Purpose

This repository catalogs repeatable, production-ready patterns for building autonomous AI agents. It surfaces "the repeatable patterns that bridge the gap" between tutorials and real-world deployment.

---

## Pattern Criteria

Submissions must be:
- **Repeatable** - Adopted by multiple teams
- **Agent-centric** - Improving how agents sense, reason, or act
- **Traceable** - Backed by public references: blogs, talks, repos, papers

---

## Eight Core Categories (130+ Patterns)

### 1. Context & Memory (13 patterns)

Focus on efficient context handling—sliding-window curation, vector caching, episodic memory retrieval, and state externalization techniques.

**Patterns:**
- Sliding Window Context Management
- Vector Cache for Repeated Queries
- Episodic Memory Retrieval
- State Externalization
- Context Compression
- Hierarchical Context Summarization
- Working Memory vs Long-Term Memory Separation
- Context Priority Scoring
- Lazy Context Loading
- Context Checkpointing
- Semantic Context Clustering
- Context Window Anxiety Management
- Progressive Disclosure for Large Files

### 2. Feedback Loops (13 patterns)

Emphasize learning mechanisms: CI integration, self-critique loops, reflection, and reward shaping to improve agent decisions.

**Patterns:**
- Reflection Loop (Generate-Evaluate-Refine)
- CI/CD Integration Feedback
- Self-Critique Loop
- Reward Shaping
- Human Feedback Integration
- Test-Driven Agent Development
- Error Pattern Learning
- Success Trajectory Replay
- A/B Testing for Agent Behaviors
- Specification-as-Test
- Graph of Thoughts (GoT)
- Rich Feedback Over Perfect Prompts
- Coding Agent CI Feedback Loop

### 3. Learning & Adaptation (5 patterns)

Cover Agent Reinforcement Fine-Tuning (RFT), skill library evolution, and variance-based sample selection.

**Patterns:**
- Agent Reinforcement Fine-Tuning (Agent RFT)
- Skill Library Evolution
- Variance-Based Sample Selection
- Compounding Engineering Pattern
- Memory-Based Reinforcement Learning (MemRL)

### 4. Orchestration & Control (34 patterns)

Largest category: task decomposition, multi-agent debate, plan-then-execute, tool routing, and sub-agent spawning.

**Patterns:**
- Plan-Then-Execute
- Inversion of Control
- Task Decomposition
- Multi-Agent Debate
- Tool Routing
- Sub-Agent Spawning
- Swarm Migration
- Language Agent Tree Search (LATS)
- Tree of Thoughts
- ReACT (Reasoning + Acting)
- Reflexion
- Chain of Thought
- Hierarchical Task Networks
- Goal-Directed Planning
- Capability Compartmentalization
- Dynamic Tool Selection
- Parallel Execution Coordination
- Sequential Pipeline Orchestration
- Event-Driven Agent Triggers
- State Machine Agent Control
- Workflow DAG Execution
- Priority-Based Task Queuing
- Resource-Aware Scheduling
- Timeout and Retry Policies
- Graceful Degradation
- Circuit Breaker Pattern
- Backoff Strategies
- Idempotent Operations
- Transaction-Like Rollbacks
- Checkpoint-Resume Execution
- Conditional Branching
- Loop Detection and Prevention
- Deadlock Avoidance
- Oracle/Worker Pattern

### 5. Reliability & Eval (14 patterns)

Address guardrails, evaluation harnesses, structured outputs, and anti-reward-hacking measures.

**Patterns:**
- Guardrail Implementation
- Evaluation Harness Design
- Structured Output Validation
- Anti-Reward-Hacking Grader Design
- Schema Validation
- Action Caching/Replay
- Deterministic Test Fixtures
- Regression Testing for Agents
- Benchmark Suite Maintenance
- A/B Evaluation Framework
- Human Evaluation Integration
- Confidence Scoring
- Uncertainty Quantification
- Extended Coherence Work Sessions

### 6. Security & Safety (3 patterns)

Cover isolated VMs, PII tokenization, and deterministic security scanning.

**Patterns:**
- Isolated VM per RL Rollout
- PII Tokenization
- Deterministic Security Scanning

### 7. Tool Use & Environment (23 patterns)

Address agent-tool interaction: CLI-first design, code-over-API, web search loops, and multimodal integration.

**Patterns:**
- CLI-First Tool Design
- Code-Over-API
- Web Search Integration
- Multimodal Input Processing
- File System Interaction
- Database Query Tools
- API Client Generation
- Browser Automation
- Progressive Tool Discovery
- Tool Capability Description
- Tool Error Handling
- Tool Rate Limiting
- Tool Authentication Management
- Tool Version Compatibility
- Tool Dependency Resolution
- Environment Isolation
- Sandbox Execution
- Resource Quota Management
- Output Format Standardization
- Tool Composition
- LLM-Friendly API Design
- Egress Lockdown
- Tool Usage Analytics

### 8. UX & Collaboration (13 patterns)

Human-in-the-loop frameworks, background agents, chain-of-thought transparency, and blended initiative models.

**Patterns:**
- Human-in-the-Loop Approval
- Background-to-Foreground Handoff
- Chain-of-Thought Transparency
- Blended Initiative Models
- Spectrum of Control
- Confidence-Based Escalation
- User Preference Learning
- Feedback Collection UI
- Progress Visualization
- Explainable Decisions
- Undo/Redo Support
- Draft Review Workflow
- Abstracted Code Representation for Review

---

## Pattern Template

When contributing new patterns, use this structure:

```markdown
# Pattern Name

## Category
[One of the 8 categories]

## Problem
[What challenge does this pattern address?]

## Solution
[How does the pattern solve the problem?]

## Implementation
[Code examples, architecture diagrams, or step-by-step guides]

## Trade-offs
[Pros and cons of this approach]

## Related Patterns
[Links to complementary or alternative patterns]

## References
[Links to source material, papers, or implementations]

## Maturity Level
[Proposed | Emerging | Established | Validated-in-Production | Best-Practice]
```

---

## Contribution Process

1. Fork the repository
2. Add pattern files to `patterns/` directory
3. Follow the pattern template
4. Open a pull request
5. README and site regenerate automatically

---

## Notable Context

The project was inspired by a 2025 writeup on "What Sourcegraph learned building AI coding agents," emphasizing production-grade lessons over toy examples.

Key contributors include work from:
- Anthropic
- Sourcegraph
- Cognition AI
- Individual researchers (Will Larson, Simon Willison)

Evolution: Grew from 0 to 113+ patterns during 2025-2026 as production teams shared learnings openly.

---

## Pattern Quick Reference by Use Case

### Building Your First Agent
- Inversion of Control
- Reflection Loop
- Chain-of-Thought Transparency

### Improving Reliability
- Plan-Then-Execute
- Guardrail Implementation
- Schema Validation

### Scaling Operations
- Swarm Migration
- Oracle/Worker Pattern
- Parallel Execution Coordination

### Security Hardening
- PII Tokenization
- Isolated VM per RL Rollout
- Egress Lockdown

### Better Human Collaboration
- Spectrum of Control
- Human-in-the-Loop Approval
- Abstracted Code Representation for Review

### Long-Running Agents
- Skill Library Evolution
- Episodic Memory Retrieval
- Checkpoint-Resume Execution
