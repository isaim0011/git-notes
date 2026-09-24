package notes

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

func setupTestRepo(tb testing.TB) string {
	tb.Helper()
	dir, err := os.MkdirTemp("", "git-notes-test-*")
	if err != nil {
		tb.Fatalf("failed to create temp dir: %v", err)
	}

	runGit := func(args ...string) {
		cmd := exec.Command("git", args...)
		cmd.Dir = dir
		if err := cmd.Run(); err != nil {
			tb.Fatalf("git %v failed: %v", args, err)
		}
	}

	runGit("init")
	runGit("config", "user.email", "test@example.com")
	runGit("config", "user.name", "Test User")

	dummyFile := filepath.Join(dir, "README.md")
	if err := os.WriteFile(dummyFile, []byte("# Test Repo\n"), 0644); err != nil {
		tb.Fatalf("failed to write dummy file: %v", err)
	}
	runGit("add", ".")
	runGit("commit", "-m", "Initial commit")

	return dir
}

func TestWriteNotesBulk(t *testing.T) {
	gnBinary, err := filepath.Abs("../../../target/debug/git-notes")
	if err != nil {
		t.Fatalf("failed to get binary path: %v", err)
	}
	if _, err := os.Stat(gnBinary); os.IsNotExist(err) {
		t.Skip("git-notes binary not found at ", gnBinary)
	}

	repoPath := setupTestRepo(t)
	defer os.RemoveAll(repoPath)

	reader := NewNotesReader(gnBinary, repoPath)

	const count = 50
	var batch []Note
	for j := 0; j < count; j++ {
		batch = append(batch, Note{
			Body:      fmt.Sprintf("Comment body %d\n\n[github-comment-id: %d]", j, j),
			File:      "README.md",
			Line:      1,
			Namespace: "review",
		})
	}

	if err := reader.WriteNotes(batch); err != nil {
		t.Fatalf("WriteNotes failed: %v", err)
	}

	readNotes, err := reader.ReadNotesJSON("review")
	if err != nil {
		t.Fatalf("ReadNotesJSON failed: %v", err)
	}
	if len(readNotes) != count {
		t.Fatalf("expected %d notes, got %d", count, len(readNotes))
	}
}

func BenchmarkWriteNotesSequential(b *testing.B) {
	gnBinary, err := filepath.Abs("../../../target/debug/git-notes")
	if err != nil {
		b.Fatalf("failed to get binary path: %v", err)
	}

	for i := 0; i < b.N; i++ {
		repoPath := setupTestRepo(b)
		reader := NewNotesReader(gnBinary, repoPath)

		for j := 0; j < 50; j++ {
			note := Note{
				Body:      fmt.Sprintf("Comment body %d-%d\n\n[github-comment-id: %d]", i, j, j),
				File:      "README.md",
				Line:      1,
				Namespace: "review",
			}
			if err := reader.WriteNotes([]Note{note}); err != nil {
				b.Fatalf("WriteNote failed: %v", err)
			}
		}
		os.RemoveAll(repoPath)
	}
}

func BenchmarkWriteNotesBulk(b *testing.B) {
	gnBinary, err := filepath.Abs("../../../target/debug/git-notes")
	if err != nil {
		b.Fatalf("failed to get binary path: %v", err)
	}

	for i := 0; i < b.N; i++ {
		repoPath := setupTestRepo(b)
		reader := NewNotesReader(gnBinary, repoPath)

		var batch []Note
		for j := 0; j < 50; j++ {
			batch = append(batch, Note{
				Body:      fmt.Sprintf("Comment body %d-%d\n\n[github-comment-id: %d]", i, j, j),
				File:      "README.md",
				Line:      1,
				Namespace: "review",
			})
		}
		if err := reader.WriteNotes(batch); err != nil {
			b.Fatalf("WriteNotes failed: %v", err)
		}
		os.RemoveAll(repoPath)
	}
}
