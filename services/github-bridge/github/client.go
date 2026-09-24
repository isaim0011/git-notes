package github

import (
	"context"
	"net/url"

	"github.com/google/go-github/v66/github"
	"golang.org/x/oauth2"
)

type PR struct {
	Number int
	Title  string
}

type PRComment struct {
	ID        int64
	Body      string
	Path      string
	Line      int
	CommitID  string
	Author    string
	CreatedAt string
}

type Client struct {
	gh *github.Client
}

func NewClient(token string) *Client {
	ctx := context.Background()
	ts := oauth2.StaticTokenSource(
		&oauth2.Token{AccessToken: token},
	)
	tc := oauth2.NewClient(ctx, ts)

	return &Client{
		gh: github.NewClient(tc),
	}
}

func NewClientWithBaseURL(token, baseURL string) (*Client, error) {
	ctx := context.Background()
	ts := oauth2.StaticTokenSource(
		&oauth2.Token{AccessToken: token},
	)
	tc := oauth2.NewClient(ctx, ts)
	c := github.NewClient(tc)
	u, err := url.Parse(baseURL)
	if err != nil {
		return nil, err
	}
	c.BaseURL = u
	return &Client{gh: c}, nil
}

func (c *Client) GetPRComments(ctx context.Context, owner, repo string, prNum int) ([]PRComment, error) {
	opts := &github.PullRequestListCommentsOptions{
		ListOptions: github.ListOptions{PerPage: 100},
	}

	var allComments []PRComment
	for {
		comments, resp, err := c.gh.PullRequests.ListComments(ctx, owner, repo, prNum, opts)
		if err != nil {
			return nil, err
		}

		for _, comm := range comments {
			line := 0
			if comm.Line != nil {
				line = *comm.Line
			}
			path := ""
			if comm.Path != nil {
				path = *comm.Path
			}
			body := ""
			if comm.Body != nil {
				body = *comm.Body
			}
			author := ""
			if comm.User != nil && comm.User.Login != nil {
				author = *comm.User.Login
			}
			createdAt := ""
			if comm.CreatedAt != nil {
				createdAt = comm.CreatedAt.String()
			}
			commitID := ""
			if comm.CommitID != nil {
				commitID = *comm.CommitID
			}

			allComments = append(allComments, PRComment{
				ID:        *comm.ID,
				Body:      body,
				Path:      path,
				Line:      line,
				Author:    author,
				CreatedAt: createdAt,
				CommitID:  commitID,
			})
		}

		if resp.NextPage == 0 {
			break
		}
		opts.Page = resp.NextPage
	}

	return allComments, nil
}

func (c *Client) PostPRReviewComment(ctx context.Context, owner, repo string, prNum int, body, path string, line int, commitSHA string) error {
	comment := &github.PullRequestComment{
		Body:     github.String(body),
		Path:     github.String(path),
		Line:     github.Int(line),
		CommitID: github.String(commitSHA),
	}

	_, _, err := c.gh.PullRequests.CreateComment(ctx, owner, repo, prNum, comment)
	return err
}

func (c *Client) ListOpenPRs(ctx context.Context, owner, repo string) ([]PR, error) {
	opts := &github.PullRequestListOptions{
		State:       "open",
		ListOptions: github.ListOptions{PerPage: 100},
	}

	var allPRs []PR
	for {
		prs, resp, err := c.gh.PullRequests.List(ctx, owner, repo, opts)
		if err != nil {
			return nil, err
		}

		for _, pr := range prs {
			allPRs = append(allPRs, PR{
				Number: *pr.Number,
				Title:  *pr.Title,
			})
		}

		if resp.NextPage == 0 {
			break
		}
		opts.Page = resp.NextPage
	}

	return allPRs, nil
}
