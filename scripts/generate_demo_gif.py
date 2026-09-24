import os
from PIL import Image, ImageDraw, ImageFont

# Crisp HiDPI 2x scale
SCALE = 2
BASE_WIDTH = 1000
BASE_HEIGHT = 600
WIDTH = BASE_WIDTH * SCALE
HEIGHT = BASE_HEIGHT * SCALE

font_mono = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 15 * SCALE)
font_bold = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 15 * SCALE)
font_title = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 14 * SCALE)
font_badge = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 12 * SCALE)

def create_window():
    img = Image.new("RGBA", (WIDTH, HEIGHT), (11, 14, 19, 255))
    draw = ImageDraw.Draw(img)
    margin = 16 * SCALE

    # Window box
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
    draw.line(
        [(margin, margin + header_h), (WIDTH - margin, margin + header_h)],
        fill=(45, 55, 72, 255),
        width=1 * SCALE,
    )

    # Traffic lights
    dot_y = margin + 22 * SCALE
    draw.ellipse((margin + 18 * SCALE, dot_y - 6 * SCALE, margin + 30 * SCALE, dot_y + 6 * SCALE), fill=(248, 81, 73))
    draw.ellipse((margin + 38 * SCALE, dot_y - 6 * SCALE, margin + 50 * SCALE, dot_y + 6 * SCALE), fill=(227, 179, 65))
    draw.ellipse((margin + 58 * SCALE, dot_y - 6 * SCALE, margin + 70 * SCALE, dot_y + 6 * SCALE), fill=(46, 160, 67))

    title = "git-notes (gn) — quickies & interactive workflow"
    tb = font_title.getbbox(title)
    tw = tb[2] - tb[0]
    draw.text(((WIDTH - tw) // 2, margin + 12 * SCALE), title, fill=(160, 174, 192), font=font_title)

    return img, draw

frames = []
durations = []

def add_frame(draw_func, duration=150):
    img, draw = create_window()
    draw_func(draw)
    final_img = img.resize((BASE_WIDTH, BASE_HEIGHT), Image.Resampling.LANCZOS)
    paletted = final_img.convert("RGB").convert("P", palette=Image.Palette.ADAPTIVE, colors=256)
    frames.append(paletted)
    durations.append(duration)

PAD_X = 36 * SCALE
PAD_Y = 82 * SCALE
LINE_H = 28 * SCALE

# SCENE 1: gn init (1-second setup)
cmd1 = "gn init"
def draw_scene1(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd1, fill=(240, 246, 252), font=font_mono)

    y = PAD_Y + LINE_H + 4 * SCALE
    draw.text((PAD_X, y), "🚀 Initializing git-notes in repository...", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X, y + LINE_H), "  ✔ Configured remote.origin.fetch for refs/notes/*", fill=(74, 222, 128), font=font_mono)
    draw.text((PAD_X, y + 2 * LINE_H), "  ✔ Installed git auto-sync hooks (.git/hooks/post-merge & pre-push)", fill=(74, 222, 128), font=font_mono)
    draw.text((PAD_X, y + 3 * LINE_H + 4 * SCALE), "✨ git-notes initialized successfully! (Ready in 0.05s)", fill=(74, 222, 128), font=font_bold)

    y_next = y + 4 * LINE_H + 12 * SCALE
    draw.text((PAD_X, y_next), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.rectangle((PAD_X + 22 * SCALE, y_next, PAD_X + 31 * SCALE, y_next + 18 * SCALE), fill=(160, 174, 192))

add_frame(draw_scene1, duration=2400)

# SCENE 2: gn a (Fast Quickie Add)
cmd2 = 'gn a -f src/auth.rs -l 42 -m "Security: validate JWT expiry before decoding"'
def draw_scene2(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd2, fill=(240, 246, 252), font=font_mono)

    y = PAD_Y + LINE_H + 4 * SCALE
    draw.text((PAD_X, y), "✔ Note 7c9a4e21 written to refs/notes/comments", fill=(74, 222, 128), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, y + LINE_H), "Anchored to commit 8d31ef2  |  src/auth.rs:42  |  Author: Bimo", fill=(148, 163, 184), font=font_mono)

    y_next = y + 2 * LINE_H + 12 * SCALE
    draw.text((PAD_X, y_next), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.rectangle((PAD_X + 22 * SCALE, y_next, PAD_X + 31 * SCALE, y_next + 18 * SCALE), fill=(160, 174, 192))

add_frame(draw_scene2, duration=2200)

# SCENE 3: gn l (Numbered index table)
cmd3 = "gn l"
def draw_scene3(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd3, fill=(240, 246, 252), font=font_mono)

    table_data = [
        ("┌─────┬──────────┬────────────────┬──────────┬──────────┬───────────────────────────────────────────┐", (100, 116, 139)),
        ("│ #   │ ID       │ LOCATION       │ AUTHOR   │ STATUS   │ NOTE CONTENT                              │", (56, 189, 248)),
        ("├─────┼──────────┼────────────────┼──────────┼──────────┼───────────────────────────────────────────┤", (100, 116, 139)),
        ("│ [1] │ 7c9a4e21 │ src/auth.rs:42 │ Bimo     │ Open     │ Security: validate JWT expiry before dec… │", (241, 245, 249)),
        ("│ [2] │ 3e8b091f │ src/jwt.rs:18  │ Alice    │ Open     │ Add clock skew tolerance for RFC 7519     │", (148, 163, 184)),
        ("│ [3] │ d21c448a │ src/db.rs:104  │ Bob      │ Approved │ Connection pooling retry logic looks good │", (148, 163, 184)),
        ("└─────┴──────────┴────────────────┴──────────┴──────────┴───────────────────────────────────────────┘", (100, 116, 139)),
    ]
    cur_y = PAD_Y + LINE_H + 2 * SCALE
    for line_text, color in table_data:
        draw.text((PAD_X, cur_y), line_text, fill=color, font=font_mono)
        cur_y += LINE_H - 4 * SCALE

    draw.text((PAD_X, cur_y + 6 * SCALE), "Tip: Use numbers to reply or resolve: gn r 1 -m \"...\" or gn ok 1", fill=(100, 116, 139), font=font_mono)

    y_next = cur_y + LINE_H + 10 * SCALE
    draw.text((PAD_X, y_next), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.rectangle((PAD_X + 22 * SCALE, y_next, PAD_X + 31 * SCALE, y_next + 18 * SCALE), fill=(160, 174, 192))

add_frame(draw_scene3, duration=2800)

# SCENE 4: gn s (Interactive Arrow-Key Picker)
cmd4 = "gn s"
def draw_scene4(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd4, fill=(240, 246, 252), font=font_mono)

    y = PAD_Y + LINE_H + 4 * SCALE
    draw.text((PAD_X, y), "? Select note to view: ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 240 * SCALE, y), "(Use ↑/↓ arrows, Enter to select, Esc to cancel)", fill=(100, 116, 139), font=font_mono)

    items = [
        ("  [1] 7c9a4e21 src/auth.rs:42     Bimo         \"Security: validate JWT expiry...\"", (148, 163, 184), False),
        ("❯ [2] 3e8b091f src/jwt.rs:18      Alice        \"Add clock skew tolerance for...\"", (56, 189, 248), True),
        ("  [3] d21c448a src/db.rs:104      Bob          \"Connection pooling retry log...\"", (148, 163, 184), False),
    ]

    iy = y + LINE_H + 4 * SCALE
    for text, color, active in items:
        if active:
            draw.rectangle((PAD_X - 6 * SCALE, iy - 2 * SCALE, WIDTH - PAD_X - 100 * SCALE, iy + 22 * SCALE), fill=(56, 189, 248, 30))
        draw.text((PAD_X, iy), text, fill=color, font=font_mono)
        iy += LINE_H

    # Adaptive learning recommendation bubble
    by = iy + 16 * SCALE
    draw.rounded_rectangle((PAD_X, by, WIDTH - PAD_X - 160 * SCALE, by + 34 * SCALE), radius=6 * SCALE, fill=(30, 41, 59))
    draw.text((PAD_X + 16 * SCALE, by + 8 * SCALE), "💡 Learned Pattern: Alice frequently reviews JWT auth files. Suggesting related threads.", fill=(227, 179, 65), font=font_badge)

add_frame(draw_scene4, duration=3200)

# SCENE 5: gn ok 1 (Fast Resolve) & Adaptive Suggestion
cmd5 = "gn ok 1"
def draw_scene5(draw):
    draw.text((PAD_X, PAD_Y), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, PAD_Y), cmd5, fill=(240, 246, 252), font=font_mono)

    y = PAD_Y + LINE_H + 4 * SCALE
    draw.text((PAD_X, y), "✔ Note 7c9a4e21 marked as Resolved", fill=(74, 222, 128), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, y + LINE_H), "Updated refs/notes/comments (Thread closed)", fill=(148, 163, 184), font=font_mono)

    # Adaptive learning tip
    y_tip = y + 2 * LINE_H + 12 * SCALE
    draw.text((PAD_X, y_tip), "💡 Pro-tip: You resolved all notes on src/auth.rs! Run 'gn push' to sync to remote.", fill=(227, 179, 65), font=font_mono)

    y_next = y_tip + LINE_H + 8 * SCALE
    draw.text((PAD_X, y_next), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.text((PAD_X + 22 * SCALE, y_next), "gn push", fill=(240, 246, 252), font=font_mono)

    y_push = y_next + LINE_H + 4 * SCALE
    draw.text((PAD_X, y_push), "✔ Pushed refs/notes/comments -> origin (1 resolution synced)", fill=(74, 222, 128), font=font_bold)

    y_fin = y_push + LINE_H + 12 * SCALE
    draw.text((PAD_X, y_fin), "❯ ", fill=(56, 189, 248), font=font_bold)
    draw.rectangle((PAD_X + 22 * SCALE, y_fin, PAD_X + 31 * SCALE, y_fin + 18 * SCALE), fill=(160, 174, 192))

add_frame(draw_scene5, duration=3500)

print(f"Generated {len(frames)} showcase frames.")
frames[0].save(
    "assets/demo.gif",
    save_all=True,
    append_images=frames[1:],
    duration=durations,
    loop=0,
    disposal=2,
)
print("Updated assets/demo.gif with complete showcase!")
