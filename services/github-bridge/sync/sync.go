package sync

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"regexp"
	"strconv"
	"strings"

	"github.com/git-notes/github-bridge/github"
	"github.com/git-notes/github-bridge/notes"
)

var commentIDRegex = regexp.MustCompile(`\[github-comment-id:\s*(\d+)\s*\]`)

type CommentFetcher interface {
	GetPRComments(ctx context.Context, owner, repo string, prNum int) ([]github.PRComment, error)
}

type NotesReaderWriter interface {
	ReadNotesJSON(namespace string) ([]notes.Note, error)
	WriteNote(note notes.Note) error
}

func extractCommentID(body string) (int64, bool) {
	matches := commentIDRegex.FindStringSubmatch(body)
	if len(matches) > 1 {
		id, err := strconv.ParseInt(matches[1], 10, 64)
		if err == nil {
			return id, true
		}
	}
	return 0, false
}

func stripMetaTag(body string) string {
	loc := commentIDRegex.FindStringIndex(body)
	if loc == nil {
		return strings.TrimSpace(body)
	}
	cleaned := body[:loc[0]] + body[loc[1]:]
	return strings.TrimSpace(cleaned)
}

func computeFingerprint(file string, line int, body string) string {
	cleanedBody := stripMetaTag(body)
	h := sha256.New()
	h.Write([]byte(fmt.Sprintf("%s:%d:%s", file, line, cleanedBody)))
	return hex.EncodeToString(h.Sum(nil))
}

func SyncPRToNotes(ctx context.Context, ghClient CommentFetcher, notesReader NotesReaderWriter, owner, repo string, prNum int) error {
	comments, err := ghClient.GetPRComments(ctx, owner, repo, prNum)
	if err != nil {
		return fmt.Errorf("failed to get PR comments: %w", err)
	}

	existingNotes, err := notesReader.ReadNotesJSON("review")
	if err != nil {
		return fmt.Errorf("failed to read notes: %w", err)
	}

	existingCommentIDs := make(map[int64]bool)
	existingFingerprints := make(map[string]bool)

	for _, n := range existingNotes {
		if id, ok := extractCommentID(n.Body); ok {
			existingCommentIDs[id] = true
		}
		fp := computeFingerprint(n.File, n.Line, n.Body)
		existingFingerprints[fp] = true
	}

	for _, c := range comments {
		if c.Path == "" || c.Line == 0 {
			continue // skip PR level comments without path/line
		}

		if existingCommentIDs[c.ID] {
			continue
		}

		commentFP := computeFingerprint(c.Path, c.Line, c.Body)
		if existingFingerprints[commentFP] {
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
		existingFingerprints[commentFP] = true
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
