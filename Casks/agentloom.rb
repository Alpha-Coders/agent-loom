# typed: false
# frozen_string_literal: true

cask "agentloom" do
  arch arm: "arm64", intel: "x64"

  version "1.1.1"
  sha256 arm:   "90598cb7f623da41c2de4688e2b5d9f06cb1c395b901b500e316437b6c3d5d2f",
         intel: "953f655d495b28f1578385ce3b0dfe41bd6e2fa15eaab152e3a1c275939a5f40"

  url "https://github.com/Alpha-Coders/agent-loom/releases/download/v#{version}/AgentLoom-#{version}-macos-#{arch}.dmg",
      verified: "github.com/Alpha-Coders/agent-loom/"
  name "AgentLoom"
  desc "Manage AI agent skills across multiple CLI tools"
  homepage "https://github.com/Alpha-Coders/agent-loom"

  livecheck do
    url :url
    strategy :github_latest
  end

  # App is not notarized, requires quarantine bypass
  # Users should install with: brew install --cask --no-quarantine agentloom
  app "AgentLoom.app"

  zap trash: [
    "~/.agentloom",
    "~/Library/Application Support/com.agentloom.app",
    "~/Library/Caches/com.agentloom.app",
    "~/Library/Preferences/com.agentloom.app.plist",
    "~/Library/Saved Application State/com.agentloom.app.savedState",
  ]

  caveats <<~EOS
    #{token} is not signed with an Apple Developer certificate.

    To install, use the --no-quarantine flag:
      brew install --cask --no-quarantine #{token}

    Or if already installed, remove quarantine manually:
      xattr -cr /Applications/AgentLoom.app
  EOS
end
