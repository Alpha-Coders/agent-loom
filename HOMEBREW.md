# AgentLoom Homebrew Tap

This repository serves as a Homebrew tap. The cask formula is located in `/Casks/agentloom.rb`.

## Installation

```bash
# Add this tap (only needed once)
brew tap Alpha-Coders/agentloom https://github.com/Alpha-Coders/agent-loom.git

# Install AgentLoom (--no-quarantine required for unsigned apps)
brew install --cask --no-quarantine agentloom
```

## Upgrade

```bash
brew upgrade --cask agentloom
```

## Uninstall

```bash
brew uninstall --cask agentloom
```

## Why `--no-quarantine`?

AgentLoom is an open-source app that is not signed with an Apple Developer certificate ($99/year). When macOS downloads apps from the internet, it adds a quarantine flag that triggers Gatekeeper security checks.

The `--no-quarantine` flag tells Homebrew to remove this quarantine attribute, allowing the app to run without requiring manual security bypass in System Settings.

This is the standard approach for distributing open-source macOS applications via Homebrew.

## Formula Updates

The cask formula is automatically updated by the GitHub Actions release workflow when a new version is tagged.
