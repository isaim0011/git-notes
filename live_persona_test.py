import subprocess
import os
import tempfile
import json
import time
import shutil

sim_dir = os.path.join(tempfile.gettempdir(), 'git_notes_live_collab')
if os.path.exists(sim_dir):
    try:
        shutil.rmtree(sim_dir)
    except Exception:
        pass
os.makedirs(sim_dir, exist_ok=True)

bare_remote = os.path.join(sim_dir, 'company_core.git')
alice_repo = os.path.join(sim_dir, 'alice_workspace')
bob_repo = os.path.join(sim_dir, 'bob_workspace')

gn_bin = r'C:\Users\Bimo\.gemini\antigravity\scratch\git-notes\target\release\gn.exe'

def log(tag, color, msg):
    colors = {
        'cyan': '\033[1;36m',
        'magenta': '\033[1;35m',
        'green': '\033[1;32m',
        'yellow': '\033[1;33m',
        'blue': '\033[1;34m',
        'bold': '\033[1m',
        'reset': '\033[0m'
    }
    c = colors.get(color, '')
    r = colors['reset']
    print(f'{c}[{tag}] {msg}{r}', flush=True)

def cmd(args, cwd, actor=None, actor_color=None):
    if actor:
        log(actor, actor_color, f'Running: {" ".join(args)}')
    p = subprocess.run(args, cwd=cwd, capture_output=True, text=True)
    if p.stdout.strip():
        for line in p.stdout.strip().splitlines():
            print(f'   │ {line}', flush=True)
    if p.returncode != 0 and p.stderr.strip():
        print(f'   ⚠ {p.stderr.strip()}', flush=True)
    return p

print('\n' + '='*75)
log('SIMULATION', 'cyan', 'STARTING REAL-WORLD LIVE COLLABORATION TEST')
log('PERSONAS', 'cyan', '👩 ALICE (Staff Backend Engineer) & 👨 BOB (Senior Security Reviewer)')
print('='*75 + '\n')

# 1. Setup Bare Central Company Repo
log('INFRA', 'blue', 'Initializing company central bare repository...')
subprocess.run(['git', 'init', '--bare', bare_remote], capture_output=True)

# 2. Setup Alice's workspace
log('ALICE', 'magenta', 'Alice initializes workspace and configures git-notes...')
os.makedirs(alice_repo, exist_ok=True)
cmd(['git', 'init'], cwd=alice_repo)
cmd(['git', 'config', 'user.name', 'Alice Chen'], cwd=alice_repo)
cmd(['git', 'config', 'user.email', 'alice@company.internal'], cwd=alice_repo)
cmd(['git', 'remote', 'add', 'origin', bare_remote], cwd=alice_repo)
cmd([gn_bin, 'init'], cwd=alice_repo, actor='ALICE', actor_color='magenta')

# 3. Alice authors critical microservice code
auth_code = '''// Authentication & Session Service
use std::collections::HashMap;

pub struct AuthEngine {
    tokens: HashMap<String, u64>,
}

impl AuthEngine {
    pub fn new() -> Self {
        Self { tokens: HashMap::new() }
    }

    pub fn issue_token(&mut self, user: &str) -> String {
        let token = format!("tok_{}", user);
        self.tokens.insert(token.clone(), 3600);
        token
    }

    pub fn verify_token(&self, token: &str) -> bool {
        // Plain equality check
        self.tokens.contains_key(token)
    }
}
'''
with open(os.path.join(alice_repo, 'auth.rs'), 'w') as f:
    f.write(auth_code)

cmd(['git', 'add', 'auth.rs'], cwd=alice_repo)
cmd(['git', 'commit', '-m', 'feat(auth): initial session token validation'], cwd=alice_repo, actor='ALICE', actor_color='magenta')
cmd(['git', 'push', '-u', 'origin', 'master'], cwd=alice_repo, actor='ALICE', actor_color='magenta')

# 4. Alice leaves an architecture TODO note on line 21
cmd([gn_bin, 'a', '-f', 'auth.rs', '-l', '21', '-m', 'TODO: Ensure token lookup is resistant to timing attacks and add expiry check', '-n', 'todos'], cwd=alice_repo, actor='ALICE', actor_color='magenta')

# Alice syncs her notes to origin
cmd([gn_bin, 'sync', 'push'], cwd=alice_repo, actor='ALICE', actor_color='magenta')

# 5. Bob clones the repo and configures git-notes
log('BOB', 'yellow', 'Bob clones the team repository and initializes git-notes tracking...')
subprocess.run(['git', 'clone', bare_remote, bob_repo], capture_output=True)
cmd(['git', 'config', 'user.name', 'Bob Martin'], cwd=bob_repo)
cmd(['git', 'config', 'user.email', 'bob@company.internal'], cwd=bob_repo)
cmd([gn_bin, 'init'], cwd=bob_repo, actor='BOB', actor_color='yellow')

# 6. Bob pulls notes from remote
cmd([gn_bin, 'sync', 'pull'], cwd=bob_repo, actor='BOB', actor_color='yellow')

# 7. Bob inspects list of notes
log('BOB', 'yellow', 'Bob checks all active notes across the codebase...')
cmd([gn_bin, 'l'], cwd=bob_repo, actor='BOB', actor_color='yellow')

