#!/usr/bin/env python3
"""AUD-38: Verificador somente leitura das specs 001..092.

Valida:
1. Front-matter `status: rascunho` em todas as specs 059..092.
2. Todas as citações `arquivo:linha @ <hash>` contra `git show <hash>:<arquivo>`.
3. Ausência dos identificadores fabricados/trocados listados no AUD-38 como código atual nas specs 053..092.
"""

from __future__ import annotations

import glob
import os
import re
import subprocess
import sys

FABRICATED_OR_WRONG_IDS = [
    "pre_filter",
    "post_filter",
    "set_lowpass",
    "muff_lpf",
    "muff_hpf",
    "det_level",
    "Eq8Processor",
    "GraphicEqProcessor",
    "TremoloOptical",
    "bias_shift",
    "rat_filter_state",
    "re201_head_gains",
    "TapeRe201",
    "DigitalClean",
    "AnalogBbd",
    "PingPong",
    "ProCoRat",
]

CITE_RE = re.compile(
    r"([A-Za-z0-9_./-]+\.(?:rs|ts|js|py|md|json|toml)):(\d+)(?:[–-](\d+))?\s*@\s*([0-9a-f]{7,40})"
)
ID_RE = re.compile(r"\b(" + "|".join(FABRICATED_OR_WRONG_IDS) + r")\b")

_git_cache: dict[tuple[str, str], list[str] | None] = {}


def git_show_lines(commit_hash: str, rel_path: str) -> list[str] | None:
    key = (commit_hash, rel_path)
    if key in _git_cache:
        return _git_cache[key]
    res = subprocess.run(
        ["git", "show", f"{commit_hash}:{rel_path}"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if res.returncode != 0:
        _git_cache[key] = None
        return None
    lines = res.stdout.splitlines()
    _git_cache[key] = lines
    return lines


def main() -> int:
    spec_files = sorted(glob.glob("specs/[0-9][0-9][0-9]*.md"))
    invalid_citations: list[str] = []
    missing_frontmatter: list[str] = []
    bad_identifiers: list[str] = []
    checked_citations = 0

    for path in spec_files:
        base = os.path.basename(path)
        num = int(base[:3])
        with open(path, encoding="utf-8") as f:
            content = f.read()
        lines = content.splitlines()

        # 1. Front-matter status: rascunho nas specs 059..092
        if 59 <= num <= 92:
            has_yaml_status = (
                len(lines) >= 3
                and lines[0].strip() == "---"
                and any(re.match(r"^status:\s*rascunho\b", l.strip()) for l in lines[1:12])
            )
            if not has_yaml_status:
                missing_frontmatter.append(base)

        # 2. Citações arquivo:linha @ <hash>
        for idx, line in enumerate(lines, 1):
            for m in CITE_RE.finditer(line):
                rel_path, ln_s, end_s, h = m.groups()
                ln = int(ln_s)
                end_ln = int(end_s) if end_s else ln
                checked_citations += 1
                file_lines = git_show_lines(h, rel_path)
                if file_lines is None:
                    invalid_citations.append(f"{base}:{idx} -> {rel_path}:{ln} @ {h} (arquivo/hash inexistente)")
                elif ln < 1 or end_ln > len(file_lines):
                    invalid_citations.append(
                        f"{base}:{idx} -> {rel_path}:{ln}-{end_ln} @ {h} (fora de 1..{len(file_lines)})"
                    )

        # 3. Identificadores fabricados/trocados nas specs 053..092
        if 53 <= num <= 92:
            for idx, line in enumerate(lines, 1):
                if "Removido em 2026-11-20 (AUD-38)" in line:
                    continue
                for m in ID_RE.finditer(line):
                    bad_identifiers.append(f"{base}:{idx} -> {m.group(1)}")

    print(f"Specs verificadas: {len(spec_files)} (001..092)")
    print(f"Citações arquivo:linha @ <hash> verificadas: {checked_citations}")
    print(f"Citações inválidas: {len(invalid_citations)}")
    for item in invalid_citations:
        print(f"  - {item}")
    print(f"Front-matter `status: rascunho` ausente (059..092): {len(missing_frontmatter)}")
    for item in missing_frontmatter:
        print(f"  - {item}")
    print(f"Identificadores inexistentes/trocados como código atual (053..092): {len(bad_identifiers)}")
    for item in bad_identifiers:
        print(f"  - {item}")

    if invalid_citations or missing_frontmatter or bad_identifiers:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
