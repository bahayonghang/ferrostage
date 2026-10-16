"""Write and optionally compile the 100-page Beamer performance fixture.

The deck theme stays in the thesis slides directory. XeLaTeX is run with that
directory as its working directory and `-output-directory` pointed at the cache,
so the style files resolve as local inputs and build files stay out of the
theme tree. This script does not copy those style files, thesis sentences, or
logo bitmaps. Slide text and the five note fields are original fixture copy.
Notes use Beamer's right-hand second screen, so each physical page is the 16:9
slide plus its notes.

Twelve content slides in each of seven chapters, plus the seven chapter
openers, cover, contents, and thanks, is 94 pages. The acceptance target is
100, so chapters 1-6 have one extra content slide.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]
DEFAULT_DECK = Path(r"D:\Documents\LYH\200-Learning\00博士毕业\毕业论文\thesis\slides\deck")
PDF_NAME = "perf-100.pdf"
CONTENT_COUNTS = (13, 13, 13, 13, 13, 13, 12)
LAYOUTS = ("bullets", "cards", "equation", "table")
CHAPTERS = (
    ("夹具背景与范围", "process", "范围", "步骤"),
    ("分页测试框架", "layers", "分页", "顺序"),
    ("页面正文生成", "data", "正文", "差异"),
    ("讲稿对齐检查", "sense", "讲稿", "页号"),
    ("翻页性能路径", "forecast", "翻页", "耗时"),
    ("缓存与预取", "optimize", "缓存", "邻页"),
    ("夹具结论", "award", "核对", "收束"),
)
SAY_BITS = (
    "这一页只检查渲染，不承载论文原句。",
    "左侧保留十六比九幻灯片，右侧放下五栏讲稿。",
    "翻到下一页时，备注里的页号必须跟着改。",
    "正文和讲稿写在同一物理页的两个半幅上。",
    "这里的句子是夹具原文，用来撑开真实的字形。",
    "连续翻页时，上一页的像素不能留在这一页。",
    "页眉、要点和底栏都要随页号一起更新。",
    "右半幅应能读到说什么、要点、时长、过渡和提问。",
)


def say_text(page: int, heading: str) -> str:
    text = f"现在看到的是第{page:03d}页，题目是{heading}。"
    index = page
    while len(text) < 140:
        text += SAY_BITS[index % len(SAY_BITS)]
        index += 3
    if not 120 <= len(text) <= 180:
        raise RuntimeError(f"page {page} note length {len(text)} is outside 120-180")
    return text


def transition(page: int) -> str:
    if page >= 100:
        return "一百页夹具到此结束。"
    return f"下面翻到下一页，继续核对第{page + 1:03d}页。"


def note_markup(page: int, heading: str) -> str:
    seconds = 20 + (page % 41)
    return "\n".join(
        (
            f"说什么：{say_text(page, heading)}\\\\[0.4em]",
            f"要点：第{page:03d}页的讲稿必须与正文同时出现。\\\\[0.4em]",
            f"时长：{seconds} 秒\\\\[0.4em]",
            f"过渡：{transition(page)}\\\\[0.4em]",
            f"可能提问：第{page:03d}页的正文是否在左半幅？；第{page:03d}页的讲稿是否在右半幅？",
        )
    )


def build_frames() -> list[dict[str, object]]:
    frames: list[dict[str, object]] = [{"kind": "cover"}, {"kind": "toc"}]
    for chapter, count in enumerate(CONTENT_COUNTS, start=1):
        frames.append({"kind": "opener", "chapter": chapter})
        for ordinal in range(1, count + 1):
            frames.append(
                {
                    "kind": "content",
                    "chapter": chapter,
                    "ordinal": ordinal,
                    "layout": LAYOUTS[(ordinal - 1) % len(LAYOUTS)],
                }
            )
    frames.append({"kind": "thanks"})
    if len(frames) != 100:
        raise RuntimeError(f"expected 100 frames, got {len(frames)}")
    for page, frame in enumerate(frames, start=1):
        frame["page"] = page
        say_text(page, heading_for(frame))
    page50 = frames[49]
    if page50["kind"] != "content" or page50["layout"] != "bullets":
        raise RuntimeError(f"page 50 is {page50['kind']}/{page50['layout']}, expected content/bullets")
    return frames


def heading_for(frame: dict[str, object]) -> str:
    kind = frame["kind"]
    if kind == "cover":
        return "封面"
    if kind == "toc":
        return "目录"
    if kind == "thanks":
        return "致谢"
    chapter = int(frame["chapter"])
    title = CHAPTERS[chapter - 1][0]
    if kind == "opener":
        return f"第{chapter}章目录"
    return f"{chapter}.{frame['ordinal']} {title}"


def content_body(frame: dict[str, object]) -> str:
    page = int(frame["page"])
    marker = f"正文页{page:03d}"
    layout = frame["layout"]
    takeaway = f"第{page:03d}页只验证这一帧的正文与讲稿。"
    subsection = marker
    if layout == "bullets":
        body = "\n".join(
            (
                r"\begin{itemize}",
                rf"\item {marker}：本条只服务性能测试。",
                r"\item 左半幅保持十六比九的幻灯片。",
                r"\item 右半幅是这一页的五栏讲稿。",
                rf"\item 翻页后不得继续显示第{max(page - 1, 1):03d}页的句子。",
                r"\end{itemize}",
            )
        )
    elif layout == "cards":
        body = "\n".join(
            (
                rf"\DefenseCard{{正文}}{{{marker}，这张卡片放在左半幅。}}",
                r"\DefenseCard{讲稿}{同号五栏讲稿放在右半幅。}",
                rf"\DefenseCard{{页码}}{{本卡对应物理页 {page}。}}",
            )
        )
    elif layout == "equation":
        body = "\n".join(
            (
                rf"{{\normalsize {marker}，用一条公式占住版心。}}\par",
                r"\begin{equation*}",
                rf"y_{{{page}}} = {page} x + 1",
                r"\end{equation*}",
            )
        )
    elif layout == "table":
        body = "\n".join(
            (
                rf"{{\normalsize {marker}，用一张小表占住版心。}}\par",
                r"\begin{center}",
                r"\begin{tabular}{lll}",
                r"\toprule",
                r"项目 & 位置 & 页码 \\",
                r"\midrule",
                rf"正文 & 左半幅 & {page} \\",
                rf"讲稿 & 右半幅 & {page} \\",
                r"版式 & 十六比九 & 加备注 \\",
                r"\bottomrule",
                r"\end{tabular}",
                r"\end{center}",
            )
        )
    else:
        raise RuntimeError(f"unknown layout {layout}")
    title = heading_for(frame)
    return "\n".join(
        (
            rf"\begin{{frame}}{{{title}}}",
            rf"\DefenseSubsection{{{subsection}}}",
            body,
            rf"\DefenseTakeaway{{{takeaway}}}",
            note_block(frame),
            r"\end{frame}",
        )
    )


def note_block(frame: dict[str, object]) -> str:
    page = int(frame["page"])
    return "\\note{%\n" + note_markup(page, heading_for(frame)) + "\n}"


def opener_body(frame: dict[str, object]) -> str:
    chapter = int(frame["chapter"])
    title, icon, left, right = CHAPTERS[chapter - 1]
    page = int(frame["page"])
    return "\n".join(
        (
            r"\begin{frame}[plain]",
            rf"\ysuChapterOpener{{第{chapter}章如何核对本页测试文本？}}{{{left}}}{{{right}}}{{分页}}",
            rf"\ysuOpenerIcons{{{icon}}}{{}}{{}}{{}}",
            rf"\ysuOpenerOutline{{\ysuOpenRow{{{chapter}.1}}{{{left}}}\ysuOpenRow{{{chapter}.2}}{{{right}}}}}",
            rf"\ysuOpenerBridge{{输入：正文页{page:03d}；输出：同号五栏讲稿}}",
            rf"\DefenseTocFrame{{{chapter}}}",
            note_block(frame),
            r"\end{frame}",
        )
    )


def frame_body(frame: dict[str, object]) -> str:
    kind = frame["kind"]
    if kind == "cover":
        return "\n".join(
            (
                r"\begin{frame}[plain]",
                r"\DefenseCoverFrame",
                note_block(frame),
                r"\end{frame}",
            )
        )
    if kind == "toc":
        return "\n".join(
            (
                r"\begin{frame}[plain]",
                r"\DefenseTocFrame{0}",
                note_block(frame),
                r"\end{frame}",
            )
        )
    if kind == "opener":
        return opener_body(frame)
    if kind == "content":
        return content_body(frame)
    if kind == "thanks":
        page = int(frame["page"])
        return "\n".join(
            (
                r"\begin{frame}[plain]",
                rf"\ysuThanksRecap{{分页正文}}{{正文页{page:03d}\\互不重复}}{{讲稿对齐}}{{右侧备注带\\相同页号}}{{几何分栏}}{{左幻灯片\\右五栏讲稿}}",
                r"\ysuThanksTags{\ysuThanksTag{正文}\ysuThanksTag{页号}}{\ysuThanksTag{讲稿}\ysuThanksTag{五栏}}{\ysuThanksTag{分栏}\ysuThanksTag{性能}}",
                r"\DefenseThanksFrame",
                note_block(frame),
                r"\end{frame}",
            )
        )
    raise RuntimeError(f"unknown frame kind {kind}")


def preamble() -> str:
    chapters = "\n".join(
        rf"\DefenseAddChapter{{{index}}}{{{title}}}"
        for index, (title, *_rest) in enumerate(CHAPTERS, start=1)
    )
    return rf"""\documentclass[aspectratio=169,11pt]{{ctexbeamer}}
