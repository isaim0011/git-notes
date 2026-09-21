# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Active support  |

## Reporting a Vulnerability

**Please do NOT open a public GitHub issue for security vulnerabilities.**

Instead, report them privately via one of these channels:

1. **GitHub Private Vulnerability Reporting** (preferred):
   Go to [Security → Report a vulnerability](https://github.com/isaim0011/git-notes/security/advisories/new)

2. **Email**: security@git-notes.dev *(monitored)*

### What to include

- A clear description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (optional but appreciated)

### Response Timeline

| Stage | Target |
|---|---|
| Acknowledgement | Within 48 hours |
| Initial assessment | Within 5 business days |
| Fix + disclosure | Within 90 days |

We follow [coordinated disclosure](https://en.wikipedia.org/wiki/Coordinated_vulnerability_disclosure).
Reporters who follow this policy will be credited in the release notes.

## Scope

In scope:
- `gn-core` engine (arbitrary code execution via crafted git objects)
- `github-bridge` API server (auth bypass, SSRF, injection)
- `gn-cli` / `gn-tui` (path traversal, shell injection)
- Chrome extension (XSS on GitHub pages, data exfiltration)

Out of scope:
- Git itself (report to the Git security team)
- GitHub's platform (report to GitHub)
- Theoretical attacks without a working proof of concept
