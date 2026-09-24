package notes

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func createMockScript(t *testing.T, content string) string {
	t.Helper()
	dir := t.TempDir()
	scriptPath := filepath.Join(dir, "mock-gn")
	err := os.WriteFile(scriptPath, []byte(content), 0755)
	if err != nil {
		t.Fatalf("failed to create mock script: %v", err)
	}
	return scriptPath
}

func TestWriteNote_ErrorPaths(t *testing.T) {
	t.Run("non-existent binary path", func(t *testing.T) {
		reader := NewNotesReader("/nonexistent/binary/git-notes", t.TempDir())
		note := Note{
			File: "main.go",
			Line: 10,
			Body: "test note",
		}

		err := reader.WriteNote(note)
		if err == nil {
			t.Fatal("expected error when binary does not exist, got nil")
		}
		if !strings.Contains(err.Error(), "git-notes add failed:") {
			t.Errorf("unexpected error message: %v", err)
		}
	})

	t.Run("binary execution failure with stderr output", func(t *testing.T) {
		mockScript := createMockScript(t, "#!/bin/sh\necho 'custom error on stderr' >&2\nexit 1\n")
		reader := NewNotesReader(mockScript, t.TempDir())
		note := Note{
			File:      "main.go",
			Line:      42,
			Body:      "failed note",
			Namespace: "custom",
		}

		err := reader.WriteNote(note)
		if err == nil {
			t.Fatal("expected error when binary exits with non-zero status, got nil")
		}
		if !strings.Contains(err.Error(), "git-notes add failed:") {
			t.Errorf("error missing prefix: %v", err)
		}
		if !strings.Contains(err.Error(), "stderr: custom error on stderr") {
			t.Errorf("error missing stderr message: %v", err)
		}
	})
}

func TestWriteNote_Success(t *testing.T) {
	t.Run("without namespace", func(t *testing.T) {
		mockScript := createMockScript(t, "#!/bin/sh\nexit 0\n")
		reader := NewNotesReader(mockScript, t.TempDir())
		note := Note{
			File: "lib.go",
			Line: 15,
			Body: "hello world",
		}

		err := reader.WriteNote(note)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
	})

	t.Run("with namespace", func(t *testing.T) {
		mockScript := createMockScript(t, `#!/bin/sh
args="$*"
if ! echo "$args" | grep -q "\-\-namespace review"; then
  echo "missing namespace argument" >&2
  exit 1
fi
exit 0
`)
		reader := NewNotesReader(mockScript, t.TempDir())
		note := Note{
			File:      "lib.go",
			Line:      15,
			Body:      "hello world",
			Namespace: "review",
		}

		err := reader.WriteNote(note)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
	})
}

func TestReadNotesJSON(t *testing.T) {
	t.Run("binary failure", func(t *testing.T) {
		reader := NewNotesReader("/nonexistent/binary", t.TempDir())
		_, err := reader.ReadNotesJSON("")
		if err == nil {
			t.Fatal("expected error, got nil")
		}
		if !strings.Contains(err.Error(), "git-notes failed:") {
			t.Errorf("unexpected error message: %v", err)
		}
	})

	t.Run("invalid json output", func(t *testing.T) {
		mockScript := createMockScript(t, "#!/bin/sh\necho 'invalid json'\nexit 0\n")
		reader := NewNotesReader(mockScript, t.TempDir())
		_, err := reader.ReadNotesJSON("")
		if err == nil {
			t.Fatal("expected error, got nil")
		}
		if !strings.Contains(err.Error(), "failed to parse notes JSON:") {
			t.Errorf("unexpected error message: %v", err)
		}
	})

	t.Run("empty output", func(t *testing.T) {
		mockScript := createMockScript(t, "#!/bin/sh\nexit 0\n")
		reader := NewNotesReader(mockScript, t.TempDir())
		notes, err := reader.ReadNotesJSON("")
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		if len(notes) != 0 {
			t.Errorf("expected empty notes slice, got %d notes", len(notes))
		}
	})

	t.Run("valid json notes output with namespace", func(t *testing.T) {
		jsonOutput := `[{"id":"1","body":"test","file":"a.go","line":5,"namespace":"review"}]`
		mockScript := createMockScript(t, "#!/bin/sh\necho '"+jsonOutput+"'\nexit 0\n")
		reader := NewNotesReader(mockScript, t.TempDir())
		notes, err := reader.ReadNotesJSON("review")
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		if len(notes) != 1 {
			t.Fatalf("expected 1 note, got %d", len(notes))
		}
		if notes[0].ID != "1" || notes[0].Body != "test" || notes[0].File != "a.go" || notes[0].Line != 5 || notes[0].Namespace != "review" {
			t.Errorf("unexpected note content: %+v", notes[0])
		}
	})
}
