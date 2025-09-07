class MaccleanupRust < Formula
  desc "🧹 Mac Cleanup Tool (Rust Edition) By Gappa - Clean your Mac system efficiently"
  homepage "https://github.com/gappa55/maccleanup-rust"
  url "https://github.com/gappa55/maccleanup-rust/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "MIT"
  head "https://github.com/gappa55/maccleanup-rust.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
  end

  test do
    # Test that the binary exists and shows help
    assert_match "Mac Cleanup Tool (Rust Edition) By Gappa", shell_output("#{bin}/maccleanup-rust --help")
    
    # Test dry run mode works
    assert_match "DRY RUN mode", shell_output("#{bin}/maccleanup-rust --dry-run")
  end
end