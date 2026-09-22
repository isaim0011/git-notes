import os
from PIL import Image, ImageDraw, ImageFont

WIDTH = 1280
HEIGHT = 640

# Create high-res canvas with modern deep dark gradient
img = Image.new("RGBA", (WIDTH, HEIGHT), (13, 17, 23, 255))
draw = ImageDraw.Draw(img)

# Subtle background glow/accent circles
for r in range(300, 0, -10):
    alpha = int(25 * (1 - r / 300))
    draw.ellipse((850 - r, 180 - r, 850 + r, 180 + r), fill=(56, 189, 248, alpha))
    draw.ellipse((200 - r, 450 - r, 200 + r, 450 + r), fill=(168, 85, 247, alpha))

# Fonts
font_title = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 64)
font_subtitle = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 28)
font_mono_bold = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 20)
font_mono = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 17)
font_badge = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 18)

# Paste Logo if available
logo_path = "packages/chrome-ext/icon128.png"
if os.path.exists(logo_path):
    logo = Image.open(logo_path).convert("RGBA").resize((110, 110), Image.Resampling.LANCZOS)
    img.paste(logo, (90, 80), logo)

# Title & Tagline
draw.text((220, 85), "git-notes", fill=(240, 246, 252), font=font_title)
draw.text((225, 160), "Decentralized Code Comments & Line Reviews in Git", fill=(139, 148, 158), font=font_subtitle)

# Pill Badges
badges = [
    ("⚡ Offline-First", (56, 189, 248, 30), (56, 189, 248)),
    ("🔒 Zero History Rewrite", (168, 85, 247, 30), (192, 132, 252)),
    ("🌐 CLI · TUI · VS Code · Chrome", (34, 197, 94, 30), (74, 222, 128)),
    ("🦀 Built with Rust", (249, 115, 22, 30), (251, 146, 60)),
]

bx = 90
by = 220
for text, bg, fg in badges:
    bbox = font_badge.getbbox(text)
    bw = bbox[2] - bbox[0] + 28
    bh = 38
    draw.rounded_rectangle((bx, by, bx + bw, by + bh), radius=8, fill=bg, outline=fg, width=1)
    draw.text((bx + 14, by + 8), text, fill=fg, font=font_badge)
    bx += bw + 14

# Terminal Mockup Box at the bottom
tx = 90
ty = 285
tw = 1100
th = 310

# Window drop shadow / outline
draw.rounded_rectangle((tx, ty, tx + tw, ty + th), radius=12, fill=(22, 27, 34, 240), outline=(48, 54, 61), width=1)

# Titlebar
draw.rounded_rectangle((tx, ty, tx + tw, ty + 40), radius=12, fill=(33, 38, 45, 255))
draw.rectangle((tx, ty + 30, tx + tw, ty + 40), fill=(33, 38, 45, 255))
# Dots
draw.ellipse((tx + 18, ty + 14, tx + 30, ty + 26), fill=(248, 81, 73))
draw.ellipse((tx + 38, ty + 14, tx + 50, ty + 26), fill=(227, 179, 65))
draw.ellipse((tx + 58, ty + 14, tx + 70, ty + 26), fill=(46, 160, 67))
draw.text((tx + 480, ty + 10), "git-notes — interactive review", fill=(139, 148, 158), font=font_mono)

# Terminal content
lines = [
    ("$ git-notes add -f src/auth.rs -l 42 -m \"Security: validate JWT expiry before parsing claims\"", (88, 166, 255)),
    ("  ✔ Note 7c9a4e21 written to refs/notes/comments (anchored to commit e81a9f)", (63, 185, 80)),
    ("", (0,0,0)),
    ("$ git-notes-tui", (88, 166, 255)),
    ("  ┌─ File: src/auth.rs:42 ────────────────────┐ ┌─ Thread #7c9a4e ───────────────────────────────┐", (139, 148, 158)),
    ("  │ 41  let token = header.strip_prefix(...)?;│ │ 👤 Bimo (now)                           [Open] │", (230, 237, 243)),
    ("  │ 42> if token.is_expired() { return Err(); }│ │ \"Security: validate JWT expiry before pars...\"│", (240, 136, 62)),
    ("  │ 43  let claims = decode_claims(token)?;   │ ├──────────────────────────────────────────────┤", (230, 237, 243)),
    ("  │                                           │ │ [r] Reply   [a] Approve   [x] Resolve  [q]Quit│", (121, 192, 255)),
    ("  └───────────────────────────────────────────┘ └──────────────────────────────────────────────┘", (139, 148, 158)),
]

ly = ty + 50
for line, color in lines:
    draw.text((tx + 25, ly), line, fill=color, font=font_mono)
    ly += 25

os.makedirs("assets", exist_ok=True)
img.convert("RGB").save("assets/social-preview.png", "PNG", quality=95)
print("Saved assets/social-preview.png successfully!")
