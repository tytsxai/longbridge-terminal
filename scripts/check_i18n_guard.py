#!/usr/bin/env python3
"""
中文化守门脚本

用途：
1) 校验 locales/en.yml、locales/zh-CN.yml、locales/zh-HK.yml 键集一致
2) 扫描 UI 渲染路径，阻止新增“疑似硬编码英文 UI 文案”
3) 扫描 CLI 用户可见路径，阻止新增“疑似硬编码英文报错/帮助文案”
4) 校验中文入口文档存在且关键链接可达
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Iterable

import yaml


ROOT = Path(__file__).resolve().parents[1]
LOCALES_DIR = ROOT / "locales"

UI_TARGETS = [
    ROOT / "src/system.rs",
    *sorted((ROOT / "src/views").glob("*.rs")),
    *sorted((ROOT / "src/widgets").glob("*.rs")),
    ROOT / "src/ui/content.rs",
    ROOT / "src/ui/styles.rs",
]

CLI_TARGETS = [
    ROOT / "src/cli.rs",
]

DOC_ENTRY_FILES = [
    ROOT / "README.md",
    ROOT / "README_zh-CN.md",
    ROOT / "docs/quickstart_zh-CN.md",
]

UI_CALL_MARKERS = (
    "Paragraph::new(",
    "Line::from(",
    "Line::styled(",
    "Span::raw(",
    "Span::styled(",
    "Cell::from(",
    ".title(",
)

STRING_LITERAL_RE = re.compile(r'"([^"\\]*(?:\\.[^"\\]*)*)"')
I18N_CALL_RE = re.compile(r't!\(\s*"[^"]*"(?:\s*,[^)]*)?\)')
ONLY_SYMBOL_RE = re.compile(r"^[\s\d\W_]*$")
TOKEN_RE = re.compile(r"[A-Za-z][A-Za-z0-9+./-]*")
PLACEHOLDER_NAME_RE = re.compile(r"\{([A-Za-z_][A-Za-z0-9_]*)(?::[^}]*)?\}")

# 允许在中文文案中保留的英文术语（品牌、协议、常见缩写）
ALLOWED_ENGLISH_TOKENS = {
    "API",
    "App",
    "CLI",
    "ESC",
    "ETF",
    "iOS",
    "Android",
    "Tab",
    "Shift",
    "Enter",
    "Arrow",
    "OpenAPI",
    "Longport",
    "LONGPORT",
    "CHANGQIAO",
    "LONGBRIDGE",
    "HKD",
    "USD",
    "CNY",
    "SGD",
    "JPY",
    "GBP",
    "EUR",
}
FLAG_LITERAL_RE = re.compile(r"^--?[A-Za-z0-9][A-Za-z0-9-]*$")
PLACEHOLDER_ONLY_RE = re.compile(r"^[{}\s_:.,+-/*%0-9A-Za-z$<>|()[\]]+$")


def flatten_dict(data: object, prefix: str = "") -> set[str]:
    if isinstance(data, dict):
        keys: set[str] = set()
        for key, value in data.items():
            next_prefix = f"{prefix}.{key}" if prefix else str(key)
            keys |= flatten_dict(value, next_prefix)
        return keys
    return {prefix}


def load_yaml(path: Path) -> object:
    with path.open("r", encoding="utf-8") as f:
        return yaml.safe_load(f)


def has_cjk(text: str) -> bool:
    return bool(re.search(r"[\u4e00-\u9fff]", text))


def should_skip_literal(text: str) -> bool:
    stripped = text.strip()
    if not stripped:
        return True
    if stripped.startswith(("http://", "https://")):
        return True
    if re.fullmatch(r"(\\[nrt]|\\u[0-9a-fA-F]{4})+", stripped):
        return True
    if has_cjk(text):
        normalized_text = re.sub(r"\\[nrt]", " ", text)
        placeholder_names = set(PLACEHOLDER_NAME_RE.findall(text))
        tokens = [
            token
            for token in TOKEN_RE.findall(normalized_text)
            if token not in placeholder_names
        ]
        if not tokens:
            return True
        # 中文文案中仅包含允许术语时放行
        if all(token in ALLOWED_ENGLISH_TOKENS for token in tokens):
            return True
    if ONLY_SYMBOL_RE.match(text):
        return True
    if "{" in text and "}" in text and not re.search(r"[A-Za-z]", text):
        return True
    # 键位提示类占位（例如 " {} ─── {}[g] "）
    if re.fullmatch(r"[\s{}().:\-_\[\]0-9]*[A-Za-z][\s{}().:\-_\[\]0-9]*", text):
        return True
    return False


def check_locale_key_consistency() -> list[str]:
    en_keys = flatten_dict(load_yaml(LOCALES_DIR / "en.yml"))
    issues: list[str] = []
    for locale_file in ("zh-CN.yml", "zh-HK.yml"):
        locale_keys = flatten_dict(load_yaml(LOCALES_DIR / locale_file))
        missing = sorted(en_keys - locale_keys)
        extra = sorted(locale_keys - en_keys)
        if missing:
            issues.append(
                f"[{locale_file}] 缺失 {len(missing)} 个键，示例：{', '.join(missing[:5])}"
            )
        if extra:
            issues.append(
                f"[{locale_file}] 多出 {len(extra)} 个键，示例：{', '.join(extra[:5])}"
            )
    return issues


def check_doc_entrypoints() -> list[str]:
    issues: list[str] = []
    for path in DOC_ENTRY_FILES:
        if not path.exists():
            rel = path.relative_to(ROOT)
            issues.append(f"[{rel}] 文档入口缺失")

    readme_path = ROOT / "README.md"
    if readme_path.exists():
        content = readme_path.read_text(encoding="utf-8")
        required_links = [
            "docs/quickstart_zh-CN.md",
            "docs/faq_zh-CN.md",
            "docs/chinese_localization_checklist_zh-CN.md",
        ]
        for link in required_links:
            if link not in content:
                issues.append(f"[README.md] 缺少中文文档链接：{link}")

    return issues


def iter_ui_lines() -> Iterable[tuple[Path, int, str]]:
    for path in UI_TARGETS:
        if not path.exists():
            continue
        for idx, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            if any(marker in line for marker in UI_CALL_MARKERS):
                yield path, idx, line


def check_hardcoded_english_ui() -> list[str]:
    issues: list[str] = []
    for path, line_no, line in iter_ui_lines():
        sanitized = I18N_CALL_RE.sub("t!(I18N_KEY)", line)
        for match in STRING_LITERAL_RE.finditer(sanitized):
            literal = match.group(1)
            if should_skip_literal(literal):
                continue
            if re.search(r"[A-Za-z]", literal):
                rel = path.relative_to(ROOT)
                issues.append(
                    f"[{rel}:{line_no}] 疑似硬编码英文 UI 文案：\"{literal}\""
                )
    return issues


def is_suspect_english_literal(text: str) -> bool:
    if should_skip_literal(text):
        return False
    if FLAG_LITERAL_RE.fullmatch(text.strip()):
        return False
    if PLACEHOLDER_ONLY_RE.fullmatch(text.strip()):
        return False
    return bool(re.search(r"[A-Za-z]", text))


def check_hardcoded_english_cli() -> list[str]:
    issues: list[str] = []
    for path in CLI_TARGETS:
        if not path.exists():
            continue
        rel = path.relative_to(ROOT)
        for line_no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            # 聚焦帮助/报错路径，避免扫描所有测试代码导致误报
            if not any(marker in line for marker in ("help_text(", "message:", "format!(")):
                continue
            for match in STRING_LITERAL_RE.finditer(line):
                literal = match.group(1)
                if is_suspect_english_literal(literal):
                    issues.append(
                        f"[{rel}:{line_no}] 疑似硬编码英文 CLI 文案：\"{literal}\""
                    )
    return issues


def main() -> int:
    issues = []
    issues.extend(check_locale_key_consistency())
    issues.extend(check_doc_entrypoints())
    issues.extend(check_hardcoded_english_ui())
    issues.extend(check_hardcoded_english_cli())

    if issues:
        print("❌ 中文化守门检查失败：")
        for issue in issues:
            print(f"  - {issue}")
        return 1

    print("✅ 中文化守门检查通过")
    return 0


if __name__ == "__main__":
    sys.exit(main())