\usetheme{{YanshanDefense}}
\usepackage{{amsmath}}
\usepackage{{booktabs}}
\usepackage{{ysu-defense-modern}}
\setbeameroption{{show notes on second screen=right}}
\setbeamertemplate{{note page}}{{%
  \begin{{minipage}}[t][\paperheight][t]{{\paperwidth}}%
    \vspace*{{0.7cm}}%
    \hspace*{{0.7cm}}\begin{{minipage}}{{0.86\paperwidth}}%
      \normalsize\insertnote
    \end{{minipage}}%
  \end{{minipage}}%
}}
\AtBeginDocument{{%
  \NewCommandCopy\PerfIncludegraphics\includegraphics
  \RenewDocumentCommand\includegraphics{{s O{{}} m}}{{%
    \IfFileExists{{#3}}{{%
      \IfBooleanTF{{#1}}{{\PerfIncludegraphics*[#2]{{#3}}}}{{\PerfIncludegraphics[#2]{{#3}}}}%
    }}{{}}%
  }}%
}}
\DefenseSetup{{stage=predefense}}
\title{{FerroStage 分页性能夹具\\一百页测试文本}}
\author{{性能夹具}}
\DefenseSupervisor{{自动生成}}
\DefenseSubject{{演示测试}}
\DefenseSchool{{FerroStage}}
\date{{2026/10/16}}
{chapters}
"""


def render(limit: int) -> str:
    frames = build_frames()
    if not 1 <= limit <= len(frames):
        raise RuntimeError(f"limit must be 1..{len(frames)}")
    parts = [preamble(), r"\begin{document}"]
    parts.extend(frame_body(frame) for frame in frames[:limit])
    parts.append(r"\end{document}")
    parts.append("")
    return "\n\n".join(parts)


def output_dir(limit: int) -> Path:
    name = "perf-deck" if limit == 100 else "perf-deck-probe"
    return REPO / "cache" / name


def compile_tex(tex_path: Path, deck: Path) -> None:
    # Run XeLaTeX with the theme directory as cwd so the style files resolve
    # as local inputs. A non-ASCII TEXINPUTS path is not reliable here: the
    # engine's path search can miss it even when kpsewhich accepts it.
    # -output-directory keeps aux and pdf files out of the theme directory.
    xelatex = shutil.which("xelatex")
    if xelatex is None:
        raise RuntimeError("xelatex not found")
    deck = deck.resolve()
    if not deck.is_dir():
        raise RuntimeError(f"deck directory not found: {deck}")
    env = os.environ.copy()
    env.pop("TEXINPUTS", None)
    log = tex_path.with_suffix(".log")
    command = [
        xelatex,
        "-interaction=nonstopmode",
        "-halt-on-error",
        "-file-line-error",
        f"-output-directory={tex_path.parent.resolve()}",
        str(tex_path.resolve()),
    ]
    for _pass in range(4):
        result = subprocess.run(
            command,
            cwd=deck,
            env=env,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )
        if result.returncode != 0:
            tail = ""
            if log.is_file():
                tail = "\n".join(log.read_text(encoding="utf-8", errors="replace").splitlines()[-80:])
            else:
                tail = (result.stdout + "\n" + result.stderr)[-4000:]
            raise RuntimeError(f"xelatex exited {result.returncode}\n{tail}")
        if not log.is_file():
            return
        text = log.read_text(encoding="utf-8", errors="replace")
        if "Rerun to get" not in text and "Please rerun" not in text and "Label(s) may have changed" not in text:
            return
    raise RuntimeError("xelatex still requested a rerun after 4 passes")


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate the 100-page Beamer performance fixture.")
    parser.add_argument("--limit", type=int, default=100, help="number of frames to emit (2 for the probe)")
    parser.add_argument("--compile", action="store_true", help="compile with xelatex")
    parser.add_argument("--deck", type=Path, default=DEFAULT_DECK, help="directory containing the defense theme")
    args = parser.parse_args()
    out_dir = output_dir(args.limit)
    out_dir.mkdir(parents=True, exist_ok=True)
    tex_path = out_dir / "perf-deck.tex"
    tex_path.write_text(render(args.limit), encoding="utf-8", newline="\n")
    print(f"wrote {tex_path}")
    if not args.compile:
        return 0
    compile_tex(tex_path, args.deck)
    pdf_path = tex_path.with_suffix(".pdf")
    if not pdf_path.is_file():
        raise RuntimeError(f"missing {pdf_path}")
    print(f"compiled {pdf_path}")
    if args.limit == 100:
        destination = ROOT / PDF_NAME
        shutil.copyfile(pdf_path, destination)
        print(f"copied {destination}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except RuntimeError as exc:
        print(f"error: {exc}", file=sys.stderr)
        sys.exit(1)
