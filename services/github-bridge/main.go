package main

import (
	"context"
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"

	"github.com/git-notes/github-bridge/github"
	"github.com/git-notes/github-bridge/notes"
	"github.com/git-notes/github-bridge/server"
	"github.com/git-notes/github-bridge/sync"
	"github.com/spf13/cobra"
)

func main() {
	var rootCmd = &cobra.Command{
		Use:   "github-bridge",
		Short: "GitHub Bridge for git-notes",
	}

	var serveCmd = &cobra.Command{
		Use:   "serve",
		Short: "Start HTTP API server",
		Run: func(cmd *cobra.Command, args []string) {
			portStr := os.Getenv("PORT")
			if portStr == "" {
				portStr = "7477"
			}
			
			gnBinary := os.Getenv("GN_BINARY")
			if gnBinary == "" {
				gnBinary = "git-notes" // rely on PATH
			}

			reader := notes.NewNotesReader(gnBinary, ".")
			
			log.Printf("Starting API server on :%s", portStr)
			if err := server.Run(portStr, reader); err != nil {
				log.Fatalf("Server failed: %v", err)
			}
		},
	}

	var syncCmd = &cobra.Command{
		Use:   "sync [pr_number]",
		Short: "Sync notes with GitHub PR",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			token := os.Getenv("GITHUB_TOKEN")
			repo := os.Getenv("GITHUB_REPO")
			if token == "" || repo == "" {
				log.Fatal("GITHUB_TOKEN and GITHUB_REPO env vars must be set")
			}

			parts := strings.Split(repo, "/")
			if len(parts) != 2 {
				log.Fatal("GITHUB_REPO must be in format owner/repo")
			}
			owner, repoName := parts[0], parts[1]

			prNum, err := strconv.Atoi(args[0])
			if err != nil {
				log.Fatalf("Invalid PR number: %v", err)
			}

			gnBinary := os.Getenv("GN_BINARY")
			if gnBinary == "" {
				gnBinary = "git-notes"
			}

			ghClient := github.NewClient(token)
			reader := notes.NewNotesReader(gnBinary, ".")

			ctx := context.Background()

			log.Printf("Syncing PR %d to notes...", prNum)
			if err := sync.SyncPRToNotes(ctx, ghClient, reader, owner, repoName, prNum); err != nil {
				log.Fatalf("Sync PR to Notes failed: %v", err)
			}

			log.Printf("Syncing notes to PR %d...", prNum)
			if err := sync.SyncNotesToPR(ctx, ghClient, reader, owner, repoName, prNum); err != nil {
				log.Fatalf("Sync Notes to PR failed: %v", err)
			}

			fmt.Println("Sync completed successfully.")
		},
	}

	rootCmd.AddCommand(serveCmd, syncCmd)

	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
