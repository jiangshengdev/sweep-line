#!/usr/bin/env bash
set -euo pipefail

# Vercel 构建阶段脚本：安装 Rust（若缺失）并生成 viewer 所需的 session JSON。

ROOT_DIR="$(pwd)"

# 仅在 Vercel 构建环境启用缓存目录；本地不要强行改 CARGO_HOME/RUSTUP_HOME，避免与 ~/.cargo 冲突。
if [[ "${VERCEL:-}" == "1" ]]; then
  CACHE_DIR="$ROOT_DIR/.vercel/cache"
  mkdir -p "$CACHE_DIR"

  export CARGO_HOME="$CACHE_DIR/cargo-home"
  export RUSTUP_HOME="$CACHE_DIR/rustup-home"
  export CARGO_TARGET_DIR="$CACHE_DIR/cargo-target"

  mkdir -p "$CARGO_HOME" "$RUSTUP_HOME" "$CARGO_TARGET_DIR"
  export PATH="$CARGO_HOME/bin:$PATH"
fi

source_rust_env() {
  if [[ -n "${CARGO_HOME:-}" && -f "$CARGO_HOME/env" ]]; then
    # shellcheck disable=SC1090
    source "$CARGO_HOME/env"
  elif [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
  fi
}

ensure_rust() {
  # 1) 若 cargo 不存在，则安装 rustup + stable。
  if ! command -v cargo >/dev/null 2>&1; then
    echo "未检测到 cargo，开始安装 Rust toolchain（stable, minimal）..." 1>&2
    curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
    source_rust_env
  fi

  # 2) 若 cargo 是 rustup shim 但未配置默认 toolchain，显式安装并选择 stable。
  if ! cargo --version >/dev/null 2>&1; then
    source_rust_env
    if command -v rustup >/dev/null 2>&1; then
      echo "检测到 rustup shim 但未配置 toolchain，安装并选择 stable..." 1>&2
      rustup toolchain install stable --profile minimal
      rustup default stable
    fi
  fi

  # 3) 最终兜底：确保 cargo 可用。
  cargo --version >/dev/null 2>&1
}

ensure_rust

echo "生成 viewer session 数据..." 1>&2
cargo run --locked --bin generate-viewer-sessions

echo "完成：viewer/generated" 1>&2