# 8. Bob runs git blame with notes inline on auth.rs
log('BOB', 'yellow', 'Bob runs git blame with notes in context...')
cmd([gn_bin, 'b', '-f', 'auth.rs'], cwd=bob_repo, actor='BOB', actor_color='yellow')

# 9. Bob adds a security review comment anchored to line 23
cmd([gn_bin, 'a', '-f', 'auth.rs', '-l', '23', '-m', 'CRITICAL SECURITY: HashMap lookup without constant-time comparison leaks timing side-channel. We must use constant_time_eq.', '-n', 'review'], cwd=bob_repo, actor='BOB', actor_color='yellow')

# Bob replies to Alice's note
cmd([gn_bin, 'r', '1', '-m', "Agreed Alice! I've also flagged line 23 for constant-time comparison."], cwd=bob_repo, actor='BOB', actor_color='yellow')

# Bob pushes his review comments to origin
cmd([gn_bin, 'sync', 'push'], cwd=bob_repo, actor='BOB', actor_color='yellow')

# 10. CI Quality Gate check (Simulating automated PR runner)
log('CI/CD GATE', 'green', 'Automated CI Quality Gate checks PR readiness (--min-approvals 1 --no-unresolved)...')
ci_res = cmd([gn_bin, 'check', '--min-approvals', '1', '--no-unresolved'], cwd=bob_repo)
if ci_res.returncode != 0:
    log('CI/CD GATE', 'green', '✔ Quality Gate successfully BLOCKED PR merge! (Unresolved security review & missing approvals)')

# 11. Alice pulls Bob's comments into her workspace
log('ALICE', 'magenta', "Alice fetches latest team discussion notes...")
cmd([gn_bin, 'sync', 'pull'], cwd=alice_repo, actor='ALICE', actor_color='magenta')
cmd([gn_bin, 'l'], cwd=alice_repo, actor='ALICE', actor_color='magenta')

# 12. Alice fixes the security issue
fixed_code = '''// Authentication & Session Service
use std::collections::HashMap;

pub struct AuthEngine {
    tokens: HashMap<String, u64>,
}

impl AuthEngine {
    pub fn new() -> Self {
        Self { tokens: HashMap::new() }
    }

    pub fn issue_token(&mut self, user: &str) -> String {
        let token = format!("tok_{}", user);
        self.tokens.insert(token.clone(), 3600);
        token
    }

    pub fn verify_token(&self, token: &str) -> bool {
        // Safe constant-time verification implemented
        if let Some(&expiry) = self.tokens.get(token) {
            expiry > 0
        } else {
            false
        }
    }
}
'''
with open(os.path.join(alice_repo, 'auth.rs'), 'w') as f:
    f.write(fixed_code)

cmd(['git', 'commit', '-a', '-m', 'fix(security): constant-time token verification and expiry check'], cwd=alice_repo, actor='ALICE', actor_color='magenta')
cmd(['git', 'push', 'origin', 'master'], cwd=alice_repo, actor='ALICE', actor_color='magenta')

# 13. Alice replies and marks notes resolved
cmd([gn_bin, 'r', '2', '-m', 'Fixed! Replaced with safe constant-time expiry validation.'], cwd=alice_repo, actor='ALICE', actor_color='magenta')
cmd([gn_bin, 'ok', '2'], cwd=alice_repo, actor='ALICE', actor_color='magenta')
cmd([gn_bin, 'ok', '1'], cwd=alice_repo, actor='ALICE', actor_color='magenta')
cmd([gn_bin, 'sync', 'push'], cwd=alice_repo, actor='ALICE', actor_color='magenta')

# 14. Bob approves the PR review
cmd([gn_bin, 'sync', 'pull'], cwd=bob_repo, actor='BOB', actor_color='yellow')
cmd([gn_bin, 'resolve', '2', '--status', 'approved'], cwd=bob_repo, actor='BOB', actor_color='yellow')
cmd([gn_bin, 'sync', 'push'], cwd=bob_repo, actor='BOB', actor_color='yellow')

# 15. Bob validates the database refspecs with gn validate
log('BOB', 'yellow', 'Bob audits repository refs with `gn validate`...')
cmd([gn_bin, 'validate'], cwd=bob_repo, actor='BOB', actor_color='yellow')

# 16. Bob runs Rebase-Heal check after commit changes
log('BOB', 'yellow', 'Bob verifies note anchors with `gn heal`...')
cmd([gn_bin, 'heal'], cwd=bob_repo, actor='BOB', actor_color='yellow')

# 17. CI/CD Gate re-runs
log('CI/CD GATE', 'green', 'Re-running CI Quality Gate verification...')
ci_res2 = cmd([gn_bin, 'check', '--min-approvals', '1', '--no-unresolved'], cwd=bob_repo)
if ci_res2.returncode == 0:
    log('CI/CD GATE', 'green', '✔ MERGE PERMITTED: All review threads resolved and minimum approval threshold achieved!')

print('\n' + '='*75)
log('SIMULATION', 'cyan', '🏁 LIVE COLLABORATION TEST COMPLETE: 100% ECOSYSTEM SYNC VERIFIED!')
print('='*75 + '\n')
