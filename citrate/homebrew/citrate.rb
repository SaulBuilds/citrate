class Lattice < Formula
  desc "Lattice AI blockchain platform CLI and node"
  homepage "https://citrate.ai"
  version "0.1.0"

  if Hardware::CPU.intel?
    if OS.mac?
      url "https://github.com/citrate-ai/citrate-v3/releases/download/v#{version}/citrate-v#{version}-macos-x86_64.tar.gz"
      sha256 "TBD" # Will be updated during release
    else
      url "https://github.com/citrate-ai/citrate-v3/releases/download/v#{version}/citrate-v#{version}-linux-x86_64.tar.gz"
      sha256 "TBD" # Will be updated during release
    end
  else
    if OS.mac?
      url "https://github.com/citrate-ai/citrate-v3/releases/download/v#{version}/citrate-v#{version}-macos-arm64.tar.gz"
      sha256 "TBD" # Will be updated during release
    else
      url "https://github.com/citrate-ai/citrate-v3/releases/download/v#{version}/citrate-v#{version}-linux-arm64.tar.gz"
      sha256 "TBD" # Will be updated during release
    end
  end

  license "Apache-2.0"

  depends_on "openssl"

  def install
    bin.install "citrate"
    bin.install "citrate-cli"
    bin.install "citrate-wallet"
    bin.install "faucet"

    # Install shell completions
    generate_completions_from_executable(bin/"citrate-cli", "completion")

    # Create config directory
    (etc/"citrate").mkpath

    # Install example configuration
    (etc/"citrate").install "config.toml.example" if File.exist?("config.toml.example")
  end

  def post_install
    puts <<~EOS
      Lattice AI blockchain platform has been installed!

      Quick start:
        1. Initialize a new node:
           citrate init --data-dir ~/.citrate

        2. Start the node:
           citrate start --config ~/.citrate/config.toml

        3. Create a wallet:
           citrate-wallet create

        4. Check node status:
           citrate-cli status

      Documentation: https://docs.citrate.ai
      Community: https://discord.gg/citrate-ai
    EOS
  end

  service do
    run [opt_bin/"citrate", "start", "--config", etc/"citrate/config.toml"]
    keep_alive true
    log_path var/"log/citrate.log"
    error_log_path var/"log/citrate.log"
    working_dir var/"lib/citrate"
  end

  test do
    system "#{bin}/citrate", "--version"
    system "#{bin}/citrate-cli", "--help"
    system "#{bin}/citrate-wallet", "--help"
    system "#{bin}/faucet", "--help"
  end
end