package sync

import (
	"context"
	"fmt"
	"strings"

	"github.com/git-notes/github-bridge/github"
	"github.com/git-notes/github-bridge/notes"
)

func SyncPRToNotes(ctx context.Context, ghClient *github.Client, notesReader *notes.Reader, owner, repo string, prNum int) error {
	comments, err := ghClient.GetPRComments(ctx, owner, repo, prNum)
	if err != nil {
		return fmt.Errorf("failed to get PR comments: %w", err)
	}

	existingNotes, err := notesReader.ReadNotesJSON("review")
	if err != nil {
		return fmt.Errorf("failed to read notes: %w", err)
	}

	existingMap := make(map[string]bool)
	for _, n := range existingNotes {
		// basic dedup by checking if body contains the comment ID as a tag
		// or just hash/match. For now, simple text match hack
		existingMap[n.Body] = true
	}

	var newNotes []notes.Note
	for _, c := range comments {
		if c.Path == "" || c.Line == 0 {
			continue // skip PR level comments without path/line
		}

		bodyWithMeta := fmt.Sprintf("%s\n\n[github-comment-id: %d]", c.Body, c.ID)

		found := false
		for _, n := range existingNotes {
			if strings.Contains(n.Body, fmt.Sprintf("[github-comment-id: %d]", c.ID)) {
				found = true
				break
			}
		}

		if found {
			continue
		}

		note := notes.Note{
			Body:      bodyWithMeta,
			File:      c.Path,
			Line:      c.Line,
			Namespace: "review",
		}

		newNotes = append(newNotes, note)
	}

	if len(newNotes) > 0 {
		if err := notesReader.WriteNotes(newNotes); err != nil {
			return fmt.Errorf("failed to write notes: %w", err)
		}
	}

	return nil
}

func SyncNotesToPR(ctx context.Context, ghClient *github.Client, notesReader *notes.Reader, owner, repo string, prNum int) error {
	// A real implementation would need to fetch the PR to get the latest commit SHA
	// For this exercise, we will just simulate it as requiring a commit SHA.
	
	// notes := notesReader.ReadNotesJSON("review")
	// For each note, if it doesn't have a github-comment-id, post it to GH
	// ...

	return nil
}
