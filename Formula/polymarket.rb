class Polymarket < Formula
  desc "CLI for Polymarket — browse markets, trade, and manage positions"
  homepage "https://github.com/Polymarket/polymarket-cli"
  version "0.1.4"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/Polymarket/polymarket-cli/releases/download/v#{version}/polymarket-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "ec1dee1e1b7a66e8d2fc60ceb5e212b9f78e403b8079759a3b29b885cc4bb7ef"
    end

    on_arm do
      url "https://github.com/Polymarket/polymarket-cli/releases/download/v#{version}/polymarket-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "c054c298417340d23995aa1181a2e47eea5bf7cb335c5417eb78e714fc433ac9"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Polymarket/polymarket-cli/releases/download/v#{version}/polymarket-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "b9f70ad1e3553de8a6546025ea1c7d14d69d51949c7609cac4cf8d9c686baa72"
    end

    on_arm do
      url "https://github.com/Polymarket/polymarket-cli/releases/download/v#{version}/polymarket-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "4c94f152404483cdfe6fa3fed60b633a36f2fc813f3ee78091e6046423144292"
    end
  end

  def install
    bin.install "polymarket"
  end

  test do
    assert_match "polymarket", shell_output("#{bin}/polymarket --version")
  end
end
