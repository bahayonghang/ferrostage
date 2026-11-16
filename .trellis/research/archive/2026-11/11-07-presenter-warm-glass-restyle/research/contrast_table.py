"""Compute the DESIGN.md contrast table from the theme.rs token values.

Usage: python -I contrast_table.py
Composition matches theme::over (per-channel source-over, rounded to u8).
"""



LIGHT = dict(
    field_top=0xF7F2EB, field_bottom=0xEFE6DB, field_alpha=0.88, layer=0xFFFFFF, layer_alpha=0.66,
    layer_solid=0xFBF8F4, raised=0xFFFFFF, segment=0xFFFFFF, well=0xF3EEE7, control=0xEEEBE7,
    control_hover=0xE5E2DE, control_pressed=0xDCD9D5, border=0x8C847A, ink=0x1F1B17,
    body=0x3D3833, muted=0x5F5850, accent=0xD97757, on_accent=0x1F1712,
    accent_hover=0xE0866A, accent_pressed=0xC9694A, accent_text=0xA3492A,
    focus=0xA3492A, accent_wash=0xF7E6DC, warning=0x8A5A12, error=0xA63A32,
    error_wash=0xF8E3DF,
)
DARK = dict(
    field_top=0x1D1B19, field_bottom=0x26221E, field_alpha=0.82, layer=0x2C2824, layer_alpha=0.74,
    layer_solid=0x2A2622, raised=0x38332E, segment=0x524D48, well=0x221F1C, control=0x3B3734,
    control_hover=0x44403D, control_pressed=0x4C4945, border=0x8F877C, ink=0xF5F1EB,
    body=0xE2DBD2, muted=0xC4BCB2, accent=0xD97757, on_accent=0x1F1712,
    accent_hover=0xE0866A, accent_pressed=0xC9694A, accent_text=0xEBA98C,
    focus=0xEBA98C, accent_wash=0x3A2A22, warning=0xE8B774, error=0xFFA5A0,
    error_wash=0x3B2523,
)


def luminance(color):
    def channel(v):
        c = v / 255
        return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = (channel((color >> s) & 255) for s in (16, 8, 0))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a, b):
    a, b = luminance(a), luminance(b)
    return (max(a, b) + 0.05) / (min(a, b) + 0.05)


def over(front, back, alpha):
    out = 0
    for s in (16, 8, 0):
        f, b = (front >> s) & 255, (back >> s) & 255
        out |= round(f * alpha + b * (1 - alpha)) << s
    return out


def worst_native(t, fg):
    """Panel layer over the field over a pure black or white desktop; lowest ratio."""
    panels = [
        over(t["layer"], over(t[edge], desktop, t["field_alpha"]), t["layer_alpha"])
        for edge in ("field_top", "field_bottom")
        for desktop in (0x000000, 0xFFFFFF)
    ]
    return min((contrast(fg, p), p) for p in panels)


ROWS = [
    ("ink", "layer_solid", "正文，实色面板"),
    ("ink", "native", "正文，毛玻璃面板最差合成"),
    ("body", "layer_solid", "次级正文"),
    ("muted", "native", "次要文字，毛玻璃面板最差合成"),
    ("muted", "control_pressed", "按下的中性控件上的次要文字"),
    ("on_accent", "accent", "主按钮文字"),
    ("on_accent", "accent_pressed", "主按钮按下"),
    (0xFFFFFF, "accent", "白字压橙色，禁止"),
    ("accent_text", "accent_wash", "橙色文字压选中洗底"),
    ("accent_text", "control", "橙色文字压中性控件"),
    ("focus", "native", "焦点环与选中标记，毛玻璃面板最差合成"),
    ("focus", "control_pressed", "焦点环压最深中性填充"),
    ("focus", "segment", "选中分段上的选中标记"),
    ("ink", "segment", "选中分段文字"),
    ("border", "well", "输入框边界"),
    ("error", "error_wash", "停止/危险文字"),
    ("warning", "layer_solid", "警告文字"),
]


def hexs(v):
    return f"#{v:06X}"


def main():
    import sys
    sys.stdout.reconfigure(encoding="utf-8")
    print("| 前景 | 背景 | 浅色 | 深色 | 用途 |")
    print("| --- | --- | --- | --- | --- |")
    for fg, bg, use in ROWS:
        cells = []
        for t in (LIGHT, DARK):
            f = fg if isinstance(fg, int) else t[fg]
            if bg == "native":
                ratio, panel = worst_native(t, f)
                cells.append(f"{ratio:.2f} (`{hexs(f)}` / `{hexs(panel)}`)")
            else:
                cells.append(f"{contrast(f, t[bg]):.2f} (`{hexs(f)}` / `{hexs(t[bg])}`)")
        fg_name = hexs(fg) if isinstance(fg, int) else f"`{fg}`"
        bg_name = "毛玻璃合成" if bg == "native" else f"`{bg}`"
        print(f"| {fg_name} | {bg_name} | {cells[0]} | {cells[1]} | {use} |")


if __name__ == "__main__":
    main()
