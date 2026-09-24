class GitNotes < Formula
  desc "Decentralized code comments and inline reviews stored directly in Git"
  homepage "https://github.com/isaim0011/git-notes"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-macos-aarch64"
      sha256 "8b3459a818f12598115cd2e0fa5cf186d1e3d290f5cdb1a00ab642ab45f10870"

      def install
        bin.install "git-notes-macos-aarch64" => "git-notes"
      end
    else
      url "https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-macos-x86_64"
      sha256 "bf02646655377d7038d035a47cea79ef2d4855ff1ff3535da585a07563a29671"

      def install
        bin.install "git-notes-macos-x86_64" => "git-notes"
      end
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-linux-x86_64"
      sha256 "a96915458ed99871881517869153b38fca56c7ca8499c13b8aa9dcf06f52c646"

      def install
        bin.install "git-notes-linux-x86_64" => "git-notes"
      end
    end
  end

  test do
    system "#{bin}/git-notes", "--version"
  end
end
