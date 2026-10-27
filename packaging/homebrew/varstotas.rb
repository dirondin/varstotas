cask "varstotas" do
  version "@VERSION@"
  sha256 "@SHA256_MACOS_DMG@"

  url "https://github.com/dirondin/varstotas/releases/download/release-#{version}/Varstotas-#{version}-macos-universal.dmg"
  name "Varstotas"
  desc "3D modelling for makers: furniture, workshop projects and 3D prints"
  homepage "https://github.com/dirondin/varstotas"

  livecheck do
    url :url
    regex(/^release[._-]v?(\d+(?:\.\d+)+)$/i)
    strategy :github_latest
  end

  depends_on :macos

  app "Varstotas.app"

  zap trash: [
    "~/.varstotas", # 0.1.0 and earlier
    "~/Library/Application Support/Varstotas",
    "~/Library/Saved Application State/dev.varstotas.app.savedState",
  ]

  caveats <<~EOS
    Varstotas is not notarized by Apple, so macOS blocks its first launch.
    Right-click Varstotas in Applications → Open → Open (once), or run:
      xattr -dr com.apple.quarantine /Applications/Varstotas.app
  EOS
end
