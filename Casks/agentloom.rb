# typed: false
# frozen_string_literal: true

cask "agentloom" do
  arch arm: "arm64", intel: "570b4610979962ae6c0d6506696083bdaa63f95ef25d469f1adb64cfba23ece3"

  version "1.2"
  sha256 arm:   "2d5c0323969776ba31a25ae0e6155c6dc41d77c9829e7a49b3ded993d3d8accc",
         intel: "570b4610979962ae6c0d6506696083bdaa63f95ef25d469f1adb64cfba23ece3"

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
    "~/.agents",
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
