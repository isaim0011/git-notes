package notes

import (
	"os"
	"path/filepath"
	"reflect"
	"testing"
)

func createMockBinary(t *testing.T, scriptContent string) string {
	t.Helper()
	tmpDir := t.TempDir()
	binPath := filepath.Join(tmpDir, "mock-git-notes")

	fullScript := "#!/bin/sh\n" + scriptContent
	err := os.WriteFile(binPath, []byte(fullScript), 0755)
	if err != nil {
		t.Fatalf("failed to create mock binary: %v", err)
	}

	return binPath
}

func TestNewNotesReader(t *testing.T) {
	binary := "/usr/local/bin/git-notes"
	repo := "/tmp/repo"
	r := NewNotesReader(binary, repo)

	if r == nil {
		t.Fatal("expected non-nil Reader")
	}
	if r.gnBinary != binary {
		t.Errorf("expected gnBinary %q, got %q", binary, r.gnBinary)
	}
	if r.repoPath != repo {
		t.Errorf("expected repoPath %q, got %q", repo, r.repoPath)
	}
}

func TestReadNotesJSON_Success(t *testing.T) {
	script := `
if [ "$1" = "list" ] && [ "$2" = "--json" ]; then
    echo '[{"id":"n1","body":"hello","file":"main.go","line":42,"namespace":"default","author":"alice","timestamp":"2023-01-01T00:00:00Z"}]'
    exit 0
fi
echo "invalid args: $@" >&2
exit 1
`
	binPath := createMockBinary(t, script)
	repoDir := t.TempDir()
	reader := NewNotesReader(binPath, repoDir)

	notes, err := reader.ReadNotesJSON("")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	expected := []Note{
		{
			ID:        "n1",
			Body:      "hello",
			File:      "main.go",
			Line:      42,
			Namespace: "default",
			Author:    "alice",
			Timestamp: "2023-01-01T00:00:00Z",
		},
	}

	if !reflect.DeepEqual(notes, expected) {
		t.Errorf("got notes %+v, expected %+v", notes, expected)
	}
}

func TestReadNotesJSON_WithNamespace(t *testing.T) {
	script := `
if [ "$1" = "list" ] && [ "$2" = "--json" ] && [ "$3" = "--namespace" ] && [ "$4" = "review" ]; then
    echo '[{"id":"n2","body":"review comment","file":"reader.go","line":15,"namespace":"review"}]'
    exit 0
fi
echo "invalid args: $@" >&2
exit 1
`
	binPath := createMockBinary(t, script)
	repoDir := t.TempDir()
	reader := NewNotesReader(binPath, repoDir)

	notes, err := reader.ReadNotesJSON("review")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	expected := []Note{
		{
			ID:        "n2",
			Body:      "review comment",
			File:      "reader.go",
			Line:      15,
			Namespace: "review",
		},
	}

	if !reflect.DeepEqual(notes, expected) {
		t.Errorf("got notes %+v, expected %+v", notes, expected)
	}
}

func TestReadNotesJSON_EmptyOutput(t *testing.T) {
	script := `
exit 0
`
	binPath := createMockBinary(t, script)
	repoDir := t.TempDir()
	reader := NewNotesReader(binPath, repoDir)

	notes, err := reader.ReadNotesJSON("")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(notes) != 0 {
		t.Errorf("expected empty notes slice, got %d items", len(notes))
	}
}

func TestReadNotesJSON_CommandFailure(t *testing.T) {
	script := `
echo "git-notes internal error" >&2
exit 127
`
	binPath := createMockBinary(t, script)
	repoDir := t.TempDir()
	reader := NewNotesReader(binPath, repoDir)

	notes, err := reader.ReadNotesJSON("")
	if err == nil {
		t.Fatal("expected error, got nil")
	}
	if notes != nil {
		t.Errorf("expected nil notes, got %v", notes)
	}

	expectedSubstring := "git-notes failed:"
	if !contains(err.Error(), expectedSubstring) || !contains(err.Error(), "git-notes internal error") {
		t.Errorf("error %q should contain %q and stderr message", err.Error(), expectedSubstring)
	}
}

func TestReadNotesJSON_InvalidJSON(t *testing.T) {
	script := `
echo "invalid json output"
exit 0
`
	binPath := createMockBinary(t, script)
	repoDir := t.TempDir()
	reader := NewNotesReader(binPath, repoDir)

	notes, err := reader.ReadNotesJSON("")
	if err == nil {
		t.Fatal("expected error for invalid json, got nil")
	}
	if notes != nil {
		t.Errorf("expected nil notes, got %v", notes)
	}

	expectedSubstring := "failed to parse notes JSON:"
	if !contains(err.Error(), expectedSubstring) {
		t.Errorf("error %q should contain %q", err.Error(), expectedSubstring)
	}
}

func TestWriteNote_Success(t *testing.T) {
	script := `
if [ "$1" = "add" ] && [ "$2" = "--file" ] && [ "$3" = "main.go" ] && [ "$4" = "--line" ] && [ "$5" = "10" ] && [ "$6" = "--message" ] && [ "$7" = "test note" ]; then
    exit 0
fi
echo "invalid write args: $@" >&2
exit 1
`
	binPath := createMockBinary(t, script)
	repoDir := t.TempDir()
	reader := NewNotesReader(binPath, repoDir)

	note := Note{
		File: "main.go",
		Line: 10,
		Body: "test note",
	}

	err := reader.WriteNote(note)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestWriteNote_WithNamespace(t *testing.T) {
	script := `
if [ "$1" = "add" ] && [ "$2" = "--file" ] && [ "$3" = "main.go" ] && [ "$4" = "--line" ] && [ "$5" = "10" ] && [ "$6" = "--message" ] && [ "$7" = "test note" ] && [ "$8" = "--namespace" ] && [ "$9" = "custom" ]; then
    exit 0
fi
echo "invalid write args: $@" >&2
exit 1
`
	binPath := createMockBinary(t, script)
	repoDir := t.TempDir()
	reader := NewNotesReader(binPath, repoDir)

	note := Note{
		File:      "main.go",
		Line:      10,
		Body:      "test note",
		Namespace: "custom",
	}

	err := reader.WriteNote(note)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestWriteNote_Failure(t *testing.T) {
	script := `
echo "failed to write note" >&2
exit 1
`
	binPath := createMockBinary(t, script)
	repoDir := t.TempDir()
	reader := NewNotesReader(binPath, repoDir)

	note := Note{
		File: "main.go",
		Line: 10,
		Body: "test note",
	}

	err := reader.WriteNote(note)
	if err == nil {
		t.Fatal("expected error, got nil")
	}

	if !contains(err.Error(), "git-notes add failed:") || !contains(err.Error(), "failed to write note") {
		t.Errorf("error %q should contain failure details", err.Error())
	}
}

func contains(s, substr string) bool {
	return stringContains(s, substr)
}

func stringContains(s, substr string) bool {
	if len(substr) == 0 {
		return true
	}
	for i := 0; i+len(substr) <= len(s); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
