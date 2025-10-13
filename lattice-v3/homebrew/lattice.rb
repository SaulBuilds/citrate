class Lattice < Formula
  desc "Lattice AI blockchain platform CLI and node"
  homepage "https://lattice.ai"
  version "0.1.0"

  if Hardware::CPU.intel?
    if OS.mac?
      url "https://github.com/lattice-ai/lattice-v3/releases/download/v#{version}/lattice-v#{version}-macos-x86_64.tar.gz"
      sha256 "TBD" # Will be updated during release
    else
      url "https://github.com/lattice-ai/lattice-v3/releases/download/v#{version}/lattice-v#{version}-linux-x86_64.tar.gz"
      sha256 "TBD" # Will be updated during release
    end
  else
    if OS.mac?
      url "https://github.com/lattice-ai/lattice-v3/releases/download/v#{version}/lattice-v#{version}-macos-arm64.tar.gz"
      sha256 "TBD" # Will be updated during release
    else
      url "https://github.com/lattice-ai/lattice-v3/releases/download/v#{version}/lattice-v#{version}-linux-arm64.tar.gz"
      sha256 "TBD" # Will be updated during release
    end
  end

  license "Apache-2.0"

  depends_on "openssl"

  def install
    bin.install "lattice"
    bin.install "lattice-cli"
    bin.install "lattice-wallet"
    bin.install "faucet"

    # Install shell completions
    generate_completions_from_executable(bin/"lattice-cli", "completion")

    # Create config directory
    (etc/"lattice").mkpath

    # Install example configuration
    (etc/"lattice").install "config.toml.example" if File.exist?("config.toml.example")
  end

  def post_install
    puts <<~EOS
      Lattice AI blockchain platform has been installed!

      Quick start:
        1. Initialize a new node:
           lattice init --data-dir ~/.lattice

        2. Start the node:
           lattice start --config ~/.lattice/config.toml

        3. Create a wallet:
           lattice-wallet create

        4. Check node status:
           lattice-cli status

      Documentation: https://docs.lattice.ai
      Community: https://discord.gg/lattice-ai
    EOS
  end

  service do
    run [opt_bin/"lattice", "start", "--config", etc/"lattice/config.toml"]
    keep_alive true
    log_path var/"log/lattice.log"
    error_log_path var/"log/lattice.log"
    working_dir var/"lib/lattice"
  end

  test do
    system "#{bin}/lattice", "--version"
    system "#{bin}/lattice-cli", "--help"
    system "#{bin}/lattice-wallet", "--help"
    system "#{bin}/faucet", "--help"
  end
end