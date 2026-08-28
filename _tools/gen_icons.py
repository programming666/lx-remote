"""为 LX Remote 生成图标：
- icon.png（主图标，256x256，圆角矩形 + 绿色音符）
- icon.ico（Windows 多尺寸 ICO，包含 16/32/48/64/128/256）
- tray.png（托盘专用，32x32 单色，用于 Windows 托盘）
"""
from PIL import Image, ImageDraw, ImageFont
import os

OUT = os.path.dirname(os.path.abspath(__file__)) + "/../src-tauri/icons"
os.makedirs(OUT, exist_ok=True)

GREEN = (7, 197, 86, 255)        # var(--accent)
GREEN_LIGHT = (24, 210, 122, 255)
WHITE = (255, 255, 255, 255)
BG = (20, 20, 26, 255)


def draw_icon(size: int) -> Image.Image:
    """生成单个 size 的图标（透明背景 + 绿色圆角音符）"""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    # 圆角矩形背景
    r = int(size * 0.22)
    pad = int(size * 0.04)
    d.rounded_rectangle(
        [pad, pad, size - pad, size - pad],
        radius=r,
        fill=BG,
    )

    # 绿色音符（简化的 ♪ 形状）
    cx = size / 2
    cy = size / 2
    s = size / 256.0  # 缩放因子（按 256 设计）

    # 音符杆
    stem_w = max(2, int(8 * s))
    stem_x = cx + int(34 * s)
    stem_top = int(40 * s)
    stem_bot = int(180 * s)
    d.rectangle([stem_x - stem_w // 2, stem_top, stem_x + stem_w // 2, stem_bot], fill=GREEN_LIGHT)

    # 音符头（椭圆）
    head_w = int(48 * s)
    head_h = int(34 * s)
    head_y = int(160 * s)
    d.ellipse(
        [stem_x - head_w, head_y, stem_x + int(8 * s), head_y + head_h],
        fill=GREEN,
    )

    # 旗（顶部的弯钩）
    flag = [
        (stem_x, stem_top),
        (stem_x + int(70 * s), stem_top + int(30 * s)),
        (stem_x + int(70 * s), stem_top + int(70 * s)),
        (stem_x + int(20 * s), stem_top + int(50 * s)),
    ]
    d.polygon(flag, fill=GREEN)

    return img


def main():
    # ---- 主图标 PNG ----
    icon = draw_icon(256)
    icon.save(f"{OUT}/icon.png", "PNG")
    print(f"  -> {OUT}/icon.png")

    # ---- Windows ICO 多分辨率 ----
    sizes = [16, 32, 48, 64, 128, 256]
    ico_images = [draw_icon(s) for s in sizes]
    ico_images[0].save(
        f"{OUT}/icon.ico",
        format="ICO",
        sizes=[(s, s) for s in sizes],
        append_images=ico_images[1:],
    )
    print(f"  -> {OUT}/icon.ico")

    # ---- 托盘图标（白底透明，Windows 托盘习惯单色） ----
    tray_size = 32
    tray = Image.new("RGBA", (tray_size, tray_size), (0, 0, 0, 0))
    td = ImageDraw.Draw(tray)
    # 缩放后的音符
    cx = tray_size / 2
    s = tray_size / 256.0
    stem_x = cx + int(34 * s)
    stem_top = int(40 * s)
    stem_bot = int(180 * s)
    stem_w = max(1, int(8 * s))
    td.rectangle([stem_x - stem_w // 2, stem_top, stem_x + stem_w // 2, stem_bot], fill=WHITE)
    head_w = int(48 * s)
    head_h = int(34 * s)
    head_y = int(160 * s)
    td.ellipse(
        [stem_x - head_w, head_y, stem_x + int(8 * s), head_y + head_h],
        fill=WHITE,
    )
    td.polygon([
        (stem_x, stem_top),
        (stem_x + int(70 * s), stem_top + int(30 * s)),
        (stem_x + int(70 * s), stem_top + int(70 * s)),
        (stem_x + int(20 * s), stem_top + int(50 * s)),
    ], fill=WHITE)
    tray.save(f"{OUT}/tray.png", "PNG")
    print(f"  -> {OUT}/tray.png")

    # ---- macOS icns（多分辨率 PNG 序列，tauri 用 png 也行） ----
    icon.save(f"{OUT}/icon@2x.png", "PNG")  # 512x512 不需要，保留 256
    print(f"  -> {OUT}/icon@2x.png")


if __name__ == "__main__":
    main()
    print("Done.")