# typed: false
# frozen_string_literal: true

cask "agentloom" do
  arch arm: "arm64", intel: "5d3fec3838cf9be0ac8a32a5f73049ec2af344efceed7a771e1a9ff0363ec550"

  version "1.1.1"
  sha256 arm:   "3e9ef05ee334270751ab548afb147a8943c31d89e61b8de4c3c96e6813af01d4",
         intel: "5d3fec3838cf9be0ac8a32a5f73049ec2af344efceed7a771e1a9ff0363ec550"

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
