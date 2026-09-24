import os
from PIL import Image, ImageDraw, ImageFont

# Crisp dimensions (HiDPI 2x scale for tack-sharp text rendering)
SCALE = 2
BASE_WIDTH = 960
BASE_HEIGHT = 560
WIDTH = BASE_WIDTH * SCALE
HEIGHT = BASE_HEIGHT * SCALE

font_mono = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 15 * SCALE)
font_bold = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 15 * SCALE)
font_title = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 14 * SCALE)
font_sub = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 13 * SCALE)

def create_terminal_window():
    # Crisp deep dark background
    img = Image.new("RGBA", (WIDTH, HEIGHT), (11, 14, 19, 255))
    draw = ImageDraw.Draw(img)

    margin = 16 * SCALE
    # Main terminal window box with subtle border
    draw.rounded_rectangle(
        (margin, margin, WIDTH - margin, HEIGHT - margin),
        radius=14 * SCALE,
        fill=(18, 22, 29, 255),
        outline=(45, 55, 72, 255),
        width=1 * SCALE,
    )

    # Title bar
    header_h = 44 * SCALE
    draw.rounded_rectangle(
        (margin, margin, WIDTH - margin, margin + header_h),
        radius=14 * SCALE,
        fill=(26, 32, 44, 255),
    )
    draw.rectangle(
        (margin, margin + header_h - 10 * SCALE, WIDTH - margin, margin + header_h),
        fill=(26, 32, 44, 255),
    )
    # Subtle separator under titlebar
    draw.line(
        [(margin, margin + header_h), (WIDTH - margin, margin + header_h)],
        fill=(45, 55, 72, 255),
        width=1 * SCALE,
    )

    # Window traffic lights
    dot_y = margin + 22 * SCALE
    draw.ellipse((margin + 18 * SCALE, dot_y - 6 * SCALE, margin + 30 * SCALE, dot_y + 6 * SCALE), fill=(248, 81, 73))
    draw.ellipse((margin + 38 * SCALE, dot_y - 6 * SCALE, margin + 50 * SCALE, dot_y + 6 * SCALE), fill=(227, 179, 65))
    draw.ellipse((margin + 58 * SCALE, dot_y - 6 * SCALE, margin + 70 * SCALE, dot_y + 6 * SCALE), fill=(46, 160, 67))

    # Centered Title
    title = "git-notes — decentralized code review"
    bbox = font_title.getbbox(title)
    tw = bbox[2] - bbox[0]
    draw.text(((WIDTH - tw) // 2, margin + 12 * SCALE), title, fill=(160, 174, 192), font=font_title)

    return img, draw

frames = []
durations = []

def add_frame(draw_func, duration=120):
    img, draw = create_terminal_window()
    draw_func(draw)
    # Downsample cleanly with LANCZOS to 960x560 for tack-sharp crisp fonts
    final_img = img.resize((BASE_WIDTH, BASE_HEIGHT), Image.Resampling.LANCZOS)
    # Convert using adaptive palette to eliminate color banding or fuzziness
    paletted = final_img.convert("RGB").convert("P", palette=Image.Palette.ADAPTIVE, colors=256)
    frames.append(paletted)
    durations.append(duration)

PAD_X = 36 * SCALE
PAD_Y = 82 * SCALE
LINE_H = 28 * SCALE

# Phase 1: Clean Screen -> Add Note
cmd1 = 'git-notes add -f src/auth.rs -l 42 -m "Security: validate JWT expiry before parsing"'
typing_steps = [8, 18, 30, 44, 58, 72, len(cmd1)]

for count in typing_steps:
    t = cmd1[:count]
    def make_step(text):
        def d(draw):
            draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
            draw.text((PAD_X + 22 * SCALE, PAD_Y), text, fill=(240, 246, 252), font=font_mono)
            tb = font_mono.getbbox(text)
            cx = PAD_X + 22 * SCALE + (tb[2] - tb[0])
            draw.rectangle((cx + 2 * SCALE, PAD_Y, cx + 11 * SCALE, PAD_Y + 18 * SCALE), fill=(160, 174, 192))
        return d
    add_frame(make_step(t), duration=100)

# Output for cmd1
def draw_cmd1_complete(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd1, fill=(240, 246, 252), font=font_mono)

    y1 = PAD_Y + LINE_H + 4 * SCALE
    draw.text((PAD_X, y1), "✔ Note 7c9a4e21 written to refs/notes/comments", fill=(74, 222, 128), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, y1 + LINE_H), "Anchored to commit 8d31ef2  |  File: src/auth.rs:42", fill=(148, 163, 184), font=font_mono)
    draw.text((PAD_X + 22 * SCALE, y1 + 2 * LINE_H), "Author: Bimo <git-notes@open-source.dev>", fill=(148, 163, 184), font=font_mono)

    # Next prompt cursor
    y_next = y1 + 3 * LINE_H + 8 * SCALE
    draw.text((PAD_X, y_next), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.rectangle((PAD_X + 22 * SCALE, y_next, PAD_X + 31 * SCALE, y_next + 18 * SCALE), fill=(160, 174, 192))

add_frame(draw_cmd1_complete, duration=1500)

# Phase 2: Clear & Run git-notes list
cmd2 = "git-notes list"
def draw_cmd2_typed(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd2, fill=(240, 246, 252), font=font_mono)
add_frame(draw_cmd2_typed, duration=400)

def draw_cmd2_table(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd2, fill=(240, 246, 252), font=font_mono)

    table_data = [
        ("┌──────────┬────────────────┬────────┬──────────┬───────────────────────────────────────────┐", (100, 116, 139)),
        ("│ ID       │ LOCATION       │ AUTHOR │ STATUS   │ NOTE CONTENT                              │", (56, 189, 248)),
        ("├──────────┼────────────────┼────────┼──────────┼───────────────────────────────────────────┤", (100, 116, 139)),
        ("│ 7c9a4e21 │ src/auth.rs:42 │ Bimo   │ Open     │ Security: validate JWT expiry before par… │", (241, 245, 249)),
        ("│ 3e8b091f │ src/jwt.rs:18  │ Alice  │ Approved │ Add clock skew tolerance for RFC 7519     │", (148, 163, 184)),
        ("│ d21c448a │ src/db.rs:104  │ Bimo   │ Resolved │ Connection pooling retry logic looks good │", (148, 163, 184)),
        ("└──────────┴────────────────┴────────┴──────────┴───────────────────────────────────────────┘", (100, 116, 139)),
    ]
    cur_y = PAD_Y + LINE_H + 2 * SCALE
    for line_text, color in table_data:
        draw.text((PAD_X, cur_y), line_text, fill=color, font=font_mono)
        cur_y += LINE_H - 4 * SCALE

    # Prompt
    draw.text((PAD_X, cur_y + 12 * SCALE), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.rectangle((PAD_X + 22 * SCALE, cur_y + 12 * SCALE, PAD_X + 31 * SCALE, cur_y + 30 * SCALE), fill=(160, 174, 192))

add_frame(draw_cmd2_table, duration=2200)

# Phase 3: Interactive TUI
cmd3 = "git-notes-tui"
def draw_cmd3_prompt(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd3, fill=(240, 246, 252), font=font_mono)
add_frame(draw_cmd3_prompt, duration=450)

def draw_tui(draw):
    # TUI Header banner
    draw.rectangle((PAD_X - 10 * SCALE, PAD_Y - 12 * SCALE, WIDTH - PAD_X + 10 * SCALE, PAD_Y + 18 * SCALE), fill=(30, 41, 59, 255))
    draw.text((PAD_X, PAD_Y - 8 * SCALE), "git-notes TUI  ──  Diff: feature/jwt-auth (commit 8d31ef2)", fill=(56, 189, 248), font=font_bold)

    # Split screen layout
    split_w = (WIDTH - 2 * PAD_X - 20 * SCALE) // 2
    box_h = 360 * SCALE
    top_y = PAD_Y + 30 * SCALE

    # Left: Code Diff Panel
    draw.rounded_rectangle((PAD_X, top_y, PAD_X + split_w, top_y + box_h), radius=8 * SCALE, outline=(71, 85, 105), width=1 * SCALE)
    draw.text((PAD_X + 16 * SCALE, top_y + 12 * SCALE), "src/auth.rs", fill=(241, 245, 249), font=font_bold)

    code_lines = [
        (" 39   pub fn verify(header: &str) -> Result<Token> {", (148, 163, 184)),
        (" 40       let token = header.strip_prefix(\"Bearer \")?;", (148, 163, 184)),
        ("+41 >     if token.is_expired() {", (74, 222, 128)),
        ("+42 >         return Err(AuthError::Expired);", (251, 146, 60)),
        ("+43 >     }", (74, 222, 128)),
        (" 44       let claims = decode_claims(token)?;", (148, 163, 184)),
        (" 45       Ok(claims)", (148, 163, 184)),
        (" 46   }", (148, 163, 184)),
    ]
    cy = top_y + 44 * SCALE
    for c_line, color in code_lines:
        if ">" in c_line:
            draw.rectangle((PAD_X + 6 * SCALE, cy - 2 * SCALE, PAD_X + split_w - 6 * SCALE, cy + 22 * SCALE), fill=(56, 189, 248, 32))
        draw.text((PAD_X + 16 * SCALE, cy), c_line, fill=color, font=font_mono)
        cy += 26 * SCALE

    # Right: Discussion Thread Panel
    rx = PAD_X + split_w + 20 * SCALE
    draw.rounded_rectangle((rx, top_y, rx + split_w, top_y + box_h), radius=8 * SCALE, outline=(56, 189, 248), width=1 * SCALE)
    draw.text((rx + 16 * SCALE, top_y + 12 * SCALE), "Thread #7c9a4e21  [Open]", fill=(56, 189, 248), font=font_bold)

    thread_items = [
        ("👤 Bimo (Author) ── 2 mins ago", (56, 189, 248)),
        ("   \"Security: validate JWT expiry before parsing\"", (241, 245, 249)),
        ("", (0,0,0)),
        ("↳ 👤 Alice ── 1 min ago", (168, 85, 247)),
        ("   \"Make sure clock skew tolerance is set to", (226, 232, 240)),
        ("    30 seconds as per RFC 7519.\"", (226, 232, 240)),
        ("", (0,0,0)),
        ("↳ 👤 Bimo ── just now", (74, 222, 128)),
        ("   \"Updated with 30s leeway!\"", (74, 222, 128)),
    ]
    ty = top_y + 44 * SCALE
    for t_line, color in thread_items:
        draw.text((rx + 16 * SCALE, ty), t_line, fill=color, font=font_mono)
        ty += 24 * SCALE

    # Bottom shortcut bar
    by = top_y + box_h + 12 * SCALE
    draw.rounded_rectangle((PAD_X, by, WIDTH - PAD_X, by + 34 * SCALE), radius=6 * SCALE, fill=(30, 41, 59))
    draw.text((PAD_X + 16 * SCALE, by + 8 * SCALE), "[r] Reply    [a] Approve    [x] Resolve    [n] Next Note    [q] Quit", fill=(148, 163, 184), font=font_bold)

add_frame(draw_tui, duration=3200)

# Phase 4: Sync to Remote & GitHub PR
cmd4 = "git-notes sync push origin"
def draw_cmd4_prompt(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd4, fill=(240, 246, 252), font=font_mono)
add_frame(draw_cmd4_prompt, duration=450)

def draw_cmd4_result(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd4, fill=(240, 246, 252), font=font_mono)

    y4 = PAD_Y + LINE_H + 4 * SCALE
    draw.text((PAD_X, y4), "✔ Fetching remote notes from origin...", fill=(148, 163, 184), font=font_mono)
    draw.text((PAD_X, y4 + LINE_H), "✔ Merged 1 new remote note (strategy: LWW + Union)", fill=(74, 222, 128), font=font_mono)
    draw.text((PAD_X, y4 + 2 * LINE_H), "✔ Pushing refs/notes/comments -> origin (1 note, 2 replies)", fill=(74, 222, 128), font=font_bold)
    draw.text((PAD_X, y4 + 3 * LINE_H + 6 * SCALE), "✔ GitHub Bridge: Synced 3 comments to PR #14 inline discussion", fill=(56, 189, 248), font=font_bold)

    # Clean final prompt
    fin_y = y4 + 4 * LINE_H + 16 * SCALE
    draw.text((PAD_X, fin_y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.rectangle((PAD_X + 22 * SCALE, fin_y, PAD_X + 31 * SCALE, fin_y + 18 * SCALE), fill=(160, 174, 192))

add_frame(draw_cmd4_result, duration=3500)

print(f"Generated {len(frames)} crisp frames.")
# Save clean animated GIF without fuzzy artifacts or text doubling
frames[0].save(
    "assets/demo.gif",
    save_all=True,
    append_images=frames[1:],
    duration=durations,
    loop=0,
    disposal=2,  # Clear each frame to background so text NEVER doubles up
)
print("Saved razor-sharp assets/demo.gif successfully!")
