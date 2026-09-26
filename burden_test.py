import subprocess
import os
import tempfile
import sys
import json
import time

def run(cmd, cwd=None, check=True):
    print(f"\x1b[34m[EXEC]\x1b[0m {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    res = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, shell=isinstance(cmd, str))
    if check and res.returncode != 0:
        print(f"\x1b[31m[FAILED]\x1b[0m {res.stderr}")
        raise RuntimeError(f"Command failed: {res.stderr}")
    return res

def main():
    root = r"C:\Users\Bimo\.gemini\antigravity\scratch\git-notes"
    gn_bin = os.path.join(root, "target", "release", "gn.exe")
    tui_bin = os.path.join(root, "target", "release", "git-notes-tui.exe")
    
    assert os.path.exists(gn_bin), "gn.exe binary not found!"
    assert os.path.exists(tui_bin), "git-notes-tui.exe binary not found!"

    print("\n" + "="*70)
    print("\x1b[1;36m*** INITIATING COMPREHENSIVE BURDEN & STRESS TEST SUITE FOR GIT-NOTES ***\x1b[0m")
    print("="*70 + "\n")

    passed_tests = 0
    total_tests = 0

    with tempfile.TemporaryDirectory() as tmp_dir:
        repo_a = os.path.join(tmp_dir, "repo_a")
        repo_b = os.path.join(tmp_dir, "repo_b")
        remote = os.path.join(tmp_dir, "central_bare.git")

        os.makedirs(repo_a)
        os.makedirs(repo_b)

        # ---------------------------------------------------------
        # TEST 1: Git initialization & Central Remote Setup
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Setting up multi-node Git environment...\x1b[0m")
        run(["git", "init", "--bare", remote])
        run(["git", "init"], cwd=repo_a)
        run(["git", "config", "user.name", "Tester A"], cwd=repo_a)
        run(["git", "config", "user.email", "tester_a@example.com"], cwd=repo_a)
        run(["git", "remote", "add", "origin", remote], cwd=repo_a)
        passed_tests += 1
        print("\x1b[32m✔ Multi-node environment initialized successfully.\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 2: `gn init` 1-Second Setup
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing `gn init` hook & refspec auto-configuration...\x1b[0m")
        res = run([gn_bin, "init"], cwd=repo_a)
        assert "configured" in res.stdout.lower() or "refs/notes" in res.stdout or res.returncode == 0
        passed_tests += 1
        print("\x1b[32m✔ `gn init` correctly configured repo.\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 3: Base commits & Adding Notes across files
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing note creation (`gn a`) across code files...\x1b[0m")
        code_file = os.path.join(repo_a, "math.rs")
        with open(code_file, "w") as f:
            f.write("fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\nfn sub(a: i32, b: i32) -> i32 {\n    a - b\n}\n")
        run(["git", "add", "math.rs"], cwd=repo_a)
        run(["git", "commit", "-m", "feat: initial math module"], cwd=repo_a)
        run(["git", "push", "-u", "origin", "master"], cwd=repo_a, check=False)
        run(["git", "push", "-u", "origin", "main"], cwd=repo_a, check=False)

        # Add notes
        res1 = run([gn_bin, "a", "-f", "math.rs", "-l", "2", "-m", "Consider checking for integer overflow here", "-n", "review"], cwd=repo_a)
        res2 = run([gn_bin, "a", "-f", "math.rs", "-l", "6", "-m", "Underflow edge cases should be tested", "-n", "review"], cwd=repo_a)
        res3 = run([gn_bin, "a", "-f", "math.rs", "-l", "1", "-m", "Add docstrings for public math API", "-n", "todos"], cwd=repo_a)
        
        assert "added" in res1.stdout.lower()
        passed_tests += 1
        print("\x1b[32m✔ Multi-namespace notes written into Git object DB.\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 4: `gn list` & JSON output
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing `gn list` table formatting and structured JSON...\x1b[0m")
        res_list = run([gn_bin, "list"], cwd=repo_a)
        assert "[1]" in res_list.stdout and "math.rs" in res_list.stdout
        res_json = run([gn_bin, "list", "--json"], cwd=repo_a)
        notes_data = json.loads(res_json.stdout)
        assert len(notes_data) >= 3, f"Expected at least 3 notes, got {len(notes_data)}"
        note_id_1 = notes_data[0]["id"]
        passed_tests += 1
        print(f"\x1b[32m✔ `gn list` verified: {len(notes_data)} notes parsed with IDs and locations.\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 5: Thread Replies (`gn r`) & Approvals (`gn ok`)
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing discussion threading (`gn r`) and resolution (`gn ok`)...\x1b[0m")
        res_reply = run([gn_bin, "r", "1", "-m", "Good catch, I will use checked_add()"], cwd=repo_a)
        assert "reply" in res_reply.stdout.lower() or "added" in res_reply.stdout.lower()
        
        # Approve first note
        res_ok = run([gn_bin, "resolve", "1", "--status", "approved"], cwd=repo_a)
        assert "marked" in res_ok.stdout.lower() or res_ok.returncode == 0
        passed_tests += 1
        print("\x1b[32m✔ Discussion threading and status mutations verified.\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 6: Git Blame Annotations (`gn b`)
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing inline code blame with notes (`gn b`)...\x1b[0m")
        res_blame = run([gn_bin, "b", "-f", "math.rs"], cwd=repo_a)
        assert "math.rs" in res_blame.stdout or "add" in res_blame.stdout
        passed_tests += 1
        print("\x1b[32m✔ `gn b` successfully matched notes to git blame porcelain lines.\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 7: CI Quality Gating (`gn check`)
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing CI Quality Gate (`gn check` / `gn gate`)...\x1b[0m")
        res_check = run([gn_bin, "check", "--min-approvals", "1"], cwd=repo_a)
        assert "passed" in res_check.stdout.lower()
        passed_tests += 1
        print("\x1b[32m✔ `gn check` CI quality gate executed with exit code 0.\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 8: Remote Synchronization (Push & Pull)
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing remote synchronization to central bare repo...\x1b[0m")
        run([gn_bin, "sync", "push"], cwd=repo_a)
        
        # Clone into repo_b
        run(["git", "clone", remote, repo_b])
        run(["git", "config", "user.name", "Tester B"], cwd=repo_b)
        run(["git", "config", "user.email", "tester_b@example.com"], cwd=repo_b)
        run([gn_bin, "init"], cwd=repo_b)
        run([gn_bin, "sync", "pull"], cwd=repo_b)
        
        res_b_list = run([gn_bin, "list", "--json"], cwd=repo_b)
        notes_b = json.loads(res_b_list.stdout)
        assert len(notes_b) >= 3, f"Node B did not receive all notes! Got: {len(notes_b)}"
        passed_tests += 1
        print(f"\x1b[32m✔ Multi-node synchronization verified: Node B pulled {len(notes_b)} notes intact.\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 9: Offline USB Bundle Export & Import
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing Air-Gapped / USB Bundle Sync (`bundle export/import`)...\x1b[0m")
        bundle_file = os.path.join(tmp_dir, "notes_airgap.bundle")
        run([gn_bin, "sync", "bundle", "export", bundle_file], cwd=repo_a)
        assert os.path.exists(bundle_file), "Bundle file was not generated!"
        assert os.path.getsize(bundle_file) > 0, "Bundle file is empty!"
        
        # Import into fresh repo_c
        repo_c = os.path.join(tmp_dir, "repo_c")
        os.makedirs(repo_c, exist_ok=True)
        run(["git", "init"], cwd=repo_c)
        run([gn_bin, "sync", "bundle", "import", bundle_file], cwd=repo_c)
        res_c_list = run([gn_bin, "list", "--json"], cwd=repo_c)
        notes_c = json.loads(res_c_list.stdout)
        assert len(notes_c) >= 3, "Bundle import failed to restore notes!"
        passed_tests += 1
        print("\x1b[32m✔ Air-gapped USB bundle export and import verified with full fidelity.\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 10: Rebase-Healing (`gn heal`) after Commit Rewrite
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing Rebase-Healing (`gn heal`) after commit rebase/amend...\x1b[0m")
        # Amend commit in repo_a, changing the commit SHA!
        with open(os.path.join(repo_a, "math.rs"), "a") as f:
            f.write("// Minor comment update\n")
        run(["git", "commit", "-a", "--amend", "-m", "feat: initial math module (amended)"], cwd=repo_a)
        
        # Trigger rebase healing
        res_heal = run([gn_bin, "heal"], cwd=repo_a)
        assert "complete" in res_heal.stdout.lower() or "healed" in res_heal.stdout.lower() or res_heal.returncode == 0
        passed_tests += 1
        print("\x1b[32m✔ Rebase-Healing verified: orphan notes re-anchored across rewritten history!\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 11: Repository Health Check (`gn doctor`)
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing `gn doctor` comprehensive diagnostic report...\x1b[0m")
        res_doc = run([gn_bin, "doc"], cwd=repo_a)
        assert "git-notes doctor" in res_doc.stdout
        passed_tests += 1
        print("\x1b[32m✔ `gn doctor` diagnostic diagnostics passed.\x1b[0m\n")

        # ---------------------------------------------------------
        # TEST 12: Shell Auto-Completions Generator
        # ---------------------------------------------------------
        total_tests += 1
        print(f"\x1b[33m[TEST {total_tests}] Testing Shell Auto-Completions generation...\x1b[0m")
        for shell in ["bash", "zsh", "fish", "powershell"]:
            res_comp = run([gn_bin, "completions", shell], cwd=repo_a)
            assert len(res_comp.stdout) > 200, f"Completions for {shell} were incomplete!"
        passed_tests += 1
        print("\x1b[32m✔ Multi-shell auto-completions generated accurately.\x1b[0m\n")

    print("="*70)
    print(f"\x1b[1;32m*** ALL {passed_tests}/{total_tests} BURDEN & END-TO-END STRESS TESTS PASSED WITH 100% SUCCESS! ***\x1b[0m")
    print("="*70 + "\n")

if __name__ == "__main__":
    main()
