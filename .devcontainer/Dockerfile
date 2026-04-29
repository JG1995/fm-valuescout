FROM node:22-slim

# ==============================================================================
# System packages — edit this list to add project-specific runtimes
# (e.g. rustc, golang, python3-full, etc.)
# ==============================================================================
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    zsh \
    ca-certificates \
    curl \
    git \
    git-lfs \
    jq \
    python3-venv \
    unzip \
    && rm -rf /var/lib/apt/lists/*

RUN git config --global init.defaultBranch main

# ==============================================================================
# Bun runtime
# ==============================================================================
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"

# ==============================================================================
# uv + pre-commit
# ==============================================================================
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:${PATH}"
RUN uv python install && uv tool install pre-commit

# ==============================================================================
# oh-my-pi coding agent
# ==============================================================================
RUN /root/.bun/bin/bun install -g @oh-my-pi/pi-coding-agent

# ==============================================================================
# Project-specific runtimes — add yours below this line
# Examples:
#   RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
#   ENV PATH="/root/.cargo/bin:${PATH}"
# ==============================================================================
