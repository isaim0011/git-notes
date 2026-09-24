package sync

import (
	"context"
	"fmt"
	"strconv"
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

	existingCommentIDs := parseCommentIDs(existingNotes)

	for _, c := range comments {
		if c.Path == "" || c.Line == 0 {
			continue // skip PR level comments without path/line
		}

		if existingCommentIDs[c.ID] {
			continue
		}

		bodyWithMeta := fmt.Sprintf("%s\n\n[github-comment-id: %d]", c.Body, c.ID)

		note := notes.Note{
			Body:      bodyWithMeta,
			File:      c.Path,
			Line:      c.Line,
			Namespace: "review",
		}

		if err := notesReader.WriteNote(note); err != nil {
			return fmt.Errorf("failed to write note for comment %d: %w", c.ID, err)
		}

		existingCommentIDs[c.ID] = true
	}

	return nil
}

func parseCommentIDs(notes []notes.Note) map[int64]bool {
	const prefix = "[github-comment-id: "
	commentIDs := make(map[int64]bool, len(notes))
	for _, n := range notes {
		for idx := 0; idx < len(n.Body); {
			tagIdx := strings.Index(n.Body[idx:], prefix)
			if tagIdx == -1 {
				break
			}
			start := idx + tagIdx + len(prefix)
			end := strings.IndexByte(n.Body[start:], ']')
			if end == -1 {
				break
			}
			idStr := n.Body[start : start+end]
			if id, err := strconv.ParseInt(idStr, 10, 64); err == nil {
				commentIDs[id] = true
			}
			idx = start + end + 1
		}
	}
	return commentIDs
}

func SyncNotesToPR(ctx context.Context, ghClient *github.Client, notesReader *notes.Reader, owner, repo string, prNum int) error {
	// A real implementation would need to fetch the PR to get the latest commit SHA
	// For this exercise, we will just simulate it as requiring a commit SHA.

	// notes := notesReader.ReadNotesJSON("review")
	// For each note, if it doesn't have a github-comment-id, post it to GH
	// ...

	return nil
}
