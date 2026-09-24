package notes

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os/exec"
)

type Note struct {
	ID        string `json:"id,omitempty"`
	Body      string `json:"body"`
	File      string `json:"file"`
	Line      int    `json:"line"`
	Namespace string `json:"namespace"`
	Author    string `json:"author,omitempty"`
	Timestamp string `json:"timestamp,omitempty"`
}

type Reader struct {
	gnBinary string
	repoPath string
}

func NewNotesReader(gnBinary, repoPath string) *Reader {
	return &Reader{
		gnBinary: gnBinary,
		repoPath: repoPath,
	}
}

func (r *Reader) ReadNotesJSON(namespace string) ([]Note, error) {
	args := []string{"list", "--json"}
	if namespace != "" {
		args = append(args, "--namespace", namespace)
	}

	cmd := exec.Command(r.gnBinary, args...)
	cmd.Dir = r.repoPath

	var outb, errb bytes.Buffer
	cmd.Stdout = &outb
	cmd.Stderr = &errb

	err := cmd.Run()
	if err != nil {
		return nil, fmt.Errorf("git-notes failed: %v, stderr: %s", err, errb.String())
	}

	var notes []Note
	if len(outb.Bytes()) == 0 {
		return notes, nil
	}

	if err := json.Unmarshal(outb.Bytes(), &notes); err != nil {
		return nil, fmt.Errorf("failed to parse notes JSON: %v, output: %s", err, outb.String())
	}

	return notes, nil
}

func (r *Reader) WriteNotes(notes []Note) error {
	if len(notes) == 0 {
		return nil
	}

	notesJSON, err := json.Marshal(notes)
	if err != nil {
		return fmt.Errorf("failed to marshal notes: %w", err)
	}

	cmd := exec.Command(r.gnBinary, "add-bulk", "--json", string(notesJSON))
	cmd.Dir = r.repoPath

	var outb, errb bytes.Buffer
	cmd.Stdout = &outb
	cmd.Stderr = &errb

	if err := cmd.Run(); err != nil {
		return fmt.Errorf("git-notes add-bulk failed: %v, stderr: %s", err, errb.String())
	}

	return nil
}

func (r *Reader) WriteNote(note Note) error {
	return r.WriteNotes([]Note{note})
}
