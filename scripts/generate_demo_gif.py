import os
from PIL import Image, ImageDraw, ImageFont

WIDTH = 920
HEIGHT = 520

font_mono = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 16)
font_bold = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 16)

def create_window():
    img = Image.new("RGBA", (WIDTH, HEIGHT), (13, 17, 23, 255))
    draw = ImageDraw.Draw(img)
    # Background terminal window
    draw.rounded_rectangle((10, 10, WIDTH - 10, HEIGHT - 10), radius=12, fill=(22, 27, 34), outline=(48, 54, 61), width=1)
    # Window header
    draw.rounded_rectangle((10, 10, WIDTH - 10, 48), radius=12, fill=(33, 38, 45))
    draw.rectangle((10, 36, WIDTH - 10, 48), fill=(33, 38, 45))
    # Window buttons
    draw.ellipse((26, 23, 38, 35), fill=(248, 81, 73))
    draw.ellipse((46, 23, 58, 35), fill=(227, 179, 65))
    draw.ellipse((66, 23, 78, 35), fill=(46, 160, 67))
    # Title
    draw.text((WIDTH // 2 - 90, 20), "git-notes — zsh", fill=(139, 148, 158), font=font_bold)
    return img, draw

# Frames list
frames = []
durations = []

def add_frame(draw_func, duration=150):
    img, draw = create_window()
    draw_func(draw)
    frames.append(img.convert("RGB"))
    durations.append(duration)

# Scenario script
cmd1 = "git-notes add -f src/auth.rs -l 42 -m \"Validate token expiry before parsing claims\""
cmd2 = "git-notes list"
cmd3 = "git-notes-tui"
cmd4 = "git-notes sync push origin"

# Phase 1: Typing command 1
for i in range(1, len(cmd1) + 1, 4):
    typed = cmd1[:i]
    def make_draw(t):
        def d(draw):
            draw.text((30, 65), "❯ ", fill=(56, 189, 248), font=font_bold)
            draw.text((50, 65), t, fill=(240, 246, 252), font=font_mono)
            # cursor
            bbox = font_mono.getbbox(t)
            cx = 50 + (bbox[2] - bbox[0])
            draw.rectangle((cx + 2, 65, cx + 10, 81), fill=(139, 148, 158))
        return d
    add_frame(make_draw(typed), duration=80)

# Phase 2: Command 1 Output
def draw_cmd1_out(draw):
    draw.text((30, 65), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((50, 65), cmd1, fill=(240, 246, 252), font=font_mono)
    draw.text((30, 95), "✔ Note 7c9a4e21 added to refs/notes/comments", fill=(63, 185, 80), font=font_bold)
    draw.text((30, 120), "  Anchored to commit 8d31ef2 (src/auth.rs:42)", fill=(139, 148, 158), font=font_mono)
    draw.text((30, 145), "  Author: Bimo <git-notes@open-source.dev>", fill=(139, 148, 158), font=font_mono)
    draw.text((30, 180), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.rectangle((50, 180, 58, 196), fill=(139, 148, 158))

add_frame(draw_cmd1_out, duration=1400)

# Phase 3: Typing command 2 (git-notes list)
for i in range(1, len(cmd2) + 1, 3):
    typed2 = cmd2[:i]
    def make_draw2(t):
        def d(draw):
            draw.text((30, 65), "❯ ", fill=(56, 189, 248), font=font_bold)
            draw.text((50, 65), cmd1, fill=(240, 246, 252), font=font_mono)
            draw.text((30, 95), "✔ Note 7c9a4e21 added to refs/notes/comments", fill=(63, 185, 80), font=font_bold)
            draw.text((30, 120), "  Anchored to commit 8d31ef2 (src/auth.rs:42)", fill=(139, 148, 158), font=font_mono)
            draw.text((30, 145), "  Author: Bimo <git-notes@open-source.dev>", fill=(139, 148, 158), font=font_mono)
            draw.text((30, 180), "❯ ", fill=(56, 189, 248), font=font_bold)
            draw.text((50, 180), t, fill=(240, 246, 252), font=font_mono)
            bbox = font_mono.getbbox(t)
            cx = 50 + (bbox[2] - bbox[0])
            draw.rectangle((cx + 2, 180, cx + 10, 196), fill=(139, 148, 158))
        return d
    add_frame(make_draw2(typed2), duration=90)

# Phase 4: Command 2 Output (Table)
def draw_cmd2_out(draw):
    draw.text((30, 65), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((50, 65), cmd1, fill=(240, 246, 252), font=font_mono)
    draw.text((30, 95), "✔ Note 7c9a4e21 added to refs/notes/comments", fill=(63, 185, 80), font=font_bold)
    draw.text((30, 130), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((50, 130), "git-notes list", fill=(240, 246, 252), font=font_mono)
    
    table_lines = [
        ("┌──────────┬────────────────┬────────┬──────────┬──────────────────────────────────────────┐", (139, 148, 158)),
        ("│ ID       │ LOCATION       │ AUTHOR │ STATUS   │ NOTE CONTENT                             │", (88, 166, 255)),
        ("├──────────┼────────────────┼────────┼──────────┼──────────────────────────────────────────┤", (139, 148, 158)),
        ("│ 7c9a4e21 │ src/auth.rs:42 │ Bimo   │ Open     │ Validate token expiry before parsing cl… │", (240, 246, 252)),
        ("│ d3f18a09 │ src/jwt.rs:18  │ Alice  │ Resolved │ Add unit test for malformed signature    │", (139, 148, 158)),
        ("└──────────┴────────────────┴────────┴──────────┴──────────────────────────────────────────┘", (139, 148, 158)),
    ]
    y = 160
    for line, color in table_lines:
        draw.text((30, y), line, fill=color, font=font_mono)
        y += 22
    
    draw.text((30, y + 15), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.rectangle((50, y + 15, 58, y + 31), fill=(139, 148, 158))

add_frame(draw_cmd2_out, duration=1600)

# Phase 5: Launching TUI (git-notes-tui)
def draw_launch_tui(draw):
    draw.text((30, 65), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((50, 65), "git-notes-tui", fill=(240, 246, 252), font=font_mono)
    draw.text((30, 95), "Loading TUI diff viewer & comment threads...", fill=(139, 148, 158), font=font_mono)

add_frame(draw_launch_tui, duration=600)

# Phase 6: TUI Interface View
def draw_tui_view(draw):
    # Header bar
    draw.rectangle((10, 48, WIDTH - 10, 75), fill=(22, 27, 34))
    draw.text((30, 52), "git-notes TUI  ──  Diff: feature/jwt-auth (commit 8d31ef2)", fill=(88, 166, 255), font=font_bold)
    
    # Left Box: Diff Viewer
    lx, ly, lw, lh = 25, 85, 430, 370
    draw.rounded_rectangle((lx, ly, lx + lw, ly + lh), radius=6, outline=(48, 54, 61), width=1)
    draw.text((lx + 15, ly + 10), "src/auth.rs", fill=(240, 246, 252), font=font_bold)
    
    code = [
        (" 39   pub fn verify(header: &str) -> Result<Token> {", (139, 148, 158)),
        (" 40       let token = header.strip_prefix(\"Bearer \")?;", (139, 148, 158)),
        ("+41 >     if token.is_expired() {", (63, 185, 80)),
        ("+42 >         return Err(AuthError::Expired);", (240, 136, 62)),
        ("+43 >     }", (63, 185, 80)),
        (" 44       let claims = decode_claims(token)?;", (139, 148, 158)),
        (" 45       Ok(claims)", (139, 148, 158)),
        (" 46   }", (139, 148, 158)),
    ]
    cy = ly + 40
    for line, color in code:
        if ">" in line:
            draw.rectangle((lx + 5, cy - 2, lx + lw - 5, cy + 20), fill=(56, 189, 248, 25))
        draw.text((lx + 15, cy), line, fill=color, font=font_mono)
        cy += 24
        
    # Right Box: Discussion Thread
    rx, ry, rw, rh = 465, 85, 430, 370
    draw.rounded_rectangle((rx, ry, rx + rw, ry + rh), radius=6, outline=(56, 189, 248), width=1)
    draw.text((rx + 15, ry + 10), "Thread #7c9a4e21  [Open]", fill=(56, 189, 248), font=font_bold)
    
    thread = [
        ("👤 Bimo (Author) ── 2 mins ago", (88, 166, 255)),
        ("   \"Validate token expiry before parsing claims\"", (240, 246, 252)),
        ("", (0,0,0)),
        ("↳ 👤 Alice ── 1 min ago", (168, 85, 247)),
        ("   \"Good catch! Make sure clock skew tolerance", (230, 237, 243)),
        ("    is set to 30s as per RFC 7519.\"", (230, 237, 243)),
        ("", (0,0,0)),
        ("↳ 👤 Bimo ── just now", (88, 166, 255)),
        ("   \"Updated with 30s leeway!\"", (63, 185, 80)),
    ]
    ty = ry + 40
    for line, color in thread:
        draw.text((rx + 15, ty), line, fill=color, font=font_mono)
        ty += 22
        
    # Bottom keybinding bar
    draw.rounded_rectangle((25, 465, WIDTH - 25, 495), radius=6, fill=(33, 38, 45))
    draw.text((40, 472), "[r] Reply    [a] Approve    [x] Resolve    [n] Next Note    [q] Quit", fill=(139, 148, 158), font=font_bold)

add_frame(draw_tui_view, duration=2400)

# Phase 7: Syncing to remote (git-notes sync push)
for i in range(1, len(cmd4) + 1, 4):
    typed4 = cmd4[:i]
    def make_draw4(t):
        def d(draw):
            draw.text((30, 65), "❯ ", fill=(56, 189, 248), font=font_bold)
            draw.text((50, 65), t, fill=(240, 246, 252), font=font_mono)
            bbox = font_mono.getbbox(t)
            cx = 50 + (bbox[2] - bbox[0])
            draw.rectangle((cx + 2, 65, cx + 10, 81), fill=(139, 148, 158))
        return d
    add_frame(make_draw4(typed4), duration=80)

# Phase 8: Sync Output & GitHub Bridge
def draw_sync_out(draw):
    draw.text((30, 65), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((50, 65), cmd4, fill=(240, 246, 252), font=font_mono)
    draw.text((30, 95), "✔ Fetching remote notes from origin...", fill=(139, 148, 158), font=font_mono)
    draw.text((30, 120), "✔ Merged with remote (strategy: LWW + Union)", fill=(63, 185, 80), font=font_mono)
    draw.text((30, 145), "✔ Pushing refs/notes/comments -> origin (1 new note, 2 replies)", fill=(63, 185, 80), font=font_bold)
    draw.text((30, 180), "✔ GitHub Bridge: Synced 3 comments to PR #14 inline discussion", fill=(56, 189, 248), font=font_bold)
    draw.text((30, 215), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.rectangle((50, 215, 58, 231), fill=(139, 148, 158))

add_frame(draw_sync_out, duration=2600)

print(f"Total frames: {len(frames)}")
# Optimize & save as animated GIF
frames[0].save(
    "assets/demo.gif",
    save_all=True,
    append_images=frames[1:],
    duration=durations,
    loop=0,
    optimize=True
)
print("Saved assets/demo.gif successfully!")
