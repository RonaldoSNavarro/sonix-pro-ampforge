#!/usr/bin/env python3
"""Verificador de integridade documental das specs (001..099).

Valida:
1. Front-matter `status: rascunho` em todas as specs 059..092 e consistência com `specs/README.md`.
2. Todas as citações `arquivo:linha @ <hash>` contra `git show <hash>:<arquivo>`.
3. Ausência de identificadores fabricados/trocados (AUD-38) e de traits obsoletas
   (`DspBlock`, `AutomatableParams`, `ModuleRegistry`) nas specs 053..099 e `specs/README.md`.
4. Existência em disco de caminhos de testes citados (`crates/.../tests/...`, `apps/.../tests/...`,
   `tests/contract/...`) fora de marcações `Planejado` / `planejado`.
5. Consistência de chaves `param.*` citadas contra `crates/sonix-desktop/src/params.rs`.
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

OBSOLETE_TRAITS = [
    "DspBlock",
    "AutomatableParams",
    "ModuleRegistry",
]

CITE_RE = re.compile(
    r"([A-Za-z0-9_./-]+\.(?:rs|ts|js|py|md|json|toml)):(\d+)(?:[–-](\d+))?\s*@\s*([0-9a-f]{7,40})"
)
ID_RE = re.compile(r"\b(" + "|".join(FABRICATED_OR_WRONG_IDS) + r")\b")
OBSOLETE_RE = re.compile(r"\b(" + "|".join(OBSOLETE_TRAITS) + r")\b")
TEST_PATH_RE = re.compile(
    r"((?:crates|apps)/[^\s\)\"\'\`:]+/tests/[^\s\)\"\'\`:]+|tests/contract/[^\s\)\"\'\`:]+)"
)
PARAM_KEY_RE = re.compile(
    r"`((?:input|gate|amp_1|amp_2|cab_1|cab_2|comp|comp_1|comp_2|eq|eq_1|delay|dly_1|reverb|rev_1|drive|mod|pitch|wah|tuner|looper|engine|footswitch|master)\.[a-z0-9_]+)`"
)

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


def load_params_rs_keys() -> set[str]:
    params_path = os.path.join("crates", "sonix-desktop", "src", "params.rs")
    if not os.path.exists(params_path):
        return set()
    with open(params_path, encoding="utf-8") as f:
        txt = f.read()
    return set(re.findall(r'"([a-z0-9_]+\.[a-z0-9_]+)"', txt))


def load_readme_statuses() -> dict[int, str]:
    readme_path = os.path.join("specs", "README.md")
    if not os.path.exists(readme_path):
        return {}
    statuses: dict[int, str] = {}
    with open(readme_path, encoding="utf-8") as f:
        for line in f:
            m = re.match(r"^\|\s*\[(\d{3})\]\([^\)]+\)\s*\|[^|]+\|\s*([^|]+)\|", line)
            if m:
                statuses[int(m.group(1))] = m.group(2).strip().lower()
    return statuses


def test_path_exists(raw_path: str) -> bool:
    clean = raw_path.rstrip(".,;:")
    if "*" in clean:
        return len(glob.glob(clean)) > 0
    return os.path.exists(clean)


def main() -> int:
    spec_files = sorted(glob.glob("specs/[0-9][0-9][0-9]*.md"))
    params_keys = load_params_rs_keys()
    readme_statuses = load_readme_statuses()

    invalid_citations: list[str] = []
    missing_frontmatter: list[str] = []
    bad_identifiers: list[str] = []
    obsolete_trait_hits: list[str] = []
    missing_test_paths: list[str] = []
    status_mismatches: list[str] = []
    param_key_errors: list[str] = []
    checked_citations = 0
    checked_test_paths = 0
    planned_param_keys_count = 0

    # Verificar traits obsoletas também em specs/README.md
    if os.path.exists("specs/README.md"):
        with open("specs/README.md", encoding="utf-8") as f:
            for idx, line in enumerate(f, 1):
                for m in OBSOLETE_RE.finditer(line):
                    obsolete_trait_hits.append(f"README.md:{idx} -> {m.group(1)}")

    for path in spec_files:
        base = os.path.basename(path)
        num = int(base[:3])
        with open(path, encoding="utf-8") as f:
            content = f.read()
        lines = content.splitlines()

        fm_status: str | None = None
        if len(lines) >= 3 and lines[0].strip() == "---":
            for l in lines[1:15]:
                if l.strip() == "---":
                    break
                sm = re.match(r"^status:\s*(.+)$", l.strip())
                if sm:
                    fm_status = sm.group(1).strip().lower()
                    break

        # 1. Front-matter status válido (rascunho, aprovada, implementada) nas specs 059..092
        if 59 <= num <= 92 and fm_status not in ("rascunho", "aprovada", "implementada"):
            missing_frontmatter.append(base)

        # Consistência com specs/README.md
        if num in readme_statuses and fm_status is not None:
            rm = readme_statuses[num]
            if fm_status == "rascunho" and "rascunho" not in rm:
                status_mismatches.append(f"{base}: front-matter='rascunho' vs README='{rm}'")
            elif fm_status == "aprovada" and "aprovada" not in rm:
                status_mismatches.append(f"{base}: front-matter='aprovada' vs README='{rm}'")
            elif fm_status == "implementada" and "implementada" not in rm:
                status_mismatches.append(f"{base}: front-matter='implementada' vs README='{rm}'")

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

        # 3. Identificadores fabricados/trocados & traits obsoletas nas specs 053..099
        if num >= 53:
            for idx, line in enumerate(lines, 1):
                if "Removido em 2026-11-20 (AUD-38)" in line:
                    continue
                for m in ID_RE.finditer(line):
                    bad_identifiers.append(f"{base}:{idx} -> {m.group(1)}")
                # Permitir apenas se a linha explicita que a trait foi substituída/proibida
                if "substitu" not in line.lower() and "proibid" not in line.lower() and "obsolet" not in line.lower():
                    for m in OBSOLETE_RE.finditer(line):
                        obsolete_trait_hits.append(f"{base}:{idx} -> {m.group(1)}")

        # 4. Existência em disco de caminhos de testes citados (053..099)
        if num >= 53:
            for idx, line in enumerate(lines, 1):
                low = line.lower()
                if "planejado" in low or "planejada" in low:
                    continue
                for m in TEST_PATH_RE.finditer(line):
                    tp = m.group(1)
                    checked_test_paths += 1
                    if not test_path_exists(tp):
                        missing_test_paths.append(f"{base}:{idx} -> {tp}")

        # 5. Validação cruzada de chaves param.* contra params.rs (053..099)
        if num >= 53 and params_keys:
            for idx, line in enumerate(lines, 1):
                low = line.lower()
                for m in PARAM_KEY_RE.finditer(line):
                    key = m.group(1)
                    if key not in params_keys:
                        planned_param_keys_count += 1
                        if fm_status == "implementada" and "planejad" not in low and "alias" not in low and "legado" not in low:
                            param_key_errors.append(
                                f"{base}:{idx} -> chave `{key}` ausente em params.rs em spec implementada"
                            )

    print(f"Specs verificadas: {len(spec_files)} (001..{int(os.path.basename(spec_files[-1])[:3]):03d})")
    print(f"Citações arquivo:linha @ <hash> verificadas: {checked_citations}")
    print(f"Caminhos de teste em disco verificados (053+): {checked_test_paths}")
    print(f"Chaves de parâmetro planejadas ainda ausentes em params.rs (053+): {planned_param_keys_count}")
    print(f"Citações inválidas: {len(invalid_citations)}")
    for item in invalid_citations:
        print(f"  - {item}")
    print(f"Front-matter `status: rascunho` ausente (059..092): {len(missing_frontmatter)}")
    for item in missing_frontmatter:
        print(f"  - {item}")
    print(f"Divergências de status com specs/README.md: {len(status_mismatches)}")
    for item in status_mismatches:
        print(f"  - {item}")
    print(f"Identificadores inexistentes/trocados como código atual (053+): {len(bad_identifiers)}")
    for item in bad_identifiers:
        print(f"  - {item}")
    print(f"Traits obsoletas (DspBlock/AutomatableParams/ModuleRegistry) (053+ & README): {len(obsolete_trait_hits)}")
    for item in obsolete_trait_hits:
        print(f"  - {item}")
    print(f"Caminhos de teste ausentes no disco sem marcação 'Planejado:' (053+): {len(missing_test_paths)}")
    for item in missing_test_paths:
        print(f"  - {item}")
    print(f"Chaves param.* ausentes em params.rs em specs implementadas: {len(param_key_errors)}")
    for item in param_key_errors:
        print(f"  - {item}")

    if (
        invalid_citations
        or missing_frontmatter
        or status_mismatches
        or bad_identifiers
        or obsolete_trait_hits
        or missing_test_paths
        or param_key_errors
    ):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
