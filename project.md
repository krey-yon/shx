This document contains the complete source code of the repository consolidated into a single file for streamlined AI analysis.
The repository contents have been processed and combined with security validation bypassed.

# Repository Overview

## About This Document
This consolidated file represents the complete codebase from the repository, 
merged into a unified document optimized for AI consumption and automated 
analysis workflows.

## Repository Information
- **Repository:** Abhinavexists/dwarp
- **Branch:** main
- **Total Files:** 33
- **Generated:** 2026-09-27T20:54:47.229Z

## Document Structure
The content is organized in the following sequence:
1. This overview section
2. Repository metadata and information  
3. File system hierarchy
4. Repository files (when included)
5. Individual source files, each containing:
   a. File path header (## File: path/to/file)
   b. Complete file contents within code blocks

## Best Practices
- Treat this document as read-only - make changes in the original repository
- Use file path headers to navigate between different source files
- Handle with appropriate security measures as this may contain sensitive data
- This consolidated view is generated from the live repository state

## Important Notes
- Files excluded by .gitignore and configuration rules are omitted
- Binary assets are not included - refer to the file structure for complete file listings
- Default ignore patterns have been applied to filter content
- Security validation is disabled - review content for sensitive information carefully

# Repository Structure

```
Abhinavexists/dwarp/
├── .github
│   └── workflows
│       └── publish.yml
├── scripts
│   ├── build_release.sh
│   ├── install.sh
│   └── uninstall.sh
├── terminal
│   ├── agents
│   │   ├── __init__.py
│   │   ├── code_agent.py
│   │   ├── general_agent.py
│   │   └── shell_agent.py
│   ├── api
│   │   ├── __init__.py
│   │   └── gemini.py
│   ├── commands
│   │   ├── __init__.py
│   │   ├── commands.py
│   │   └── os_info.py
│   ├── safety
│   │   ├── __init__.py
│   │   └── safety.py
│   ├── utils
│   │   ├── __init__.py
│   │   ├── config.py
│   │   ├── loading.py
│   │   └── parsers.py
│   ├── __init__.py
│   └── cli.py
├── pyproject.toml
├── requirements.txt
└── uv.lock
```

================================================================================
// File: .github/workflows/publish.yml
================================================================================
name: Publish on PyPI

on:
  release:
    types: [published]

permissions:
  contents: read

jobs:
  build-and-publish:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: 3.12

      - name: Install build tools
        run: |
          python -m pip install --upgrade pip
          pip install build twine

      - name: Clean old build artifacts
        run: |
          rm -rf dist build *.egg-info dwarp.egg-info

      - name: Build package
        run: python -m build

      - name: Verify Package Metadata
        run: twine check dist/*

      - name: Publish to TestPyPI (prereleases)
        if: github.event.release.prerelease == true
        env:
          TWINE_USERNAME: __token__
          TWINE_PASSWORD: ${{ secrets.TEST_PYPI_API_TOKEN }}
        run: twine upload --repository testpypi dist/*
      
      - name: Publish to PyPI (stable)
        if: github.event.release.prerelease == false
        env:
          TWINE_USERNAME: __token__
          TWINE_PASSWORD: ${{ secrets.PYPI_API_TOKEN }}
        run: twine upload dist/*

================================================================================
// File: pyproject.toml
================================================================================
[build-system]
requires = ["setuptools>=61.0", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "dwarp"
version = "0.1.8.1"
description = "Terminal assistant built as an open source minimal alternative to Warp"
authors = [{name = "Abhinav", email = "abhinavkumarsingh2023@gmail.com"}]
readme = "README.md"
license = "MIT"
requires-python = ">=3.12"
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "Operating System :: POSIX :: Linux",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.12",
    "Topic :: System :: Shells",
    "Topic :: Utilities",
]
keywords = ["terminal", "ai", "shell", "assistant", "cli", "gemini"]
dependencies = [
    "prompt-toolkit==3.0.51",
    "rich==14.1.0",
    "python-dotenv==1.1.1",
    "google-genai==1.31.0",
    "ruff>=0.15.5",
    "build>=1.4.0",
]

[project.scripts]
dwarp = "terminal.cli:main"

[project.urls]
Homepage = "https://github.com/Abhinavexists/dwarp"
Repository = "https://github.com/Abhinavexists/dwarp.git"
Issues = "https://github.com/Abhinavexists/dwarp/issues"

[tool.setuptools.packages.find]
where = ["."]
include = ["terminal*"]
exclude = ["release*", "scripts*", "ai-terminal-linux*"]

[tool.setuptools.package-data]
terminal = ["*.py"]

[tool.setuptools]
zip-safe = false


================================================================================
// File: requirements.txt
================================================================================
google-genai==1.31.0
prompt-toolkit==3.0.51
rich==14.1.0
pydantic==2.11.7
python-dotenv==1.1.1

================================================================================
// File: scripts/build_release.sh
================================================================================
#!/usr/bin/env bash
set -euo pipefail

# Build and package dwarp binary for Linux using PyInstaller

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
DIST_DIR="$ROOT_DIR/dist"
BUILD_DIR="$ROOT_DIR/build"
OUT_NAME="dwarp"
TARBALL_NAME="${OUT_NAME}-linux.tar.gz"

echo "==> Cleaning previous build artifacts"
rm -rf "$BUILD_DIR/$OUT_NAME" "$DIST_DIR/$OUT_NAME" "$DIST_DIR/$TARBALL_NAME" || true

echo "==> Building wheel/sdist (optional)"
if command -v python &>/dev/null; then
  python -m pip -q install --upgrade build >/dev/null 2>&1 || true
  python -m build || true
fi

echo "==> Building PyInstaller binary"
pyinstaller --clean --noconfirm "$ROOT_DIR/dwarp.spec"

echo "==> Preparing release directory"
RELEASE_DIR="$DIST_DIR/${OUT_NAME}-linux"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Move binary and include helper files
if [[ -f "$DIST_DIR/dwarp" ]]; then
  install -m 755 "$DIST_DIR/dwarp" "$RELEASE_DIR/$OUT_NAME"
elif [[ -f "$DIST_DIR/dwarp/dwarp" ]]; then
  install -m 755 "$DIST_DIR/dwarp/dwarp" "$RELEASE_DIR/$OUT_NAME"
else
  echo "ERROR: Built binary not found. Expected PyInstaller dist outputs under $DIST_DIR."
  ls -l "$DIST_DIR" || true
  exit 1
fi

cp -f "$ROOT_DIR/README.md" "$RELEASE_DIR/" || true
cp -f "$ROOT_DIR/LICENSE" "$RELEASE_DIR/" 2>/dev/null || true
cp -f "$ROOT_DIR/scripts/install.sh" "$RELEASE_DIR/install.sh"
cp -f "$ROOT_DIR/scripts/uninstall.sh" "$RELEASE_DIR/uninstall.sh"
chmod +x "$RELEASE_DIR/install.sh" "$RELEASE_DIR/uninstall.sh"

echo "==> Creating tar.gz"
(
  cd "$DIST_DIR"
  tar -czf "$TARBALL_NAME" "${OUT_NAME}-linux"
)

echo "==> Done: $DIST_DIR/$TARBALL_NAME"


================================================================================
// File: scripts/install.sh
================================================================================
#!/bin/bash

set -euo pipefail

APP_NAME="dwarp"
BIN_TARGET="/usr/local/bin/${APP_NAME}"

echo "Installing ${APP_NAME}..."

if [ "${EUID}" -eq 0 ]; then
    echo "Please don't run this script as root. It will ask for sudo when needed."
    exit 1
fi

if [ ! -f "${APP_NAME}" ]; then
    echo "Error: '${APP_NAME}' binary not found in current directory."
    echo "Run this script from inside the extracted '${APP_NAME}-linux' folder."
    exit 1
fi

sudo mkdir -p /usr/local/bin
sudo install -m 755 "${APP_NAME}" "${BIN_TARGET}"

# Backward-compatibility symlinks (optional)
sudo ln -sf "${BIN_TARGET}" /usr/local/bin/ai-terminal || true
sudo ln -sf "${BIN_TARGET}" /usr/local/bin/ai-terminal-cli || true

echo "${APP_NAME} installed to ${BIN_TARGET}"
echo "You can run it with: ${APP_NAME}"
echo "Legacy commands (if you used them before) will still work: ai-terminal, ai-terminal-cli"


================================================================================
// File: scripts/uninstall.sh
================================================================================
#!/bin/bash

set -euo pipefail

APP_NAME="dwarp"
BIN_TARGET="/usr/local/bin/${APP_NAME}"

echo "Uninstalling ${APP_NAME}..."

if [ "${EUID}" -eq 0 ]; then
    echo "Please don't run this script as root. It will ask for sudo when needed."
    exit 1
fi

# Remove binary and symlinks
sudo rm -f "${BIN_TARGET}"
sudo rm -f /usr/local/bin/ai-terminal
sudo rm -f /usr/local/bin/ai-terminal-cli

# Optional: remove user config
read -p "Remove configuration file (~/.ai_terminal_config.json)? [y/N]: " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -f ~/.ai_terminal_config.json
    echo "Configuration file removed."
fi

echo "${APP_NAME} uninstalled successfully!"


================================================================================
// File: terminal/__init__.py
================================================================================
__version__ = "0.1.8.1"

================================================================================
// File: terminal/agents/__init__.py
================================================================================


================================================================================
// File: terminal/agents/code_agent.py
================================================================================
from terminal.api import client, generate_config
from terminal.core.executor import GeneralResponse, ResponseType
from terminal.utils.parsers import parse_json, parse_response_parts, handle_function_call
from terminal.utils.config import config

def prompt_code():
    return """You are a specialized coding assistant. Your job is to help users with code generation, programming questions, and technical implementations.

IMPORTANT: Keep your response concise and within token limits. Focus on the most essential code and explanations.

Output a JSON object with the following structure:

{{
    "content": "<your response with code examples and explanations>",
    "response_type": "code_generation",
    "action_required": false,
    "suggested_command": null
}}

Instructions:
- Provide clear, well-commented code examples
- Keep explanations brief but informative
- Include essential best practices only
- If the code can be executed as a script, set action_required to true and provide suggested_command
- Use appropriate programming languages based on the request
- Include basic error handling where relevant
- If the response is getting long, prioritize the core code over extensive explanations

CRITICAL: Your response must be ONLY the JSON object, with no extra text, markdown formatting, or code blocks outside the JSON."""


def process_code_request(user_input: str, context: str = "") -> GeneralResponse:
    prompt = prompt_code()
    if context:
        prompt += f"\n\nAdditional Context: {context}"
    
    model_config = config.get_model_config()
    response = client.models.generate_content(
        contents=f"{prompt}\n\nUser request: {user_input}",
        model=model_config["model"],
        config=generate_config,
    )
    
    if response.candidates and hasattr(response.candidates[0], "content") and response.candidates[0].content and hasattr(response.candidates[0].content, "parts") and response.candidates[0].content.parts:
        for part in response.candidates[0].content.parts:
            func_response = handle_function_call(part, ResponseType.CODE_GENERATION)
            if func_response:
                return GeneralResponse(**func_response)
        
        data = parse_response_parts(response.candidates[0].content.parts)
        if data and "content" in data:
            return GeneralResponse(**data)
        
        for part in response.candidates[0].content.parts:
            if hasattr(part, "text") and part.text:
                data = parse_json(part.text)
                if data and "content" in data:
                    return GeneralResponse(**data)
                
                raw_text = part.text.strip()
                if raw_text and len(raw_text) > 10:  
                    cleaned_text = raw_text
                    if cleaned_text.startswith('```'):
                        lines = cleaned_text.split('\n')
                        if lines[0].startswith('```'):
                            lines = lines[1:]
                        if lines and lines[-1].strip() == '```':
                            lines = lines[:-1]
                        cleaned_text = '\n'.join(lines)
                    
                    return GeneralResponse(
                        content=cleaned_text,
                        response_type=ResponseType.CODE_GENERATION,
                        action_required=False,
                        suggested_command=None
                    )

    return GeneralResponse(
        content=f"I apologize, but I encountered an issue generating a proper response for your request: '{user_input}'. The response may have been truncated due to length limits. Please try rephrasing your request or breaking it into smaller parts.",
        response_type=ResponseType.CODE_GENERATION,
        action_required=False,
        suggested_command=None
    )

================================================================================
// File: terminal/agents/general_agent.py
================================================================================
from terminal.api import client, generate_config
from terminal.core.executor import GeneralResponse
from terminal.utils.parsers import parse_json, parse_response_parts
from terminal.utils.config import config

def prompt_general():
    return """You are a helpful AI assistant specializing in general knowledge, explanations, and informational responses.

IMPORTANT: Keep your response concise and within token limits. Focus on the most essential information.

Output a JSON object with the following structure:

{{
    "content": "<your helpful response>",
    "response_type": "general_query",
    "action_required": false,
    "suggested_command": null
}}

Instructions:
- Provide clear, accurate, and concise information
- Use examples and analogies to make complex topics understandable
- If the user asks about something that could be done with a shell command, set action_required to true and provide suggested_command
- Be conversational but informative
- Keep responses focused and to the point
- If the response is getting long, prioritize the most important information

CRITICAL: Your response must be ONLY the JSON object, with no extra text, markdown formatting, or code blocks outside the JSON."""



def process_general_request(user_input: str, context: str = "") -> GeneralResponse:
    prompt = prompt_general()
    if context:
        prompt += f"\n\nAdditional Context: {context}"
    
    model_config = config.get_model_config()
    response = client.models.generate_content(
        contents=f"{prompt}\n\nUser request: {user_input}",
        model=model_config["model"],
        config=generate_config,
    )
    
    if response.candidates and hasattr(response.candidates[0], "content") and response.candidates[0].content and hasattr(response.candidates[0].content, "parts") and response.candidates[0].content.parts:
        data = parse_response_parts(response.candidates[0].content.parts)
        if data and "content" in data:
            return GeneralResponse(**data)
        
        for part in response.candidates[0].content.parts:
            if hasattr(part, "text") and part.text:
                data = parse_json(part.text)
                if data and "content" in data:
                    return GeneralResponse(**data)
                
                raw_text = part.text.strip()
                if raw_text and len(raw_text) > 10:  
                    return GeneralResponse(
                        content=raw_text,
                        response_type="general_query",
                        action_required=False,
                        suggested_command=None
                    )

    return GeneralResponse(
        content=f"I apologize, but I encountered an issue generating a proper response for your request: '{user_input}'. Please try rephrasing your question or breaking it into smaller parts.",
        response_type="general_query",
        action_required=False,
        suggested_command=None
    )

================================================================================
// File: terminal/agents/shell_agent.py
================================================================================
from terminal.api import client, generate_config
from terminal.utils.config import config
from terminal.core.executor import CommandResponse
from terminal.commands import operating_system
from terminal.utils.parsers import parse_json

def prompt_shell():
    os = operating_system.get_os()
    return f"""You are a specialized terminal command assistant. Your job is to translate natural language requests into appropriate shell commands.

Output a JSON object with the following structure:

{{
    "command": "<shell_command>",
    "explanation": "<brief explanation>",
    "response_type": "shell_command"
}}

Current System Context:
{operating_system.get_context()}

Instructions:
- Use the appropriate package manager: {operating_system.get_os()['package_manager']}
- For package installation: {operating_system.get_os()['install']} <package_name>
- For package updates: {operating_system.get_os()['update']}
- For package upgrades: {operating_system.get_os()['upgrade']}
- For package removal: {operating_system.get_os()['remove']} <package_name>
- Only suggest direct shell commands, not Python or other scripts
- Consider the current working directory and use relative paths appropriately
- Always verify file/directory existence before suggesting commands

Ensure your response is ONLY the JSON object, with no extra text or formatting."""



def process_shell_request(user_input: str, current_dir: str = None, context: str = "") -> CommandResponse:
    prompt = prompt_shell()
    if current_dir:
        prompt += f"\n\nCurrent Working Directory: {current_dir}"
    if context:
        prompt += f"\n\nAdditional Context: {context}"
    
    model_config = config.get_model_config()
    response = client.models.generate_content(
        contents=f"{prompt}\n\nUser request: {user_input}",
        model=model_config["model"],
        config=generate_config,
    )
    
    if response.candidates and hasattr(response.candidates[0], "content") and response.candidates[0].content and hasattr(response.candidates[0].content, "parts") and response.candidates[0].content.parts:
        for part in response.candidates[0].content.parts:
            if hasattr(part, "function_call") and part.function_call is not None and hasattr(part.function_call, "args") and part.function_call.args:
                args = part.function_call.args
                return CommandResponse(**args)
            
            if hasattr(part, "text") and part.text:
                data = parse_json(part.text)
                if data and "command" in data and "explanation" in data:
                    return CommandResponse(**data)

    raise ValueError(f"Failed to get shell command response: {response}")

================================================================================
// File: terminal/api/__init__.py
================================================================================
from .gemini import client, generate_config, generate_command_function, tools

__all__ = ['client', 'generate_config', 'generate_command_function', 'tools']

================================================================================
// File: terminal/api/gemini.py
================================================================================
from dotenv import load_dotenv
from google import genai
from google.genai import types
from terminal.utils.config import config

load_dotenv()

api_key = config.get_api_key()
client = genai.Client(api_key=api_key)

generate_command_function = {
                "name": "generate_command",
                "description": "Generate a shell command and explain what it does",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The shell command to execute"
                        },
                        "explanation": {
                            "type": "string",
                            "description": "Explanation of what the command does"
                        },
                    },
                    "required": ["command", "explanation"]
                }
            }

tools = types.Tool(function_declarations=[generate_command_function])

model_config = config.get_model_config()
generate_config = types.GenerateContentConfig(
    tools=[tools],
    temperature=model_config["temperature"],
    max_output_tokens=model_config["max_tokens"]
)

================================================================================
// File: terminal/cli.py
================================================================================
import argparse
import os
import re

from rich import print
from rich.console import Console
from rich.markdown import Markdown

from terminal import __version__
from terminal.core.executor import CommandResponse, GeneralResponse, run_command
from terminal.core.agent import process_request
from terminal.safety import check_command_safety
from terminal.commands import check_shell_command
from terminal.utils.loading import LoadingAnimation

from prompt_toolkit import PromptSession
from prompt_toolkit.history import FileHistory
from prompt_toolkit.completion import FuzzyWordCompleter


def parse_args():
    parser = argparse.ArgumentParser(
        prog="dwarp",
        description="AI-powered terminal assistant",
    )
    parser.add_argument("--version", action="version", version=f"dwarp {__version__}")
    parser.add_argument("--config", metavar="PATH", help="path to config file")
    parser.add_argument("--model", metavar="NAME", help="override model from config")
    parser.add_argument("--verbose", action="store_true", help="enable debug logging")
    return parser.parse_args()

console = Console()

HISTORY_FILE = os.path.expanduser("~/.dwarp_history")


def load_previous_commands():
    if not os.path.exists(HISTORY_FILE):
        return []
    with open(HISTORY_FILE, "r") as file:
        return list(set([line.strip() for line in file if line.strip()]))


def save_command(cmd: str):
    with open(HISTORY_FILE, "a") as file:
        file.write(cmd + "\n")


PLACEHOLDER_PATTERNS = [
    r"<[^>]+>",
    r"\b(old|new)[-_]?(file|filename|path|dir)\b",
    r"\b(source|destination)[-_ ]?(file|directory|dir|path)\b",
    r"\bYOUR[_-]?(FILE|PATH|DIR|BRANCH|REPO)\b",
]


def placeholders(cmd: str) -> bool:
    for pat in PLACEHOLDER_PATTERNS:
        if re.search(pat, cmd, flags=re.IGNORECASE):
            return True
    return False


def edit_command(suggested: str) -> str:
    print(f"[cyan]Current command:[/cyan] {suggested}")
    print("[magenta]Edit the command (press Enter to keep as is):[/magenta]")
    edited = input("> ").strip()
    return edited if edited else suggested


def handle_cd(command: str, current_dir: str) -> tuple[bool, str]:
    parts = command.strip().split()
    if not parts or parts[0] != "cd":
        return False, current_dir

    target = parts[1] if len(parts) > 1 else os.path.expanduser("~")
    target = os.path.expanduser(target)

    if not os.path.isabs(target):
        target = os.path.normpath(os.path.join(current_dir, target))

    if not os.path.isdir(target):
        print(f"[red]cd: no such directory: {target}[/red]")
        return True, current_dir

    return True, target


def handle_shell_command(result: CommandResponse, current_dir: str) -> str:
    """Handle shell command responses."""
    print(f"\n[cyan]Command:[/cyan] {result.command}")
    print(f"[yellow]Explanation:[/yellow] {result.explanation}")

    final_cmd = result.command

    # Special-case clear/cls to avoid printing raw ANSI sequences
    if final_cmd.strip().lower() in {"clear", "cls"}:
        console.clear()
        return current_dir

    if placeholders(final_cmd):
        print("[yellow]Please edit before execution:[/yellow]")
        final_cmd = edit_command(final_cmd)
    else:
        opt = input("Edit command before executing? [y/N]: ").strip().lower()
        if opt == "y":
            final_cmd = edit_command(final_cmd)

    if check_command_safety(final_cmd):
        print("\n[green]Command approved! Executing...[/green]")
        output, success = run_command(final_cmd, cwd=current_dir)
        print(output)
        if success:
            save_command(final_cmd)
        else:
            print("[red]Command failed to execute[/red]")
        return current_dir
    else:
        print("[blue]Command rejected by user[/blue]")
        return current_dir


def handle_general_response(result: GeneralResponse):
    """Handle general query responses."""
    print("\n[bold blue]Response:[/bold blue]")

    if "```" in result.content or "**" in result.content or "##" in result.content:
        console.print(Markdown(result.content))
    else:
        print(result.content)

    if result.action_required and result.suggested_command:
        print(f"\n[cyan]Suggested Command:[/cyan] {result.suggested_command}")
        opt = input("Execute this command? [y/N]: ").strip().lower()
        if opt == "y":
            return result.suggested_command
    return None


def main():
    args = parse_args()

    from terminal.utils import config as config_module
    if args.config:
        config_module.config = config_module.Config(config_file=args.config)
    if args.model:
        config_module.config.model_override = args.model

    print("[bold green]dwarp[/bold green]")
    print("Type your request (type 'exit' to quit)")
    print("Examples: 'install docker', 'what is Python?', 'write a hello world script'\n")

    history = FileHistory(HISTORY_FILE)
    previous_cmds = load_previous_commands()
    session = PromptSession(history=history)

    current_dir = os.getcwd()

    while True:
        completer = FuzzyWordCompleter(previous_cmds)
        try:
            user_input = session.prompt(f"{current_dir} > ", completer=completer).strip()
        except KeyboardInterrupt:
            print("\n[blue]Use 'exit' to quit[/blue]")
            continue
        except EOFError:
            break

        if user_input.lower() in {"exit", "quit"}:
            break

        if not user_input:
            continue

        handled, current_dir = handle_cd(user_input, current_dir)
        if handled:
            continue

        # Special-case clear/cls entered directly by the user
        if user_input.strip().lower() in {"clear", "cls"}:
            console.clear()
            previous_cmds.append(user_input)
            save_command(user_input)
            continue

        if check_shell_command(user_input):
            output, success = run_command(user_input, cwd=current_dir)
            if success:
                print(output)
                save_command(user_input)
                previous_cmds.append(user_input)
            continue

        try:
            loading_animation = LoadingAnimation("Thinking")
            loading_animation.start()
            try:
                result = process_request(user_input, current_dir)
            finally:
                loading_animation.stop()

            if isinstance(result, CommandResponse):
                current_dir = handle_shell_command(result, current_dir)
            elif isinstance(result, GeneralResponse):
                suggested_cmd = handle_general_response(result)
                if suggested_cmd:
                    output, success = run_command(suggested_cmd, cwd=current_dir)
                    print("\n[green]Executing suggested command...[/green]")
                    print(output)
                    if success:
                        save_command(suggested_cmd)
                        previous_cmds.append(suggested_cmd)

        except Exception as e:
            print(f"[red]Error generating response:[/red] {e}")
            print("[yellow]Try rephrasing your request[/yellow]")


if __name__ == "__main__":
    main()

================================================================================
// File: terminal/commands/__init__.py
================================================================================
from .commands import shell_commands, check_shell_command
from .os_info import operating_system

__all__ = ['shell_commands', 'check_shell_command', 'operating_system']

================================================================================
// File: terminal/commands/commands.py
================================================================================
import re

def check_shell_command(user_input: str) -> bool:
    """Check if user input looks like a valid shell command."""
    input_lower = user_input.lower().strip()
    words = input_lower.split()
    
    if not words:
        return False
    
    first_word = words[0]
    
    all_commands = []
    for group in shell_commands.values():
        if isinstance(group, list):
            all_commands.extend(group)
        elif isinstance(group, dict):
            for os_commands in group.values():
                all_commands.extend(os_commands)
    
    if first_word in set(all_commands):
        return True
    
    if re.match(r'^[./]', first_word) or re.search(r'[|&;<>]', user_input):
        return True
    
    return False

shell_commands = {
    "file_directory": [
        "ls", "cd", "pwd", "mkdir", "rmdir", "rm", "cp", "mv", "touch", "tree",
        "basename", "dirname", "realpath", "stat", "ln", "link", "readlink",
        "chmod", "chown", "chgrp", "umask", "file", "wc", "sort", "uniq"
    ],

    "file_viewing_editing": [
        "cat", "less", "more", "head", "tail", "nano", "vim", "emacs",
        "open",        # macOS only
        "xdg-open"     # Linux only
    ],

    "searching_filtering": [
        "grep", "egrep", "fgrep", "find", "locate", "which", "whereis", "xargs",
        "ack", "ag", "rg", "fd", "fzf", "ripgrep", "jq", "yq"
    ],

    "permissions": [
        "chmod", "chown", "chgrp", "umask", "chattr", "lsattr", "getfacl", "setfacl"
    ],

    "system_info_monitoring": [
        "uname", "uptime", "whoami", "id", "df", "du", "top", "htop", "ps",
        "kill", "killall", "systemctl", "launchctl", "free", "vm_stat",
        "clear", "dmesg", "lscpu", "lsblk", "mount", "umount", "env", "printenv",
        "hostname", "arch", "lsof", "netstat", "ss", "iostat", "vmstat", "sar",
        "strace", "ltrace", "time", "watch", "tee", "script"
    ],

    "networking": [
        "ping", "curl", "wget", "scp", "sftp", "ssh", "ftp",
        "ifconfig", "ip", "netstat", "ss", "traceroute", "dig", "nslookup",
        "networksetup", "arp", "route", "nmap", "telnet", "nc", "netcat",
        "rsync", "socat", "mtr", "iftop", "nethogs", "bandwhich"
    ],

    "archiving_compression": [
        "tar", "gzip", "gunzip", "bzip2", "bunzip2", "xz", "unxz",
        "zip", "unzip"
    ],

    "package_managers": {
        "linux": [
            "apt", "apt-get", "dpkg",
            "yum", "dnf", "zypper", "pacman", "emerge", "snap", "flatpak"
        ],
        "macos": [
            "brew", "port", "softwareupdate"
        ]
    },

    "dev_tools": [
        "git", "docker", "kubectl",
        "python", "python3", "pip", "pip3",
        "node", "npm", "yarn", "npx",
        "gcc", "make", "cmake", "cargo", "rustc", "go", "java", "javac",
        "mvn", "gradle", "sbt", "composer", "gem", "bundle", "stack",
        "cabal", "opam", "vcpkg", "conan"
    ],

    "user_system_management": [
        "sudo", "adduser", "useradd", "passwd", "who", "w", "last", "groups",
        "logout", "shutdown", "reboot", "su", "users"
    ],

    "shell_builtins": [
        "alias", "unalias", "history", "export", "set", "unset",
        "echo", "printf", "read", "true", "false", "type"
    ],

    "job_control": [
        "jobs", "fg", "bg", "disown", "wait", "kill", "sleep"
    ],

    "clipboard_macos": [
        "pbcopy", "pbpaste", "say"
    ],

    "clipboard_linux": [
        "xclip", "xsel"
    ],
    
    "text_processing": [
        "sed", "awk", "cut", "paste", "join", "split", "tr", "fold",
        "fmt", "nl", "pr", "column", "expand", "unexpand"
    ],
    
    "process_management": [
        "nice", "renice", "nohup", "screen", "tmux", "at", "cron", "crontab",
        "bg", "fg", "jobs", "disown", "wait", "timeout", "setsid"
    ],
    
    "disk_storage": [
        "fdisk", "parted", "gparted", "mkfs", "fsck", "badblocks",
        "smartctl", "hdparm", "dd", "pv", "rsync", "ddrescue"
    ],
    
    "security": [
        "gpg", "ssh-keygen", "openssl", "certbot", "letsencrypt",
        "ufw", "iptables", "firewalld", "selinux", "apparmor"
    ],
    
    "monitoring_logs": [
        "journalctl", "logrotate", "logwatch", "fail2ban", "auditd",
        "prometheus", "grafana", "zabbix", "nagios"
    ],
    
    "container_orchestration": [
        "docker", "docker-compose", "podman", "buildah", "skopeo",
        "kubectl", "helm", "k9s", "lens", "rancher"
    ],
    
    "cloud_tools": [
        "aws", "gcloud", "az", "terraform", "ansible", "puppet", "chef",
        "vagrant", "packer", "cloud-init"
    ]
}

================================================================================
// File: terminal/commands/os_info.py
================================================================================
import platform
import os

class OperatingSystem:
    def __init__(self):
        self.os_info = None
        self.packages = None

    def get_os(self):
        if self.os_info is None:
            system = platform.system().lower()

            if system == "linux":
                self.os_info = self.linux()
            elif system == "darwin":
                self.os_info = self.macos()
            elif system == "windows":
                self.os_info = self.windows()
            else:
                raise ValueError(f"Unsupported OS: {system}")
        return self.os_info
    
    def linux(self):
        if os.path.exists("/etc/os-release"):
            with open("/etc/os-release", "r") as file:
                lines = file.readlines()
                os_info = {}

                for line in lines:
                    if "=" in line:
                        key, value = line.strip().split("=", 1)
                        os_info[key] = value.strip('"')

            id = os_info.get("ID", "").lower()
            name = os_info.get("NAME", "").lower()

            if id in ["ubuntu", "debian", "linuxmint", "pop", "elementary"]:
                return {
                    "os": "debian",
                    "distro": id,
                    "package_manager": "apt",
                    "install": "sudo apt install",
                    "update": "sudo apt update",
                    "upgrade": "upgrade",
                    "remove": "sudo apt remove",
                }

            elif id in ["fedora", "rhel", "centos", "rocky", "alma"]:
                return {
                    "os": "fedora",
                    "distro": id,
                    "package_manager": "dnf",
                    "install": "sudo dnf install",
                    "update": "sudo dnf update",
                    "upgrade": "upgrade",
                    "remove": "sudo dnf remove",
                }
            
            elif id in ["arch", "manjaro", "endeavouros"]:
                    return {
                    "os": "arch",
                    "distro": id,
                    "package_manager": "pacman",
                    "install": "sudo pacman -S",
                    "update": "sudo pacman -Sy",
                    "upgrade": "sudo pacman -Syu",
                    "remove": "sudo pacman -R"
                }

            elif id in ["opensuse", "sles"]:
                return {
                    "os": "opensuse",
                    "distro": id,
                    "package_manager": "zypper",
                    "install": "sudo zypper install",
                    "update": "sudo zypper refresh",
                    "upgrade": "sudo zypper update",
                    "remove": "sudo zypper remove"
                }
            
            elif id in ["gentoo"]:
                return {
                    "os": "gentoo",
                    "distro": id,
                    "package_manager": "emerge",
                    "install": "sudo emerge",
                    "update": "sudo emerge -u",
                    "upgrade": "sudo emerge -u",
                    "remove": "sudo emerge -C"
                }
            
            elif id in ["nixos"]:
                return {
                    "os": "nix",
                    "distro": id,
                    "package_manager": "nix",
                    "install": "nix-env -iA",
                    "update": "nix-env -u",
                    "upgrade": "nix-env -u",
                    "remove": "nix-env -e"
                }
            
            elif id in ["freebsd", "openbsd", "netbsd"]:
                return {
                    "os": "bsd",
                    "distro": id,
                    "package_manager": "pkg",
                    "install": "sudo pkg install",
                    "update": "sudo pkg update",
                    "upgrade": "sudo pkg upgrade",
                    "remove": "sudo pkg delete"
                }

            raise ValueError(f"Unsupported Linux distribution: {id} ({name})")
        
        return {
            "os": "linux",
            "distro": "unknown",
            "package_manager": "unknown",
            "install": "unknown",
            "update": "unknown",
            "upgrade": "unknown",
            "remove": "unknown"
        }

    def macos(self):
            return {
                "os": "macos",
                "distro": "macos",
                "package_manager": "brew",
                "install": "brew install",
                "update": "brew update",
                "upgrade": "brew upgrade",
                "remove": "brew uninstall"
            }

    def windows(self):
            return {
                "os": "windows",
                "distro": "windows",
                "package_manager": "choco",
                "install": "choco install",
                "update": "choco update",
                "upgrade": "choco upgrade",
                "remove": "choco uninstall"
            }

    def get_package_manager(self):
        return self.get_os()
    
    def install_command(self, package: str):
        os = self.get_os()

        if os['install'] == "unknown":
            return f"{os['install']} {package}"
        return f"# Unknown package manager for {os['distro']} - please install {package} manually"

    def get_context(self):
        os = self.get_os()
        return f"Operating System: {os['os']}, Distribution: {os['distro']}, Package Manager: {os['package_manager']}"

operating_system = OperatingSystem()

================================================================================
// File: terminal/safety/__init__.py
================================================================================
from .safety import CommandSafety, SafetyResult, RiskLevel, check_command_safety

__all__ = ['CommandSafety', 'SafetyResult', 'RiskLevel', 'check_command_safety']

================================================================================
// File: terminal/safety/safety.py
================================================================================
import re
from typing import List, Optional
from dataclasses import dataclass
from enum import Enum
from rich import print

class RiskLevel(Enum):
    SAFE = 'safe'
    LOW = 'low'
    MEDIUM = 'medium'
    HIGH = 'high'
    CRITICAL = 'critical'

@dataclass
class SafetyResult:
    risk_level: RiskLevel
    warning: List[str]
    blocked: bool = False
    suggestions: Optional[List[str]] = None

class CommandSafety:
    """Analyzes commands for potential risks and provides warnings."""
    
    CRITICAL_PATTERNS = [
        r'\brm\s+-rf\s+/',                       # rm -rf /
        r'\bdd\s+.*of=/dev/',                    # dd to device
        r'\bmkfs\.',                             # filesystem creation
        r'\bfdisk\b.*-d',                        # disk partitioning with delete
        r':\(\)\{\s*:\|:\&\s*\}',                # fork bomb
        r'\bchmod\s+777\s+/',                    # chmod 777 on root
    ]
    
    HIGH_RISK_PATTERNS = [
        r'\brm\s+-rf\s+\*',                      # rm -rf *
        r'\brm\s+-rf\s+\.',                      # rm -rf .
        r'\bsudo\s+rm\s+-rf',                    # sudo rm -rf anything
        r'\bchmod\s+-R\s+777',                   # recursive 777
        r'\bdd\s+if=/dev/urandom',               # random data writing
        r'\b>\s*/dev/sd[a-z]',                   # direct write to disk
        r'\bkillall\s+-9',                       # force kill all processes
        r'\bpkill\s+-9',                         # force kill by name
    ]
    
    MEDIUM_RISK_PATTERNS = [
        r'\brm\s+-rf\s+[^/\s]',                  # rm -rf something specific
        r'\bsudo\s+.*',                          # any sudo command
        r'\bchown\s+-R',                         # recursive ownership change
        r'\bchmod\s+-R',                         # recursive permission change
        r'\bcrontab\s+-r',                       # remove all cron jobs
        r'\biptables\s+-F',                      # flush firewall rules
        r'\bkill\s+-9',                          # force kill process
        r'\bumount\s+.*force',                   # force unmount
    ]
    
    LOW_RISK_PATTERNS = [
        r'\bapt\s+remove',                       # package removal
        r'\byum\s+remove',                       # package removal
        r'\bpip\s+uninstall',                    # python package removal
        r'\bnpm\s+uninstall',                    # node package removal
        r'\bgit\s+reset\s+--hard',               # git hard reset
        r'\bgit\s+clean\s+-fd',                  # git force clean
        r'\bmv\s+.*\s+/tmp',                     # moving to tmp
    ]
    
    def analyse_command(self, command: str) -> SafetyResult:
        command = command.strip().lower()
        warning = []
        suggestions = []

        for pattern in self.CRITICAL_PATTERNS:
            if re.search(pattern, command, re.IGNORECASE):
                warning.append("CRITICAL: This command can cause irreversible system damage!")
                return SafetyResult(
                    risk_level=RiskLevel.CRITICAL,
                    warning=warning,
                    blocked=True,
                    suggestions=suggestions
                )

        for pattern in self.HIGH_RISK_PATTERNS:
            if re.search(pattern, command, re.IGNORECASE):
                warning.append("HIGH RISK: This command can cause significant damage or data loss")
                if "rm -rf" in command:
                    suggestions.append("Consider using 'trash' or 'mv to backup' instead")
                if "sudo" in command:
                    suggestions.append("Double-check you need elevated privileges")
                return SafetyResult(
                    risk_level=RiskLevel.HIGH,
                    warning=warning,
                    blocked=True,
                    suggestions=suggestions
                )

        for pattern in self.MEDIUM_RISK_PATTERNS:
            if re.search(pattern, command, re.IGNORECASE):
                warning.append("MEDIUM RISK: This command can cause significant changes")
                return SafetyResult(
                    risk_level=RiskLevel.MEDIUM,
                    warning=warning,
                    blocked=False,
                    suggestions=suggestions
                )

        for pattern in self.LOW_RISK_PATTERNS:
            if re.search(pattern, command, re.IGNORECASE):
                warning.append("LOW RISK: This command may cause data loss or changes")
                return SafetyResult(
                    risk_level=RiskLevel.LOW,
                    warning=warning,
                    blocked=False,
                    suggestions=suggestions
                )
        
        return SafetyResult(
            risk_level= RiskLevel.SAFE,
            warning=[],
            suggestions=[]
        )

    def get_confirmation_message(self, safety_result: SafetyResult) -> str:
        if safety_result.risk_level == RiskLevel.CRITICAL:
            return "BLOCKED: Command is too dangerous to execute"
        elif safety_result.risk_level == RiskLevel.HIGH:
            return "HIGH RISK: Type 'YES' (in caps) to confirm"
        elif safety_result.risk_level == RiskLevel.MEDIUM:
            return "MEDIUM RISK: Type 'yes' to confirm"
        elif safety_result.risk_level == RiskLevel.LOW:
            return "Are you sure? [y/N]"
        else:
            return "Run this command? [y/N]"

    def validate_confirmation(self, confirmation: str, risk_level: RiskLevel) -> bool:
        confirmation = confirmation.strip()
        
        if risk_level == RiskLevel.CRITICAL:
            return False  # Never allow critical commands
        elif risk_level == RiskLevel.HIGH:
            return confirmation == "YES"
        elif risk_level == RiskLevel.MEDIUM:
            return confirmation == "yes"
        elif risk_level == RiskLevel.LOW:
            return confirmation.lower() in ["y", "yes"]
        else:
            return confirmation.lower() in ["y", "yes"]

def check_command_safety(cmd: str) -> bool:
    safety_result = CommandSafety().analyse_command(cmd)
    
    if safety_result.blocked and safety_result.risk_level == RiskLevel.CRITICAL:
        print(f"[red]{safety_result.warning[0]}[/red]")
        if safety_result.suggestions:
            print("[yellow]Suggestions:[/yellow]")
            for suggestion in safety_result.suggestions:
                print(f"  • {suggestion}")
        return False
    
    if safety_result.warning:
        print("\n[bold yellow]Safety Warning:[/bold yellow]")
        for warning in safety_result.warning:
            print(f"[yellow]{warning}[/yellow]")
        
        if safety_result.suggestions:
            print("[cyan]Suggestions:[/cyan]")
            for suggestion in safety_result.suggestions:
                print(f"  • {suggestion}")
    
    if safety_result.risk_level != RiskLevel.SAFE:
        confirm_msg = CommandSafety().get_confirmation_message(safety_result)
        print(f"\n[bold]{confirm_msg}[/bold]")
        
        user_confirm = input("> ").strip()
        return CommandSafety().validate_confirmation(user_confirm, safety_result.risk_level)
    
    return True
    

================================================================================
// File: terminal/utils/__init__.py
================================================================================


================================================================================
// File: terminal/utils/config.py
================================================================================
import json
import os
from dataclasses import dataclass
from pathlib import Path

DEFAULT_CONFIG_PATH = Path.home() / ".dwarp_config.json"

DEFAULTS = {
    "gemini_api_key": None,
    "model": "gemini-2.5-flash",
    "max_tokens": 10000,
    "temperature": 0.7,
    "safety_settings": {
        "harassment": "BLOCK_MEDIUM_AND_ABOVE",
        "hate_speech": "BLOCK_MEDIUM_AND_ABOVE",
        "dangerous_content": "BLOCK_MEDIUM_AND_ABOVE",
        "sexual_content": "BLOCK_MEDIUM_AND_ABOVE",
    },
}


@dataclass
class Config:
    config_file: Path = DEFAULT_CONFIG_PATH
    model_override: str | None = None

    def __post_init__(self):
        self.config_file = Path(self.config_file)

    def load(self) -> dict:
        if not self.config_file.exists():
            return DEFAULTS.copy()
        try:
            with open(self.config_file) as f:
                return {**DEFAULTS, **json.load(f)}
        except Exception as e:
            print(f"Warning: Could not load config file: {e}")
            return DEFAULTS.copy()

    def save(self, data: dict) -> bool:
        try:
            with open(self.config_file, "w") as f:
                json.dump(data, f, indent=2)
            os.chmod(self.config_file, mode=0o600)
            return True
        except Exception as e:
            print(f"Error saving config: {e}")
            return False

    def get_api_key(self) -> str:
        api_key = os.getenv("GEMINI_API_KEY")
        if api_key:
            return api_key

        data = self.load()
        if data.get("gemini_api_key"):
            return data["gemini_api_key"]

        print("\nGemini API Key Required")
        print("To use AI features, you need a Gemini API key from Google AI Studio.")
        print("Get one at: https://aistudio.google.com/app/apikey\n")

        while True:
            api_key = input("Enter your Gemini API key: ").strip()
            if api_key and len(api_key) > 10:
                data["gemini_api_key"] = api_key
                if self.save(data):
                    print("API key saved securely!")
                return api_key
            print("Invalid API key. Please try again.")

    def get_model_config(self) -> dict:
        data = self.load()
        return {
            "model": self.model_override or data.get("model", "gemini-2.5-flash"),
            "max_tokens": data.get("max_tokens", 4000),
            "temperature": data.get("temperature", 0.7),
            "safety_settings": data.get("safety_settings", DEFAULTS["safety_settings"]),
        }


config = Config()

================================================================================
// File: terminal/utils/loading.py
================================================================================
from rich.console import Console
from rich.spinner import Spinner
from rich.live import Live

console = Console()


class LoadingAnimation:
    def __init__(self, text: str = "Thinking"):
        self.live = Live(Spinner("dots", text=text), console=console, refresh_per_second=10)

    def start(self):
        self.live.start()

    def stop(self):
        self.live.stop()

================================================================================
// File: terminal/utils/parsers.py
================================================================================
import json
import re


def parse_json(text: str) -> dict | None:
    if not text:
        return None

    text = re.sub(r"```json\s*", "", text)
    text = re.sub(r"```\s*$", "", text)
    text = text.strip()

    json_match = re.search(r"\{.*\}", text, flags=re.DOTALL)
    if not json_match:
        return None

    try:
        return json.loads(json_match.group())
    except json.JSONDecodeError:
        pass

    # Fallback: extract content field via regex
    content_match = re.search(
        r'"content"\s*:\s*"([^"]*(?:\\.[^"]*)*)"', text, re.DOTALL
    )
    if content_match:
        return {
            "content": content_match.group(1),
            "response_type": "general_query",
            "action_required": False,
            "suggested_command": None,
        }
    return None


def parse_response_parts(parts) -> dict | None:
    full_text = "".join(part.text for part in parts if hasattr(part, "text") and part.text)
    return parse_json(full_text)


def handle_function_call(part, response_type, action_required=True):
    fc = getattr(part, "function_call", None)
    args = getattr(fc, "args", None) if fc else None
    if not args:
        return None
    return {
        "content": f"**Command:** {args.get('command', '')}\n\n**Explanation:** {args.get('explanation', '')}",
        "response_type": response_type,
        "action_required": action_required,
        "suggested_command": args.get("command", ""),
    }

================================================================================
// File: uv.lock
================================================================================
version = 1
revision = 2
requires-python = ">=3.12"

[[package]]
name = "annotated-types"
version = "0.7.0"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/ee/67/531ea369ba64dcff5ec9c3402f9f51bf748cec26dde048a2f973a4eea7f5/annotated_types-0.7.0.tar.gz", hash = "sha256:aff07c09a53a08bc8cfccb9c85b05f1aa9a2a6f23728d790723543408344ce89", size = 16081, upload-time = "2024-05-20T21:33:25.928Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/78/b6/6307fbef88d9b5ee7421e68d78a9f162e0da4900bc5f5793f6d3d0e34fb8/annotated_types-0.7.0-py3-none-any.whl", hash = "sha256:1f02e8b43a8fbbc3f3e0d4f0f4bfc8131bcb4eebe8849b8e5c773f3a1c582a53", size = 13643, upload-time = "2024-05-20T21:33:24.1Z" },
]

[[package]]
name = "anyio"
version = "4.10.0"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "idna" },
    { name = "sniffio" },
    { name = "typing-extensions", marker = "python_full_version < '3.13'" },
]
sdist = { url = "https://files.pythonhosted.org/packages/f1/b4/636b3b65173d3ce9a38ef5f0522789614e590dab6a8d505340a4efe4c567/anyio-4.10.0.tar.gz", hash = "sha256:3f3fae35c96039744587aa5b8371e7e8e603c0702999535961dd336026973ba6", size = 213252, upload-time = "2025-08-04T08:54:26.451Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/6f/12/e5e0282d673bb9746bacfb6e2dba8719989d3660cdb2ea79aee9a9651afb/anyio-4.10.0-py3-none-any.whl", hash = "sha256:60e474ac86736bbfd6f210f7a61218939c318f43f9972497381f1c5e930ed3d1", size = 107213, upload-time = "2025-08-04T08:54:24.882Z" },
]

[[package]]
name = "build"
version = "1.4.0"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "colorama", marker = "os_name == 'nt'" },
    { name = "packaging" },
    { name = "pyproject-hooks" },
]
sdist = { url = "https://files.pythonhosted.org/packages/42/18/94eaffda7b329535d91f00fe605ab1f1e5cd68b2074d03f255c7d250687d/build-1.4.0.tar.gz", hash = "sha256:f1b91b925aa322be454f8330c6fb48b465da993d1e7e7e6fa35027ec49f3c936", size = 50054, upload-time = "2026-01-08T16:41:47.696Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/c5/0d/84a4380f930db0010168e0aa7b7a8fed9ba1835a8fbb1472bc6d0201d529/build-1.4.0-py3-none-any.whl", hash = "sha256:6a07c1b8eb6f2b311b96fcbdbce5dab5fe637ffda0fd83c9cac622e927501596", size = 24141, upload-time = "2026-01-08T16:41:46.453Z" },
]

[[package]]
name = "cachetools"
version = "5.5.2"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/6c/81/3747dad6b14fa2cf53fcf10548cf5aea6913e96fab41a3c198676f8948a5/cachetools-5.5.2.tar.gz", hash = "sha256:1a661caa9175d26759571b2e19580f9d6393969e5dfca11fdb1f947a23e640d4", size = 28380, upload-time = "2025-02-20T21:01:19.524Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/72/76/20fa66124dbe6be5cafeb312ece67de6b61dd91a0247d1ea13db4ebb33c2/cachetools-5.5.2-py3-none-any.whl", hash = "sha256:d26a22bcc62eb95c3beabd9f1ee5e820d3d2704fe2967cbe350e20c8ffcd3f0a", size = 10080, upload-time = "2025-02-20T21:01:16.647Z" },
]

[[package]]
name = "certifi"
version = "2025.8.3"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/dc/67/960ebe6bf230a96cda2e0abcf73af550ec4f090005363542f0765df162e0/certifi-2025.8.3.tar.gz", hash = "sha256:e564105f78ded564e3ae7c923924435e1daa7463faeab5bb932bc53ffae63407", size = 162386, upload-time = "2025-08-03T03:07:47.08Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/e5/48/1549795ba7742c948d2ad169c1c8cdbae65bc450d6cd753d124b17c8cd32/certifi-2025.8.3-py3-none-any.whl", hash = "sha256:f6c12493cfb1b06ba2ff328595af9350c65d6644968e5d3a2ffd78699af217a5", size = 161216, upload-time = "2025-08-03T03:07:45.777Z" },
]

[[package]]
name = "charset-normalizer"
version = "3.4.3"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/83/2d/5fd176ceb9b2fc619e63405525573493ca23441330fcdaee6bef9460e924/charset_normalizer-3.4.3.tar.gz", hash = "sha256:6fce4b8500244f6fcb71465d4a4930d132ba9ab8e71a7859e6a5d59851068d14", size = 122371, upload-time = "2025-08-09T07:57:28.46Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/e9/5e/14c94999e418d9b87682734589404a25854d5f5d0408df68bc15b6ff54bb/charset_normalizer-3.4.3-cp312-cp312-macosx_10_13_universal2.whl", hash = "sha256:e28e334d3ff134e88989d90ba04b47d84382a828c061d0d1027b1b12a62b39b1", size = 205655, upload-time = "2025-08-09T07:56:08.475Z" },
    { url = "https://files.pythonhosted.org/packages/7d/a8/c6ec5d389672521f644505a257f50544c074cf5fc292d5390331cd6fc9c3/charset_normalizer-3.4.3-cp312-cp312-manylinux2014_aarch64.manylinux_2_17_aarch64.manylinux_2_28_aarch64.whl", hash = "sha256:0cacf8f7297b0c4fcb74227692ca46b4a5852f8f4f24b3c766dd94a1075c4884", size = 146223, upload-time = "2025-08-09T07:56:09.708Z" },
    { url = "https://files.pythonhosted.org/packages/fc/eb/a2ffb08547f4e1e5415fb69eb7db25932c52a52bed371429648db4d84fb1/charset_normalizer-3.4.3-cp312-cp312-manylinux2014_ppc64le.manylinux_2_17_ppc64le.manylinux_2_28_ppc64le.whl", hash = "sha256:c6fd51128a41297f5409deab284fecbe5305ebd7e5a1f959bee1c054622b7018", size = 159366, upload-time = "2025-08-09T07:56:11.326Z" },
    { url = "https://files.pythonhosted.org/packages/82/10/0fd19f20c624b278dddaf83b8464dcddc2456cb4b02bb902a6da126b87a1/charset_normalizer-3.4.3-cp312-cp312-manylinux2014_s390x.manylinux_2_17_s390x.manylinux_2_28_s390x.whl", hash = "sha256:3cfb2aad70f2c6debfbcb717f23b7eb55febc0bb23dcffc0f076009da10c6392", size = 157104, upload-time = "2025-08-09T07:56:13.014Z" },
    { url = "https://files.pythonhosted.org/packages/16/ab/0233c3231af734f5dfcf0844aa9582d5a1466c985bbed6cedab85af9bfe3/charset_normalizer-3.4.3-cp312-cp312-manylinux2014_x86_64.manylinux_2_17_x86_64.manylinux_2_28_x86_64.whl", hash = "sha256:1606f4a55c0fd363d754049cdf400175ee96c992b1f8018b993941f221221c5f", size = 151830, upload-time = "2025-08-09T07:56:14.428Z" },
    { url = "https://files.pythonhosted.org/packages/ae/02/e29e22b4e02839a0e4a06557b1999d0a47db3567e82989b5bb21f3fbbd9f/charset_normalizer-3.4.3-cp312-cp312-musllinux_1_2_aarch64.whl", hash = "sha256:027b776c26d38b7f15b26a5da1044f376455fb3766df8fc38563b4efbc515154", size = 148854, upload-time = "2025-08-09T07:56:16.051Z" },
    { url = "https://files.pythonhosted.org/packages/05/6b/e2539a0a4be302b481e8cafb5af8792da8093b486885a1ae4d15d452bcec/charset_normalizer-3.4.3-cp312-cp312-musllinux_1_2_ppc64le.whl", hash = "sha256:42e5088973e56e31e4fa58eb6bd709e42fc03799c11c42929592889a2e54c491", size = 160670, upload-time = "2025-08-09T07:56:17.314Z" },
    { url = "https://files.pythonhosted.org/packages/31/e7/883ee5676a2ef217a40ce0bffcc3d0dfbf9e64cbcfbdf822c52981c3304b/charset_normalizer-3.4.3-cp312-cp312-musllinux_1_2_s390x.whl", hash = "sha256:cc34f233c9e71701040d772aa7490318673aa7164a0efe3172b2981218c26d93", size = 158501, upload-time = "2025-08-09T07:56:18.641Z" },
    { url = "https://files.pythonhosted.org/packages/c1/35/6525b21aa0db614cf8b5792d232021dca3df7f90a1944db934efa5d20bb1/charset_normalizer-3.4.3-cp312-cp312-musllinux_1_2_x86_64.whl", hash = "sha256:320e8e66157cc4e247d9ddca8e21f427efc7a04bbd0ac8a9faf56583fa543f9f", size = 153173, upload-time = "2025-08-09T07:56:20.289Z" },
    { url = "https://files.pythonhosted.org/packages/50/ee/f4704bad8201de513fdc8aac1cabc87e38c5818c93857140e06e772b5892/charset_normalizer-3.4.3-cp312-cp312-win32.whl", hash = "sha256:fb6fecfd65564f208cbf0fba07f107fb661bcd1a7c389edbced3f7a493f70e37", size = 99822, upload-time = "2025-08-09T07:56:21.551Z" },
    { url = "https://files.pythonhosted.org/packages/39/f5/3b3836ca6064d0992c58c7561c6b6eee1b3892e9665d650c803bd5614522/charset_normalizer-3.4.3-cp312-cp312-win_amd64.whl", hash = "sha256:86df271bf921c2ee3818f0522e9a5b8092ca2ad8b065ece5d7d9d0e9f4849bcc", size = 107543, upload-time = "2025-08-09T07:56:23.115Z" },
    { url = "https://files.pythonhosted.org/packages/65/ca/2135ac97709b400c7654b4b764daf5c5567c2da45a30cdd20f9eefe2d658/charset_normalizer-3.4.3-cp313-cp313-macosx_10_13_universal2.whl", hash = "sha256:14c2a87c65b351109f6abfc424cab3927b3bdece6f706e4d12faaf3d52ee5efe", size = 205326, upload-time = "2025-08-09T07:56:24.721Z" },
    { url = "https://files.pythonhosted.org/packages/71/11/98a04c3c97dd34e49c7d247083af03645ca3730809a5509443f3c37f7c99/charset_normalizer-3.4.3-cp313-cp313-manylinux2014_aarch64.manylinux_2_17_aarch64.manylinux_2_28_aarch64.whl", hash = "sha256:41d1fc408ff5fdfb910200ec0e74abc40387bccb3252f3f27c0676731df2b2c8", size = 146008, upload-time = "2025-08-09T07:56:26.004Z" },
    { url = "https://files.pythonhosted.org/packages/60/f5/4659a4cb3c4ec146bec80c32d8bb16033752574c20b1252ee842a95d1a1e/charset_normalizer-3.4.3-cp313-cp313-manylinux2014_ppc64le.manylinux_2_17_ppc64le.manylinux_2_28_ppc64le.whl", hash = "sha256:1bb60174149316da1c35fa5233681f7c0f9f514509b8e399ab70fea5f17e45c9", size = 159196, upload-time = "2025-08-09T07:56:27.25Z" },
    { url = "https://files.pythonhosted.org/packages/86/9e/f552f7a00611f168b9a5865a1414179b2c6de8235a4fa40189f6f79a1753/charset_normalizer-3.4.3-cp313-cp313-manylinux2014_s390x.manylinux_2_17_s390x.manylinux_2_28_s390x.whl", hash = "sha256:30d006f98569de3459c2fc1f2acde170b7b2bd265dc1943e87e1a4efe1b67c31", size = 156819, upload-time = "2025-08-09T07:56:28.515Z" },
    { url = "https://files.pythonhosted.org/packages/7e/95/42aa2156235cbc8fa61208aded06ef46111c4d3f0de233107b3f38631803/charset_normalizer-3.4.3-cp313-cp313-manylinux2014_x86_64.manylinux_2_17_x86_64.manylinux_2_28_x86_64.whl", hash = "sha256:416175faf02e4b0810f1f38bcb54682878a4af94059a1cd63b8747244420801f", size = 151350, upload-time = "2025-08-09T07:56:29.716Z" },
    { url = "https://files.pythonhosted.org/packages/c2/a9/3865b02c56f300a6f94fc631ef54f0a8a29da74fb45a773dfd3dcd380af7/charset_normalizer-3.4.3-cp313-cp313-musllinux_1_2_aarch64.whl", hash = "sha256:6aab0f181c486f973bc7262a97f5aca3ee7e1437011ef0c2ec04b5a11d16c927", size = 148644, upload-time = "2025-08-09T07:56:30.984Z" },
    { url = "https://files.pythonhosted.org/packages/77/d9/cbcf1a2a5c7d7856f11e7ac2d782aec12bdfea60d104e60e0aa1c97849dc/charset_normalizer-3.4.3-cp313-cp313-musllinux_1_2_ppc64le.whl", hash = "sha256:fdabf8315679312cfa71302f9bd509ded4f2f263fb5b765cf1433b39106c3cc9", size = 160468, upload-time = "2025-08-09T07:56:32.252Z" },
    { url = "https://files.pythonhosted.org/packages/f6/42/6f45efee8697b89fda4d50580f292b8f7f9306cb2971d4b53f8914e4d890/charset_normalizer-3.4.3-cp313-cp313-musllinux_1_2_s390x.whl", hash = "sha256:bd28b817ea8c70215401f657edef3a8aa83c29d447fb0b622c35403780ba11d5", size = 158187, upload-time = "2025-08-09T07:56:33.481Z" },
    { url = "https://files.pythonhosted.org/packages/70/99/f1c3bdcfaa9c45b3ce96f70b14f070411366fa19549c1d4832c935d8e2c3/charset_normalizer-3.4.3-cp313-cp313-musllinux_1_2_x86_64.whl", hash = "sha256:18343b2d246dc6761a249ba1fb13f9ee9a2bcd95decc767319506056ea4ad4dc", size = 152699, upload-time = "2025-08-09T07:56:34.739Z" },
    { url = "https://files.pythonhosted.org/packages/a3/ad/b0081f2f99a4b194bcbb1934ef3b12aa4d9702ced80a37026b7607c72e58/charset_normalizer-3.4.3-cp313-cp313-win32.whl", hash = "sha256:6fb70de56f1859a3f71261cbe41005f56a7842cc348d3aeb26237560bfa5e0ce", size = 99580, upload-time = "2025-08-09T07:56:35.981Z" },
    { url = "https://files.pythonhosted.org/packages/9a/8f/ae790790c7b64f925e5c953b924aaa42a243fb778fed9e41f147b2a5715a/charset_normalizer-3.4.3-cp313-cp313-win_amd64.whl", hash = "sha256:cf1ebb7d78e1ad8ec2a8c4732c7be2e736f6e5123a4146c5b89c9d1f585f8cef", size = 107366, upload-time = "2025-08-09T07:56:37.339Z" },
    { url = "https://files.pythonhosted.org/packages/8e/91/b5a06ad970ddc7a0e513112d40113e834638f4ca1120eb727a249fb2715e/charset_normalizer-3.4.3-cp314-cp314-macosx_10_13_universal2.whl", hash = "sha256:3cd35b7e8aedeb9e34c41385fda4f73ba609e561faedfae0a9e75e44ac558a15", size = 204342, upload-time = "2025-08-09T07:56:38.687Z" },
    { url = "https://files.pythonhosted.org/packages/ce/ec/1edc30a377f0a02689342f214455c3f6c2fbedd896a1d2f856c002fc3062/charset_normalizer-3.4.3-cp314-cp314-manylinux2014_aarch64.manylinux_2_17_aarch64.manylinux_2_28_aarch64.whl", hash = "sha256:b89bc04de1d83006373429975f8ef9e7932534b8cc9ca582e4db7d20d91816db", size = 145995, upload-time = "2025-08-09T07:56:40.048Z" },
    { url = "https://files.pythonhosted.org/packages/17/e5/5e67ab85e6d22b04641acb5399c8684f4d37caf7558a53859f0283a650e9/charset_normalizer-3.4.3-cp314-cp314-manylinux2014_ppc64le.manylinux_2_17_ppc64le.manylinux_2_28_ppc64le.whl", hash = "sha256:2001a39612b241dae17b4687898843f254f8748b796a2e16f1051a17078d991d", size = 158640, upload-time = "2025-08-09T07:56:41.311Z" },
    { url = "https://files.pythonhosted.org/packages/f1/e5/38421987f6c697ee3722981289d554957c4be652f963d71c5e46a262e135/charset_normalizer-3.4.3-cp314-cp314-manylinux2014_s390x.manylinux_2_17_s390x.manylinux_2_28_s390x.whl", hash = "sha256:8dcfc373f888e4fb39a7bc57e93e3b845e7f462dacc008d9749568b1c4ece096", size = 156636, upload-time = "2025-08-09T07:56:43.195Z" },
    { url = "https://files.pythonhosted.org/packages/a0/e4/5a075de8daa3ec0745a9a3b54467e0c2967daaaf2cec04c845f73493e9a1/charset_normalizer-3.4.3-cp314-cp314-manylinux2014_x86_64.manylinux_2_17_x86_64.manylinux_2_28_x86_64.whl", hash = "sha256:18b97b8404387b96cdbd30ad660f6407799126d26a39ca65729162fd810a99aa", size = 150939, upload-time = "2025-08-09T07:56:44.819Z" },
    { url = "https://files.pythonhosted.org/packages/02/f7/3611b32318b30974131db62b4043f335861d4d9b49adc6d57c1149cc49d4/charset_normalizer-3.4.3-cp314-cp314-musllinux_1_2_aarch64.whl", hash = "sha256:ccf600859c183d70eb47e05a44cd80a4ce77394d1ac0f79dbd2dd90a69a3a049", size = 148580, upload-time = "2025-08-09T07:56:46.684Z" },
    { url = "https://files.pythonhosted.org/packages/7e/61/19b36f4bd67f2793ab6a99b979b4e4f3d8fc754cbdffb805335df4337126/charset_normalizer-3.4.3-cp314-cp314-musllinux_1_2_ppc64le.whl", hash = "sha256:53cd68b185d98dde4ad8990e56a58dea83a4162161b1ea9272e5c9182ce415e0", size = 159870, upload-time = "2025-08-09T07:56:47.941Z" },
    { url = "https://files.pythonhosted.org/packages/06/57/84722eefdd338c04cf3030ada66889298eaedf3e7a30a624201e0cbe424a/charset_normalizer-3.4.3-cp314-cp314-musllinux_1_2_s390x.whl", hash = "sha256:30a96e1e1f865f78b030d65241c1ee850cdf422d869e9028e2fc1d5e4db73b92", size = 157797, upload-time = "2025-08-09T07:56:49.756Z" },
    { url = "https://files.pythonhosted.org/packages/72/2a/aff5dd112b2f14bcc3462c312dce5445806bfc8ab3a7328555da95330e4b/charset_normalizer-3.4.3-cp314-cp314-musllinux_1_2_x86_64.whl", hash = "sha256:d716a916938e03231e86e43782ca7878fb602a125a91e7acb8b5112e2e96ac16", size = 152224, upload-time = "2025-08-09T07:56:51.369Z" },
    { url = "https://files.pythonhosted.org/packages/b7/8c/9839225320046ed279c6e839d51f028342eb77c91c89b8ef2549f951f3ec/charset_normalizer-3.4.3-cp314-cp314-win32.whl", hash = "sha256:c6dbd0ccdda3a2ba7c2ecd9d77b37f3b5831687d8dc1b6ca5f56a4880cc7b7ce", size = 100086, upload-time = "2025-08-09T07:56:52.722Z" },
    { url = "https://files.pythonhosted.org/packages/ee/7a/36fbcf646e41f710ce0a563c1c9a343c6edf9be80786edeb15b6f62e17db/charset_normalizer-3.4.3-cp314-cp314-win_amd64.whl", hash = "sha256:73dc19b562516fc9bcf6e5d6e596df0b4eb98d87e4f79f3ae71840e6ed21361c", size = 107400, upload-time = "2025-08-09T07:56:55.172Z" },
    { url = "https://files.pythonhosted.org/packages/8a/1f/f041989e93b001bc4e44bb1669ccdcf54d3f00e628229a85b08d330615c5/charset_normalizer-3.4.3-py3-none-any.whl", hash = "sha256:ce571ab16d890d23b5c278547ba694193a45011ff86a9162a71307ed9f86759a", size = 53175, upload-time = "2025-08-09T07:57:26.864Z" },
]

[[package]]
name = "colorama"
version = "0.4.6"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/d8/53/6f443c9a4a8358a93a6792e2acffb9d9d5cb0a5cfd8802644b7b1c9a02e4/colorama-0.4.6.tar.gz", hash = "sha256:08695f5cb7ed6e0531a20572697297273c47b8cae5a63ffc6d6ed5c201be6e44", size = 27697, upload-time = "2022-10-25T02:36:22.414Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/d1/d6/3965ed04c63042e047cb6a3e6ed1a63a35087b6a609aa3a15ed8ac56c221/colorama-0.4.6-py2.py3-none-any.whl", hash = "sha256:4f1d9991f5acc0ca119f9d443620b77f9d6b33703e51011c16baf57afb285fc6", size = 25335, upload-time = "2022-10-25T02:36:20.889Z" },
]

[[package]]
name = "dwarp"
version = "0.1.8.1"
source = { editable = "." }
dependencies = [
    { name = "build" },
    { name = "google-genai" },
    { name = "prompt-toolkit" },
    { name = "python-dotenv" },
    { name = "rich" },
    { name = "ruff" },
]

[package.metadata]
requires-dist = [
    { name = "build", specifier = ">=1.4.0" },
    { name = "google-genai", specifier = "==1.31.0" },
    { name = "prompt-toolkit", specifier = "==3.0.51" },
    { name = "python-dotenv", specifier = "==1.1.1" },
    { name = "rich", specifier = "==14.1.0" },
    { name = "ruff", specifier = ">=0.15.5" },
]

[[package]]
name = "google-auth"
version = "2.40.3"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "cachetools" },
    { name = "pyasn1-modules" },
    { name = "rsa" },
]
sdist = { url = "https://files.pythonhosted.org/packages/9e/9b/e92ef23b84fa10a64ce4831390b7a4c2e53c0132568d99d4ae61d04c8855/google_auth-2.40.3.tar.gz", hash = "sha256:500c3a29adedeb36ea9cf24b8d10858e152f2412e3ca37829b3fa18e33d63b77", size = 281029, upload-time = "2025-06-04T18:04:57.577Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/17/63/b19553b658a1692443c62bd07e5868adaa0ad746a0751ba62c59568cd45b/google_auth-2.40.3-py2.py3-none-any.whl", hash = "sha256:1370d4593e86213563547f97a92752fc658456fe4514c809544f330fed45a7ca", size = 216137, upload-time = "2025-06-04T18:04:55.573Z" },
]

[[package]]
name = "google-genai"
version = "1.31.0"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "anyio" },
    { name = "google-auth" },
    { name = "httpx" },
    { name = "pydantic" },
    { name = "requests" },
    { name = "tenacity" },
    { name = "typing-extensions" },
    { name = "websockets" },
]
sdist = { url = "https://files.pythonhosted.org/packages/e0/1b/da30fa6e2966942d7028a58eb7aa7d04544dcc3aa66194365b2e0adac570/google_genai-1.31.0.tar.gz", hash = "sha256:8572b47aa684357c3e5e10d290ec772c65414114939e3ad2955203e27cd2fcbc", size = 233482, upload-time = "2025-08-18T23:40:21.733Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/41/27/1525bc9cbec58660f0842ebcbfe910a1dde908c2672373804879666e0bb8/google_genai-1.31.0-py3-none-any.whl", hash = "sha256:5c6959bcf862714e8ed0922db3aaf41885bacf6318751b3421bf1e459f78892f", size = 231876, upload-time = "2025-08-18T23:40:20.385Z" },
]

[[package]]
name = "h11"
version = "0.16.0"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/01/ee/02a2c011bdab74c6fb3c75474d40b3052059d95df7e73351460c8588d963/h11-0.16.0.tar.gz", hash = "sha256:4e35b956cf45792e4caa5885e69fba00bdbc6ffafbfa020300e549b208ee5ff1", size = 101250, upload-time = "2025-04-24T03:35:25.427Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/04/4b/29cac41a4d98d144bf5f6d33995617b185d14b22401f75ca86f384e87ff1/h11-0.16.0-py3-none-any.whl", hash = "sha256:63cf8bbe7522de3bf65932fda1d9c2772064ffb3dae62d55932da54b31cb6c86", size = 37515, upload-time = "2025-04-24T03:35:24.344Z" },
]

[[package]]
name = "httpcore"
version = "1.0.9"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "certifi" },
    { name = "h11" },
]
sdist = { url = "https://files.pythonhosted.org/packages/06/94/82699a10bca87a5556c9c59b5963f2d039dbd239f25bc2a63907a05a14cb/httpcore-1.0.9.tar.gz", hash = "sha256:6e34463af53fd2ab5d807f399a9b45ea31c3dfa2276f15a2c3f00afff6e176e8", size = 85484, upload-time = "2025-04-24T22:06:22.219Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/7e/f5/f66802a942d491edb555dd61e3a9961140fd64c90bce1eafd741609d334d/httpcore-1.0.9-py3-none-any.whl", hash = "sha256:2d400746a40668fc9dec9810239072b40b4484b640a8c38fd654a024c7a1bf55", size = 78784, upload-time = "2025-04-24T22:06:20.566Z" },
]

[[package]]
name = "httpx"
version = "0.28.1"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "anyio" },
    { name = "certifi" },
    { name = "httpcore" },
    { name = "idna" },
]
sdist = { url = "https://files.pythonhosted.org/packages/b1/df/48c586a5fe32a0f01324ee087459e112ebb7224f646c0b5023f5e79e9956/httpx-0.28.1.tar.gz", hash = "sha256:75e98c5f16b0f35b567856f597f06ff2270a374470a5c2392242528e3e3e42fc", size = 141406, upload-time = "2024-12-06T15:37:23.222Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/2a/39/e50c7c3a983047577ee07d2a9e53faf5a69493943ec3f6a384bdc792deb2/httpx-0.28.1-py3-none-any.whl", hash = "sha256:d909fcccc110f8c7faf814ca82a9a4d816bc5a6dbfea25d6591d6985b8ba59ad", size = 73517, upload-time = "2024-12-06T15:37:21.509Z" },
]

[[package]]
name = "idna"
version = "3.10"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/f1/70/7703c29685631f5a7590aa73f1f1d3fa9a380e654b86af429e0934a32f7d/idna-3.10.tar.gz", hash = "sha256:12f65c9b470abda6dc35cf8e63cc574b1c52b11df2c86030af0ac09b01b13ea9", size = 190490, upload-time = "2024-09-15T18:07:39.745Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/76/c6/c88e154df9c4e1a2a66ccf0005a88dfb2650c1dffb6f5ce603dfbd452ce3/idna-3.10-py3-none-any.whl", hash = "sha256:946d195a0d259cbba61165e88e65941f16e9b36ea6ddb97f00452bae8b1287d3", size = 70442, upload-time = "2024-09-15T18:07:37.964Z" },
]

[[package]]
name = "markdown-it-py"
version = "4.0.0"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "mdurl" },
]
sdist = { url = "https://files.pythonhosted.org/packages/5b/f5/4ec618ed16cc4f8fb3b701563655a69816155e79e24a17b651541804721d/markdown_it_py-4.0.0.tar.gz", hash = "sha256:cb0a2b4aa34f932c007117b194e945bd74e0ec24133ceb5bac59009cda1cb9f3", size = 73070, upload-time = "2025-08-11T12:57:52.854Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/94/54/e7d793b573f298e1c9013b8c4dade17d481164aa517d1d7148619c2cedbf/markdown_it_py-4.0.0-py3-none-any.whl", hash = "sha256:87327c59b172c5011896038353a81343b6754500a08cd7a4973bb48c6d578147", size = 87321, upload-time = "2025-08-11T12:57:51.923Z" },
]

[[package]]
name = "mdurl"
version = "0.1.2"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/d6/54/cfe61301667036ec958cb99bd3efefba235e65cdeb9c84d24a8293ba1d90/mdurl-0.1.2.tar.gz", hash = "sha256:bb413d29f5eea38f31dd4754dd7377d4465116fb207585f97bf925588687c1ba", size = 8729, upload-time = "2022-08-14T12:40:10.846Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/b3/38/89ba8ad64ae25be8de66a6d463314cf1eb366222074cfda9ee839c56a4b4/mdurl-0.1.2-py3-none-any.whl", hash = "sha256:84008a41e51615a49fc9966191ff91509e3c40b939176e643fd50a5c2196b8f8", size = 9979, upload-time = "2022-08-14T12:40:09.779Z" },
]

[[package]]
name = "packaging"
version = "26.0"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/65/ee/299d360cdc32edc7d2cf530f3accf79c4fca01e96ffc950d8a52213bd8e4/packaging-26.0.tar.gz", hash = "sha256:00243ae351a257117b6a241061796684b084ed1c516a08c48a3f7e147a9d80b4", size = 143416, upload-time = "2026-01-21T20:50:39.064Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/b7/b9/c538f279a4e237a006a2c98387d081e9eb060d203d8ed34467cc0f0b9b53/packaging-26.0-py3-none-any.whl", hash = "sha256:b36f1fef9334a5588b4166f8bcd26a14e521f2b55e6b9de3aaa80d3ff7a37529", size = 74366, upload-time = "2026-01-21T20:50:37.788Z" },
]

[[package]]
name = "prompt-toolkit"
version = "3.0.51"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "wcwidth" },
]
sdist = { url = "https://files.pythonhosted.org/packages/bb/6e/9d084c929dfe9e3bfe0c6a47e31f78a25c54627d64a66e884a8bf5474f1c/prompt_toolkit-3.0.51.tar.gz", hash = "sha256:931a162e3b27fc90c86f1b48bb1fb2c528c2761475e57c9c06de13311c7b54ed", size = 428940, upload-time = "2025-04-15T09:18:47.731Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/ce/4f/5249960887b1fbe561d9ff265496d170b55a735b76724f10ef19f9e40716/prompt_toolkit-3.0.51-py3-none-any.whl", hash = "sha256:52742911fde84e2d423e2f9a4cf1de7d7ac4e51958f648d9540e0fb8db077b07", size = 387810, upload-time = "2025-04-15T09:18:44.753Z" },
]

[[package]]
name = "pyasn1"
version = "0.6.1"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/ba/e9/01f1a64245b89f039897cb0130016d79f77d52669aae6ee7b159a6c4c018/pyasn1-0.6.1.tar.gz", hash = "sha256:6f580d2bdd84365380830acf45550f2511469f673cb4a5ae3857a3170128b034", size = 145322, upload-time = "2024-09-10T22:41:42.55Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/c8/f1/d6a797abb14f6283c0ddff96bbdd46937f64122b8c925cab503dd37f8214/pyasn1-0.6.1-py3-none-any.whl", hash = "sha256:0d632f46f2ba09143da3a8afe9e33fb6f92fa2320ab7e886e2d0f7672af84629", size = 83135, upload-time = "2024-09-11T16:00:36.122Z" },
]

[[package]]
name = "pyasn1-modules"
version = "0.4.2"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "pyasn1" },
]
sdist = { url = "https://files.pythonhosted.org/packages/e9/e6/78ebbb10a8c8e4b61a59249394a4a594c1a7af95593dc933a349c8d00964/pyasn1_modules-0.4.2.tar.gz", hash = "sha256:677091de870a80aae844b1ca6134f54652fa2c8c5a52aa396440ac3106e941e6", size = 307892, upload-time = "2025-03-28T02:41:22.17Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/47/8d/d529b5d697919ba8c11ad626e835d4039be708a35b0d22de83a269a6682c/pyasn1_modules-0.4.2-py3-none-any.whl", hash = "sha256:29253a9207ce32b64c3ac6600edc75368f98473906e8fd1043bd6b5b1de2c14a", size = 181259, upload-time = "2025-03-28T02:41:19.028Z" },
]

[[package]]
name = "pydantic"
version = "2.11.7"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "annotated-types" },
    { name = "pydantic-core" },
    { name = "typing-extensions" },
    { name = "typing-inspection" },
]
sdist = { url = "https://files.pythonhosted.org/packages/00/dd/4325abf92c39ba8623b5af936ddb36ffcfe0beae70405d456ab1fb2f5b8c/pydantic-2.11.7.tar.gz", hash = "sha256:d989c3c6cb79469287b1569f7447a17848c998458d49ebe294e975b9baf0f0db", size = 788350, upload-time = "2025-06-14T08:33:17.137Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/6a/c0/ec2b1c8712ca690e5d61979dee872603e92b8a32f94cc1b72d53beab008a/pydantic-2.11.7-py3-none-any.whl", hash = "sha256:dde5df002701f6de26248661f6835bbe296a47bf73990135c7d07ce741b9623b", size = 444782, upload-time = "2025-06-14T08:33:14.905Z" },
]

[[package]]
name = "pydantic-core"
version = "2.33.2"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "typing-extensions" },
]
sdist = { url = "https://files.pythonhosted.org/packages/ad/88/5f2260bdfae97aabf98f1778d43f69574390ad787afb646292a638c923d4/pydantic_core-2.33.2.tar.gz", hash = "sha256:7cb8bc3605c29176e1b105350d2e6474142d7c1bd1d9327c4a9bdb46bf827acc", size = 435195, upload-time = "2025-04-23T18:33:52.104Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/18/8a/2b41c97f554ec8c71f2a8a5f85cb56a8b0956addfe8b0efb5b3d77e8bdc3/pydantic_core-2.33.2-cp312-cp312-macosx_10_12_x86_64.whl", hash = "sha256:a7ec89dc587667f22b6a0b6579c249fca9026ce7c333fc142ba42411fa243cdc", size = 2009000, upload-time = "2025-04-23T18:31:25.863Z" },
    { url = "https://files.pythonhosted.org/packages/a1/02/6224312aacb3c8ecbaa959897af57181fb6cf3a3d7917fd44d0f2917e6f2/pydantic_core-2.33.2-cp312-cp312-macosx_11_0_arm64.whl", hash = "sha256:3c6db6e52c6d70aa0d00d45cdb9b40f0433b96380071ea80b09277dba021ddf7", size = 1847996, upload-time = "2025-04-23T18:31:27.341Z" },
    { url = "https://files.pythonhosted.org/packages/d6/46/6dcdf084a523dbe0a0be59d054734b86a981726f221f4562aed313dbcb49/pydantic_core-2.33.2-cp312-cp312-manylinux_2_17_aarch64.manylinux2014_aarch64.whl", hash = "sha256:4e61206137cbc65e6d5256e1166f88331d3b6238e082d9f74613b9b765fb9025", size = 1880957, upload-time = "2025-04-23T18:31:28.956Z" },
    { url = "https://files.pythonhosted.org/packages/ec/6b/1ec2c03837ac00886ba8160ce041ce4e325b41d06a034adbef11339ae422/pydantic_core-2.33.2-cp312-cp312-manylinux_2_17_armv7l.manylinux2014_armv7l.whl", hash = "sha256:eb8c529b2819c37140eb51b914153063d27ed88e3bdc31b71198a198e921e011", size = 1964199, upload-time = "2025-04-23T18:31:31.025Z" },
    { url = "https://files.pythonhosted.org/packages/2d/1d/6bf34d6adb9debd9136bd197ca72642203ce9aaaa85cfcbfcf20f9696e83/pydantic_core-2.33.2-cp312-cp312-manylinux_2_17_ppc64le.manylinux2014_ppc64le.whl", hash = "sha256:c52b02ad8b4e2cf14ca7b3d918f3eb0ee91e63b3167c32591e57c4317e134f8f", size = 2120296, upload-time = "2025-04-23T18:31:32.514Z" },
    { url = "https://files.pythonhosted.org/packages/e0/94/2bd0aaf5a591e974b32a9f7123f16637776c304471a0ab33cf263cf5591a/pydantic_core-2.33.2-cp312-cp312-manylinux_2_17_s390x.manylinux2014_s390x.whl", hash = "sha256:96081f1605125ba0855dfda83f6f3df5ec90c61195421ba72223de35ccfb2f88", size = 2676109, upload-time = "2025-04-23T18:31:33.958Z" },
    { url = "https://files.pythonhosted.org/packages/f9/41/4b043778cf9c4285d59742281a769eac371b9e47e35f98ad321349cc5d61/pydantic_core-2.33.2-cp312-cp312-manylinux_2_17_x86_64.manylinux2014_x86_64.whl", hash = "sha256:8f57a69461af2a5fa6e6bbd7a5f60d3b7e6cebb687f55106933188e79ad155c1", size = 2002028, upload-time = "2025-04-23T18:31:39.095Z" },
    { url = "https://files.pythonhosted.org/packages/cb/d5/7bb781bf2748ce3d03af04d5c969fa1308880e1dca35a9bd94e1a96a922e/pydantic_core-2.33.2-cp312-cp312-manylinux_2_5_i686.manylinux1_i686.whl", hash = "sha256:572c7e6c8bb4774d2ac88929e3d1f12bc45714ae5ee6d9a788a9fb35e60bb04b", size = 2100044, upload-time = "2025-04-23T18:31:41.034Z" },
    { url = "https://files.pythonhosted.org/packages/fe/36/def5e53e1eb0ad896785702a5bbfd25eed546cdcf4087ad285021a90ed53/pydantic_core-2.33.2-cp312-cp312-musllinux_1_1_aarch64.whl", hash = "sha256:db4b41f9bd95fbe5acd76d89920336ba96f03e149097365afe1cb092fceb89a1", size = 2058881, upload-time = "2025-04-23T18:31:42.757Z" },
    { url = "https://files.pythonhosted.org/packages/01/6c/57f8d70b2ee57fc3dc8b9610315949837fa8c11d86927b9bb044f8705419/pydantic_core-2.33.2-cp312-cp312-musllinux_1_1_armv7l.whl", hash = "sha256:fa854f5cf7e33842a892e5c73f45327760bc7bc516339fda888c75ae60edaeb6", size = 2227034, upload-time = "2025-04-23T18:31:44.304Z" },
    { url = "https://files.pythonhosted.org/packages/27/b9/9c17f0396a82b3d5cbea4c24d742083422639e7bb1d5bf600e12cb176a13/pydantic_core-2.33.2-cp312-cp312-musllinux_1_1_x86_64.whl", hash = "sha256:5f483cfb75ff703095c59e365360cb73e00185e01aaea067cd19acffd2ab20ea", size = 2234187, upload-time = "2025-04-23T18:31:45.891Z" },
    { url = "https://files.pythonhosted.org/packages/b0/6a/adf5734ffd52bf86d865093ad70b2ce543415e0e356f6cacabbc0d9ad910/pydantic_core-2.33.2-cp312-cp312-win32.whl", hash = "sha256:9cb1da0f5a471435a7bc7e439b8a728e8b61e59784b2af70d7c169f8dd8ae290", size = 1892628, upload-time = "2025-04-23T18:31:47.819Z" },
    { url = "https://files.pythonhosted.org/packages/43/e4/5479fecb3606c1368d496a825d8411e126133c41224c1e7238be58b87d7e/pydantic_core-2.33.2-cp312-cp312-win_amd64.whl", hash = "sha256:f941635f2a3d96b2973e867144fde513665c87f13fe0e193c158ac51bfaaa7b2", size = 1955866, upload-time = "2025-04-23T18:31:49.635Z" },
    { url = "https://files.pythonhosted.org/packages/0d/24/8b11e8b3e2be9dd82df4b11408a67c61bb4dc4f8e11b5b0fc888b38118b5/pydantic_core-2.33.2-cp312-cp312-win_arm64.whl", hash = "sha256:cca3868ddfaccfbc4bfb1d608e2ccaaebe0ae628e1416aeb9c4d88c001bb45ab", size = 1888894, upload-time = "2025-04-23T18:31:51.609Z" },
    { url = "https://files.pythonhosted.org/packages/46/8c/99040727b41f56616573a28771b1bfa08a3d3fe74d3d513f01251f79f172/pydantic_core-2.33.2-cp313-cp313-macosx_10_12_x86_64.whl", hash = "sha256:1082dd3e2d7109ad8b7da48e1d4710c8d06c253cbc4a27c1cff4fbcaa97a9e3f", size = 2015688, upload-time = "2025-04-23T18:31:53.175Z" },
    { url = "https://files.pythonhosted.org/packages/3a/cc/5999d1eb705a6cefc31f0b4a90e9f7fc400539b1a1030529700cc1b51838/pydantic_core-2.33.2-cp313-cp313-macosx_11_0_arm64.whl", hash = "sha256:f517ca031dfc037a9c07e748cefd8d96235088b83b4f4ba8939105d20fa1dcd6", size = 1844808, upload-time = "2025-04-23T18:31:54.79Z" },
    { url = "https://files.pythonhosted.org/packages/6f/5e/a0a7b8885c98889a18b6e376f344da1ef323d270b44edf8174d6bce4d622/pydantic_core-2.33.2-cp313-cp313-manylinux_2_17_aarch64.manylinux2014_aarch64.whl", hash = "sha256:0a9f2c9dd19656823cb8250b0724ee9c60a82f3cdf68a080979d13092a3b0fef", size = 1885580, upload-time = "2025-04-23T18:31:57.393Z" },
    { url = "https://files.pythonhosted.org/packages/3b/2a/953581f343c7d11a304581156618c3f592435523dd9d79865903272c256a/pydantic_core-2.33.2-cp313-cp313-manylinux_2_17_armv7l.manylinux2014_armv7l.whl", hash = "sha256:2b0a451c263b01acebe51895bfb0e1cc842a5c666efe06cdf13846c7418caa9a", size = 1973859, upload-time = "2025-04-23T18:31:59.065Z" },
    { url = "https://files.pythonhosted.org/packages/e6/55/f1a813904771c03a3f97f676c62cca0c0a4138654107c1b61f19c644868b/pydantic_core-2.33.2-cp313-cp313-manylinux_2_17_ppc64le.manylinux2014_ppc64le.whl", hash = "sha256:1ea40a64d23faa25e62a70ad163571c0b342b8bf66d5fa612ac0dec4f069d916", size = 2120810, upload-time = "2025-04-23T18:32:00.78Z" },
    { url = "https://files.pythonhosted.org/packages/aa/c3/053389835a996e18853ba107a63caae0b9deb4a276c6b472931ea9ae6e48/pydantic_core-2.33.2-cp313-cp313-manylinux_2_17_s390x.manylinux2014_s390x.whl", hash = "sha256:0fb2d542b4d66f9470e8065c5469ec676978d625a8b7a363f07d9a501a9cb36a", size = 2676498, upload-time = "2025-04-23T18:32:02.418Z" },
    { url = "https://files.pythonhosted.org/packages/eb/3c/f4abd740877a35abade05e437245b192f9d0ffb48bbbbd708df33d3cda37/pydantic_core-2.33.2-cp313-cp313-manylinux_2_17_x86_64.manylinux2014_x86_64.whl", hash = "sha256:9fdac5d6ffa1b5a83bca06ffe7583f5576555e6c8b3a91fbd25ea7780f825f7d", size = 2000611, upload-time = "2025-04-23T18:32:04.152Z" },
    { url = "https://files.pythonhosted.org/packages/59/a7/63ef2fed1837d1121a894d0ce88439fe3e3b3e48c7543b2a4479eb99c2bd/pydantic_core-2.33.2-cp313-cp313-manylinux_2_5_i686.manylinux1_i686.whl", hash = "sha256:04a1a413977ab517154eebb2d326da71638271477d6ad87a769102f7c2488c56", size = 2107924, upload-time = "2025-04-23T18:32:06.129Z" },
    { url = "https://files.pythonhosted.org/packages/04/8f/2551964ef045669801675f1cfc3b0d74147f4901c3ffa42be2ddb1f0efc4/pydantic_core-2.33.2-cp313-cp313-musllinux_1_1_aarch64.whl", hash = "sha256:c8e7af2f4e0194c22b5b37205bfb293d166a7344a5b0d0eaccebc376546d77d5", size = 2063196, upload-time = "2025-04-23T18:32:08.178Z" },
    { url = "https://files.pythonhosted.org/packages/26/bd/d9602777e77fc6dbb0c7db9ad356e9a985825547dce5ad1d30ee04903918/pydantic_core-2.33.2-cp313-cp313-musllinux_1_1_armv7l.whl", hash = "sha256:5c92edd15cd58b3c2d34873597a1e20f13094f59cf88068adb18947df5455b4e", size = 2236389, upload-time = "2025-04-23T18:32:10.242Z" },
    { url = "https://files.pythonhosted.org/packages/42/db/0e950daa7e2230423ab342ae918a794964b053bec24ba8af013fc7c94846/pydantic_core-2.33.2-cp313-cp313-musllinux_1_1_x86_64.whl", hash = "sha256:65132b7b4a1c0beded5e057324b7e16e10910c106d43675d9bd87d4f38dde162", size = 2239223, upload-time = "2025-04-23T18:32:12.382Z" },
    { url = "https://files.pythonhosted.org/packages/58/4d/4f937099c545a8a17eb52cb67fe0447fd9a373b348ccfa9a87f141eeb00f/pydantic_core-2.33.2-cp313-cp313-win32.whl", hash = "sha256:52fb90784e0a242bb96ec53f42196a17278855b0f31ac7c3cc6f5c1ec4811849", size = 1900473, upload-time = "2025-04-23T18:32:14.034Z" },
    { url = "https://files.pythonhosted.org/packages/a0/75/4a0a9bac998d78d889def5e4ef2b065acba8cae8c93696906c3a91f310ca/pydantic_core-2.33.2-cp313-cp313-win_amd64.whl", hash = "sha256:c083a3bdd5a93dfe480f1125926afcdbf2917ae714bdb80b36d34318b2bec5d9", size = 1955269, upload-time = "2025-04-23T18:32:15.783Z" },
    { url = "https://files.pythonhosted.org/packages/f9/86/1beda0576969592f1497b4ce8e7bc8cbdf614c352426271b1b10d5f0aa64/pydantic_core-2.33.2-cp313-cp313-win_arm64.whl", hash = "sha256:e80b087132752f6b3d714f041ccf74403799d3b23a72722ea2e6ba2e892555b9", size = 1893921, upload-time = "2025-04-23T18:32:18.473Z" },
    { url = "https://files.pythonhosted.org/packages/a4/7d/e09391c2eebeab681df2b74bfe6c43422fffede8dc74187b2b0bf6fd7571/pydantic_core-2.33.2-cp313-cp313t-macosx_11_0_arm64.whl", hash = "sha256:61c18fba8e5e9db3ab908620af374db0ac1baa69f0f32df4f61ae23f15e586ac", size = 1806162, upload-time = "2025-04-23T18:32:20.188Z" },
    { url = "https://files.pythonhosted.org/packages/f1/3d/847b6b1fed9f8ed3bb95a9ad04fbd0b212e832d4f0f50ff4d9ee5a9f15cf/pydantic_core-2.33.2-cp313-cp313t-manylinux_2_17_x86_64.manylinux2014_x86_64.whl", hash = "sha256:95237e53bb015f67b63c91af7518a62a8660376a6a0db19b89acc77a4d6199f5", size = 1981560, upload-time = "2025-04-23T18:32:22.354Z" },
    { url = "https://files.pythonhosted.org/packages/6f/9a/e73262f6c6656262b5fdd723ad90f518f579b7bc8622e43a942eec53c938/pydantic_core-2.33.2-cp313-cp313t-win_amd64.whl", hash = "sha256:c2fc0a768ef76c15ab9238afa6da7f69895bb5d1ee83aeea2e3509af4472d0b9", size = 1935777, upload-time = "2025-04-23T18:32:25.088Z" },
]

[[package]]
name = "pygments"
version = "2.19.2"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/b0/77/a5b8c569bf593b0140bde72ea885a803b82086995367bf2037de0159d924/pygments-2.19.2.tar.gz", hash = "sha256:636cb2477cec7f8952536970bc533bc43743542f70392ae026374600add5b887", size = 4968631, upload-time = "2025-06-21T13:39:12.283Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/c7/21/705964c7812476f378728bdf590ca4b771ec72385c533964653c68e86bdc/pygments-2.19.2-py3-none-any.whl", hash = "sha256:86540386c03d588bb81d44bc3928634ff26449851e99741617ecb9037ee5ec0b", size = 1225217, upload-time = "2025-06-21T13:39:07.939Z" },
]

[[package]]
name = "pyproject-hooks"
version = "1.2.0"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/e7/82/28175b2414effca1cdac8dc99f76d660e7a4fb0ceefa4b4ab8f5f6742925/pyproject_hooks-1.2.0.tar.gz", hash = "sha256:1e859bd5c40fae9448642dd871adf459e5e2084186e8d2c2a79a824c970da1f8", size = 19228, upload-time = "2024-09-29T09:24:13.293Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/bd/24/12818598c362d7f300f18e74db45963dbcb85150324092410c8b49405e42/pyproject_hooks-1.2.0-py3-none-any.whl", hash = "sha256:9e5c6bfa8dcc30091c74b0cf803c81fdd29d94f01992a7707bc97babb1141913", size = 10216, upload-time = "2024-09-29T09:24:11.978Z" },
]

[[package]]
name = "python-dotenv"
version = "1.1.1"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/f6/b0/4bc07ccd3572a2f9df7e6782f52b0c6c90dcbb803ac4a167702d7d0dfe1e/python_dotenv-1.1.1.tar.gz", hash = "sha256:a8a6399716257f45be6a007360200409fce5cda2661e3dec71d23dc15f6189ab", size = 41978, upload-time = "2025-06-24T04:21:07.341Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/5f/ed/539768cf28c661b5b068d66d96a2f155c4971a5d55684a514c1a0e0dec2f/python_dotenv-1.1.1-py3-none-any.whl", hash = "sha256:31f23644fe2602f88ff55e1f5c79ba497e01224ee7737937930c448e4d0e24dc", size = 20556, upload-time = "2025-06-24T04:21:06.073Z" },
]

[[package]]
name = "requests"
version = "2.32.5"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "certifi" },
    { name = "charset-normalizer" },
    { name = "idna" },
    { name = "urllib3" },
]
sdist = { url = "https://files.pythonhosted.org/packages/c9/74/b3ff8e6c8446842c3f5c837e9c3dfcfe2018ea6ecef224c710c85ef728f4/requests-2.32.5.tar.gz", hash = "sha256:dbba0bac56e100853db0ea71b82b4dfd5fe2bf6d3754a8893c3af500cec7d7cf", size = 134517, upload-time = "2025-08-18T20:46:02.573Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/1e/db/4254e3eabe8020b458f1a747140d32277ec7a271daf1d235b70dc0b4e6e3/requests-2.32.5-py3-none-any.whl", hash = "sha256:2462f94637a34fd532264295e186976db0f5d453d1cdd31473c85a6a161affb6", size = 64738, upload-time = "2025-08-18T20:46:00.542Z" },
]

[[package]]
name = "rich"
version = "14.1.0"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "markdown-it-py" },
    { name = "pygments" },
]
sdist = { url = "https://files.pythonhosted.org/packages/fe/75/af448d8e52bf1d8fa6a9d089ca6c07ff4453d86c65c145d0a300bb073b9b/rich-14.1.0.tar.gz", hash = "sha256:e497a48b844b0320d45007cdebfeaeed8db2a4f4bcf49f15e455cfc4af11eaa8", size = 224441, upload-time = "2025-07-25T07:32:58.125Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/e3/30/3c4d035596d3cf444529e0b2953ad0466f6049528a879d27534700580395/rich-14.1.0-py3-none-any.whl", hash = "sha256:536f5f1785986d6dbdea3c75205c473f970777b4a0d6c6dd1b696aa05a3fa04f", size = 243368, upload-time = "2025-07-25T07:32:56.73Z" },
]

[[package]]
name = "rsa"
version = "4.9.1"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "pyasn1" },
]
sdist = { url = "https://files.pythonhosted.org/packages/da/8a/22b7beea3ee0d44b1916c0c1cb0ee3af23b700b6da9f04991899d0c555d4/rsa-4.9.1.tar.gz", hash = "sha256:e7bdbfdb5497da4c07dfd35530e1a902659db6ff241e39d9953cad06ebd0ae75", size = 29034, upload-time = "2025-04-16T09:51:18.218Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/64/8d/0133e4eb4beed9e425d9a98ed6e081a55d195481b7632472be1af08d2f6b/rsa-4.9.1-py3-none-any.whl", hash = "sha256:68635866661c6836b8d39430f97a996acbd61bfa49406748ea243539fe239762", size = 34696, upload-time = "2025-04-16T09:51:17.142Z" },
]

[[package]]
name = "ruff"
version = "0.15.5"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/77/9b/840e0039e65fcf12758adf684d2289024d6140cde9268cc59887dc55189c/ruff-0.15.5.tar.gz", hash = "sha256:7c3601d3b6d76dce18c5c824fc8d06f4eef33d6df0c21ec7799510cde0f159a2", size = 4574214, upload-time = "2026-03-05T20:06:34.946Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/47/20/5369c3ce21588c708bcbe517a8fbe1a8dfdb5dfd5137e14790b1da71612c/ruff-0.15.5-py3-none-linux_armv6l.whl", hash = "sha256:4ae44c42281f42e3b06b988e442d344a5b9b72450ff3c892e30d11b29a96a57c", size = 10478185, upload-time = "2026-03-05T20:06:29.093Z" },
    { url = "https://files.pythonhosted.org/packages/44/ed/e81dd668547da281e5dce710cf0bc60193f8d3d43833e8241d006720e42b/ruff-0.15.5-py3-none-macosx_10_12_x86_64.whl", hash = "sha256:6edd3792d408ebcf61adabc01822da687579a1a023f297618ac27a5b51ef0080", size = 10859201, upload-time = "2026-03-05T20:06:32.632Z" },
    { url = "https://files.pythonhosted.org/packages/c4/8f/533075f00aaf19b07c5cd6aa6e5d89424b06b3b3f4583bfa9c640a079059/ruff-0.15.5-py3-none-macosx_11_0_arm64.whl", hash = "sha256:89f463f7c8205a9f8dea9d658d59eff49db05f88f89cc3047fb1a02d9f344010", size = 10184752, upload-time = "2026-03-05T20:06:40.312Z" },
    { url = "https://files.pythonhosted.org/packages/66/0e/ba49e2c3fa0395b3152bad634c7432f7edfc509c133b8f4529053ff024fb/ruff-0.15.5-py3-none-manylinux_2_17_aarch64.manylinux2014_aarch64.whl", hash = "sha256:ba786a8295c6574c1116704cf0b9e6563de3432ac888d8f83685654fe528fd65", size = 10534857, upload-time = "2026-03-05T20:06:19.581Z" },
    { url = "https://files.pythonhosted.org/packages/59/71/39234440f27a226475a0659561adb0d784b4d247dfe7f43ffc12dd02e288/ruff-0.15.5-py3-none-manylinux_2_17_armv7l.manylinux2014_armv7l.whl", hash = "sha256:fd4b801e57955fe9f02b31d20375ab3a5c4415f2e5105b79fb94cf2642c91440", size = 10309120, upload-time = "2026-03-05T20:06:00.435Z" },
    { url = "https://files.pythonhosted.org/packages/f5/87/4140aa86a93df032156982b726f4952aaec4a883bb98cb6ef73c347da253/ruff-0.15.5-py3-none-manylinux_2_17_i686.manylinux2014_i686.whl", hash = "sha256:391f7c73388f3d8c11b794dbbc2959a5b5afe66642c142a6effa90b45f6f5204", size = 11047428, upload-time = "2026-03-05T20:05:51.867Z" },
    { url = "https://files.pythonhosted.org/packages/5a/f7/4953e7e3287676f78fbe85e3a0ca414c5ca81237b7575bdadc00229ac240/ruff-0.15.5-py3-none-manylinux_2_17_ppc64le.manylinux2014_ppc64le.whl", hash = "sha256:8dc18f30302e379fe1e998548b0f5e9f4dff907f52f73ad6da419ea9c19d66c8", size = 11914251, upload-time = "2026-03-05T20:06:22.887Z" },
    { url = "https://files.pythonhosted.org/packages/77/46/0f7c865c10cf896ccf5a939c3e84e1cfaeed608ff5249584799a74d33835/ruff-0.15.5-py3-none-manylinux_2_17_s390x.manylinux2014_s390x.whl", hash = "sha256:1cc6e7f90087e2d27f98dc34ed1b3ab7c8f0d273cc5431415454e22c0bd2a681", size = 11333801, upload-time = "2026-03-05T20:05:57.168Z" },
    { url = "https://files.pythonhosted.org/packages/d3/01/a10fe54b653061585e655f5286c2662ebddb68831ed3eaebfb0eb08c0a16/ruff-0.15.5-py3-none-manylinux_2_17_x86_64.manylinux2014_x86_64.whl", hash = "sha256:c1cb7169f53c1ddb06e71a9aebd7e98fc0fea936b39afb36d8e86d36ecc2636a", size = 11206821, upload-time = "2026-03-05T20:06:03.441Z" },
    { url = "https://files.pythonhosted.org/packages/7a/0d/2132ceaf20c5e8699aa83da2706ecb5c5dcdf78b453f77edca7fb70f8a93/ruff-0.15.5-py3-none-manylinux_2_31_riscv64.whl", hash = "sha256:9b037924500a31ee17389b5c8c4d88874cc6ea8e42f12e9c61a3d754ff72f1ca", size = 11133326, upload-time = "2026-03-05T20:06:25.655Z" },
    { url = "https://files.pythonhosted.org/packages/72/cb/2e5259a7eb2a0f87c08c0fe5bf5825a1e4b90883a52685524596bfc93072/ruff-0.15.5-py3-none-musllinux_1_2_aarch64.whl", hash = "sha256:65bb414e5b4eadd95a8c1e4804f6772bbe8995889f203a01f77ddf2d790929dd", size = 10510820, upload-time = "2026-03-05T20:06:37.79Z" },
    { url = "https://files.pythonhosted.org/packages/ff/20/b67ce78f9e6c59ffbdb5b4503d0090e749b5f2d31b599b554698a80d861c/ruff-0.15.5-py3-none-musllinux_1_2_armv7l.whl", hash = "sha256:d20aa469ae3b57033519c559e9bc9cd9e782842e39be05b50e852c7c981fa01d", size = 10302395, upload-time = "2026-03-05T20:05:54.504Z" },
    { url = "https://files.pythonhosted.org/packages/5f/e5/719f1acccd31b720d477751558ed74e9c88134adcc377e5e886af89d3072/ruff-0.15.5-py3-none-musllinux_1_2_i686.whl", hash = "sha256:15388dd28c9161cdb8eda68993533acc870aa4e646a0a277aa166de9ad5a8752", size = 10754069, upload-time = "2026-03-05T20:06:06.422Z" },
    { url = "https://files.pythonhosted.org/packages/c3/9c/d1db14469e32d98f3ca27079dbd30b7b44dbb5317d06ab36718dee3baf03/ruff-0.15.5-py3-none-musllinux_1_2_x86_64.whl", hash = "sha256:b30da330cbd03bed0c21420b6b953158f60c74c54c5f4c1dabbdf3a57bf355d2", size = 11304315, upload-time = "2026-03-05T20:06:10.867Z" },
    { url = "https://files.pythonhosted.org/packages/28/3a/950367aee7c69027f4f422059227b290ed780366b6aecee5de5039d50fa8/ruff-0.15.5-py3-none-win32.whl", hash = "sha256:732e5ee1f98ba5b3679029989a06ca39a950cced52143a0ea82a2102cb592b74", size = 10551676, upload-time = "2026-03-05T20:06:13.705Z" },
    { url = "https://files.pythonhosted.org/packages/b8/00/bf077a505b4e649bdd3c47ff8ec967735ce2544c8e4a43aba42ee9bf935d/ruff-0.15.5-py3-none-win_amd64.whl", hash = "sha256:821d41c5fa9e19117616c35eaa3f4b75046ec76c65e7ae20a333e9a8696bc7fe", size = 11678972, upload-time = "2026-03-05T20:06:45.379Z" },
    { url = "https://files.pythonhosted.org/packages/fe/4e/cd76eca6db6115604b7626668e891c9dd03330384082e33662fb0f113614/ruff-0.15.5-py3-none-win_arm64.whl", hash = "sha256:b498d1c60d2fe5c10c45ec3f698901065772730b411f164ae270bb6bfcc4740b", size = 10965572, upload-time = "2026-03-05T20:06:16.984Z" },
]

[[package]]
name = "sniffio"
version = "1.3.1"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/a2/87/a6771e1546d97e7e041b6ae58d80074f81b7d5121207425c964ddf5cfdbd/sniffio-1.3.1.tar.gz", hash = "sha256:f4324edc670a0f49750a81b895f35c3adb843cca46f0530f79fc1babb23789dc", size = 20372, upload-time = "2024-02-25T23:20:04.057Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/e9/44/75a9c9421471a6c4805dbf2356f7c181a29c1879239abab1ea2cc8f38b40/sniffio-1.3.1-py3-none-any.whl", hash = "sha256:2f6da418d1f1e0fddd844478f41680e794e6051915791a034ff65e5f100525a2", size = 10235, upload-time = "2024-02-25T23:20:01.196Z" },
]

[[package]]
name = "tenacity"
version = "9.1.2"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/0a/d4/2b0cd0fe285e14b36db076e78c93766ff1d529d70408bd1d2a5a84f1d929/tenacity-9.1.2.tar.gz", hash = "sha256:1169d376c297e7de388d18b4481760d478b0e99a777cad3a9c86e556f4b697cb", size = 48036, upload-time = "2025-04-02T08:25:09.966Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/e5/30/643397144bfbfec6f6ef821f36f33e57d35946c44a2352d3c9f0ae847619/tenacity-9.1.2-py3-none-any.whl", hash = "sha256:f77bf36710d8b73a50b2dd155c97b870017ad21afe6ab300326b0371b3b05138", size = 28248, upload-time = "2025-04-02T08:25:07.678Z" },
]

[[package]]
name = "typing-extensions"
version = "4.15.0"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/72/94/1a15dd82efb362ac84269196e94cf00f187f7ed21c242792a923cdb1c61f/typing_extensions-4.15.0.tar.gz", hash = "sha256:0cea48d173cc12fa28ecabc3b837ea3cf6f38c6d1136f85cbaaf598984861466", size = 109391, upload-time = "2025-08-25T13:49:26.313Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/18/67/36e9267722cc04a6b9f15c7f3441c2363321a3ea07da7ae0c0707beb2a9c/typing_extensions-4.15.0-py3-none-any.whl", hash = "sha256:f0fa19c6845758ab08074a0cfa8b7aecb71c999ca73d62883bc25cc018c4e548", size = 44614, upload-time = "2025-08-25T13:49:24.86Z" },
]

[[package]]
name = "typing-inspection"
version = "0.4.1"
source = { registry = "https://pypi.org/simple" }
dependencies = [
    { name = "typing-extensions" },
]
sdist = { url = "https://files.pythonhosted.org/packages/f8/b1/0c11f5058406b3af7609f121aaa6b609744687f1d158b3c3a5bf4cc94238/typing_inspection-0.4.1.tar.gz", hash = "sha256:6ae134cc0203c33377d43188d4064e9b357dba58cff3185f22924610e70a9d28", size = 75726, upload-time = "2025-05-21T18:55:23.885Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/17/69/cd203477f944c353c31bade965f880aa1061fd6bf05ded0726ca845b6ff7/typing_inspection-0.4.1-py3-none-any.whl", hash = "sha256:389055682238f53b04f7badcb49b989835495a96700ced5dab2d8feae4b26f51", size = 14552, upload-time = "2025-05-21T18:55:22.152Z" },
]

[[package]]
name = "urllib3"
version = "2.5.0"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/15/22/9ee70a2574a4f4599c47dd506532914ce044817c7752a79b6a51286319bc/urllib3-2.5.0.tar.gz", hash = "sha256:3fc47733c7e419d4bc3f6b3dc2b4f890bb743906a30d56ba4a5bfa4bbff92760", size = 393185, upload-time = "2025-06-18T14:07:41.644Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/a7/c2/fe1e52489ae3122415c51f387e221dd0773709bad6c6cdaa599e8a2c5185/urllib3-2.5.0-py3-none-any.whl", hash = "sha256:e6b01673c0fa6a13e374b50871808eb3bf7046c4b125b216f6bf1cc604cff0dc", size = 129795, upload-time = "2025-06-18T14:07:40.39Z" },
]

[[package]]
name = "wcwidth"
version = "0.2.13"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/6c/63/53559446a878410fc5a5974feb13d31d78d752eb18aeba59c7fef1af7598/wcwidth-0.2.13.tar.gz", hash = "sha256:72ea0c06399eb286d978fdedb6923a9eb47e1c486ce63e9b4e64fc18303972b5", size = 101301, upload-time = "2024-01-06T02:10:57.829Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/fd/84/fd2ba7aafacbad3c4201d395674fc6348826569da3c0937e75505ead3528/wcwidth-0.2.13-py2.py3-none-any.whl", hash = "sha256:3da69048e4540d84af32131829ff948f1e022c1c6bdb8d6102117aac784f6859", size = 34166, upload-time = "2024-01-06T02:10:55.763Z" },
]

[[package]]
name = "websockets"
version = "15.0.1"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/packages/21/e6/26d09fab466b7ca9c7737474c52be4f76a40301b08362eb2dbc19dcc16c1/websockets-15.0.1.tar.gz", hash = "sha256:82544de02076bafba038ce055ee6412d68da13ab47f0c60cab827346de828dee", size = 177016, upload-time = "2025-03-05T20:03:41.606Z" }
wheels = [
    { url = "https://files.pythonhosted.org/packages/51/6b/4545a0d843594f5d0771e86463606a3988b5a09ca5123136f8a76580dd63/websockets-15.0.1-cp312-cp312-macosx_10_13_universal2.whl", hash = "sha256:3e90baa811a5d73f3ca0bcbf32064d663ed81318ab225ee4f427ad4e26e5aff3", size = 175437, upload-time = "2025-03-05T20:02:16.706Z" },
    { url = "https://files.pythonhosted.org/packages/f4/71/809a0f5f6a06522af902e0f2ea2757f71ead94610010cf570ab5c98e99ed/websockets-15.0.1-cp312-cp312-macosx_10_13_x86_64.whl", hash = "sha256:592f1a9fe869c778694f0aa806ba0374e97648ab57936f092fd9d87f8bc03665", size = 173096, upload-time = "2025-03-05T20:02:18.832Z" },
    { url = "https://files.pythonhosted.org/packages/3d/69/1a681dd6f02180916f116894181eab8b2e25b31e484c5d0eae637ec01f7c/websockets-15.0.1-cp312-cp312-macosx_11_0_arm64.whl", hash = "sha256:0701bc3cfcb9164d04a14b149fd74be7347a530ad3bbf15ab2c678a2cd3dd9a2", size = 173332, upload-time = "2025-03-05T20:02:20.187Z" },
    { url = "https://files.pythonhosted.org/packages/a6/02/0073b3952f5bce97eafbb35757f8d0d54812b6174ed8dd952aa08429bcc3/websockets-15.0.1-cp312-cp312-manylinux_2_17_aarch64.manylinux2014_aarch64.whl", hash = "sha256:e8b56bdcdb4505c8078cb6c7157d9811a85790f2f2b3632c7d1462ab5783d215", size = 183152, upload-time = "2025-03-05T20:02:22.286Z" },
    { url = "https://files.pythonhosted.org/packages/74/45/c205c8480eafd114b428284840da0b1be9ffd0e4f87338dc95dc6ff961a1/websockets-15.0.1-cp312-cp312-manylinux_2_5_i686.manylinux1_i686.manylinux_2_17_i686.manylinux2014_i686.whl", hash = "sha256:0af68c55afbd5f07986df82831c7bff04846928ea8d1fd7f30052638788bc9b5", size = 182096, upload-time = "2025-03-05T20:02:24.368Z" },
    { url = "https://files.pythonhosted.org/packages/14/8f/aa61f528fba38578ec553c145857a181384c72b98156f858ca5c8e82d9d3/websockets-15.0.1-cp312-cp312-manylinux_2_5_x86_64.manylinux1_x86_64.manylinux_2_17_x86_64.manylinux2014_x86_64.whl", hash = "sha256:64dee438fed052b52e4f98f76c5790513235efaa1ef7f3f2192c392cd7c91b65", size = 182523, upload-time = "2025-03-05T20:02:25.669Z" },
    { url = "https://files.pythonhosted.org/packages/ec/6d/0267396610add5bc0d0d3e77f546d4cd287200804fe02323797de77dbce9/websockets-15.0.1-cp312-cp312-musllinux_1_2_aarch64.whl", hash = "sha256:d5f6b181bb38171a8ad1d6aa58a67a6aa9d4b38d0f8c5f496b9e42561dfc62fe", size = 182790, upload-time = "2025-03-05T20:02:26.99Z" },
    { url = "https://files.pythonhosted.org/packages/02/05/c68c5adbf679cf610ae2f74a9b871ae84564462955d991178f95a1ddb7dd/websockets-15.0.1-cp312-cp312-musllinux_1_2_i686.whl", hash = "sha256:5d54b09eba2bada6011aea5375542a157637b91029687eb4fdb2dab11059c1b4", size = 182165, upload-time = "2025-03-05T20:02:30.291Z" },
    { url = "https://files.pythonhosted.org/packages/29/93/bb672df7b2f5faac89761cb5fa34f5cec45a4026c383a4b5761c6cea5c16/websockets-15.0.1-cp312-cp312-musllinux_1_2_x86_64.whl", hash = "sha256:3be571a8b5afed347da347bfcf27ba12b069d9d7f42cb8c7028b5e98bbb12597", size = 182160, upload-time = "2025-03-05T20:02:31.634Z" },
    { url = "https://files.pythonhosted.org/packages/ff/83/de1f7709376dc3ca9b7eeb4b9a07b4526b14876b6d372a4dc62312bebee0/websockets-15.0.1-cp312-cp312-win32.whl", hash = "sha256:c338ffa0520bdb12fbc527265235639fb76e7bc7faafbb93f6ba80d9c06578a9", size = 176395, upload-time = "2025-03-05T20:02:33.017Z" },
    { url = "https://files.pythonhosted.org/packages/7d/71/abf2ebc3bbfa40f391ce1428c7168fb20582d0ff57019b69ea20fa698043/websockets-15.0.1-cp312-cp312-win_amd64.whl", hash = "sha256:fcd5cf9e305d7b8338754470cf69cf81f420459dbae8a3b40cee57417f4614a7", size = 176841, upload-time = "2025-03-05T20:02:34.498Z" },
    { url = "https://files.pythonhosted.org/packages/cb/9f/51f0cf64471a9d2b4d0fc6c534f323b664e7095640c34562f5182e5a7195/websockets-15.0.1-cp313-cp313-macosx_10_13_universal2.whl", hash = "sha256:ee443ef070bb3b6ed74514f5efaa37a252af57c90eb33b956d35c8e9c10a1931", size = 175440, upload-time = "2025-03-05T20:02:36.695Z" },
    { url = "https://files.pythonhosted.org/packages/8a/05/aa116ec9943c718905997412c5989f7ed671bc0188ee2ba89520e8765d7b/websockets-15.0.1-cp313-cp313-macosx_10_13_x86_64.whl", hash = "sha256:5a939de6b7b4e18ca683218320fc67ea886038265fd1ed30173f5ce3f8e85675", size = 173098, upload-time = "2025-03-05T20:02:37.985Z" },
    { url = "https://files.pythonhosted.org/packages/ff/0b/33cef55ff24f2d92924923c99926dcce78e7bd922d649467f0eda8368923/websockets-15.0.1-cp313-cp313-macosx_11_0_arm64.whl", hash = "sha256:746ee8dba912cd6fc889a8147168991d50ed70447bf18bcda7039f7d2e3d9151", size = 173329, upload-time = "2025-03-05T20:02:39.298Z" },
    { url = "https://files.pythonhosted.org/packages/31/1d/063b25dcc01faa8fada1469bdf769de3768b7044eac9d41f734fd7b6ad6d/websockets-15.0.1-cp313-cp313-manylinux_2_17_aarch64.manylinux2014_aarch64.whl", hash = "sha256:595b6c3969023ecf9041b2936ac3827e4623bfa3ccf007575f04c5a6aa318c22", size = 183111, upload-time = "2025-03-05T20:02:40.595Z" },
    { url = "https://files.pythonhosted.org/packages/93/53/9a87ee494a51bf63e4ec9241c1ccc4f7c2f45fff85d5bde2ff74fcb68b9e/websockets-15.0.1-cp313-cp313-manylinux_2_5_i686.manylinux1_i686.manylinux_2_17_i686.manylinux2014_i686.whl", hash = "sha256:3c714d2fc58b5ca3e285461a4cc0c9a66bd0e24c5da9911e30158286c9b5be7f", size = 182054, upload-time = "2025-03-05T20:02:41.926Z" },
    { url = "https://files.pythonhosted.org/packages/ff/b2/83a6ddf56cdcbad4e3d841fcc55d6ba7d19aeb89c50f24dd7e859ec0805f/websockets-15.0.1-cp313-cp313-manylinux_2_5_x86_64.manylinux1_x86_64.manylinux_2_17_x86_64.manylinux2014_x86_64.whl", hash = "sha256:0f3c1e2ab208db911594ae5b4f79addeb3501604a165019dd221c0bdcabe4db8", size = 182496, upload-time = "2025-03-05T20:02:43.304Z" },
    { url = "https://files.pythonhosted.org/packages/98/41/e7038944ed0abf34c45aa4635ba28136f06052e08fc2168520bb8b25149f/websockets-15.0.1-cp313-cp313-musllinux_1_2_aarch64.whl", hash = "sha256:229cf1d3ca6c1804400b0a9790dc66528e08a6a1feec0d5040e8b9eb14422375", size = 182829, upload-time = "2025-03-05T20:02:48.812Z" },
    { url = "https://files.pythonhosted.org/packages/e0/17/de15b6158680c7623c6ef0db361da965ab25d813ae54fcfeae2e5b9ef910/websockets-15.0.1-cp313-cp313-musllinux_1_2_i686.whl", hash = "sha256:756c56e867a90fb00177d530dca4b097dd753cde348448a1012ed6c5131f8b7d", size = 182217, upload-time = "2025-03-05T20:02:50.14Z" },
    { url = "https://files.pythonhosted.org/packages/33/2b/1f168cb6041853eef0362fb9554c3824367c5560cbdaad89ac40f8c2edfc/websockets-15.0.1-cp313-cp313-musllinux_1_2_x86_64.whl", hash = "sha256:558d023b3df0bffe50a04e710bc87742de35060580a293c2a984299ed83bc4e4", size = 182195, upload-time = "2025-03-05T20:02:51.561Z" },
    { url = "https://files.pythonhosted.org/packages/86/eb/20b6cdf273913d0ad05a6a14aed4b9a85591c18a987a3d47f20fa13dcc47/websockets-15.0.1-cp313-cp313-win32.whl", hash = "sha256:ba9e56e8ceeeedb2e080147ba85ffcd5cd0711b89576b83784d8605a7df455fa", size = 176393, upload-time = "2025-03-05T20:02:53.814Z" },
    { url = "https://files.pythonhosted.org/packages/1b/6c/c65773d6cab416a64d191d6ee8a8b1c68a09970ea6909d16965d26bfed1e/websockets-15.0.1-cp313-cp313-win_amd64.whl", hash = "sha256:e09473f095a819042ecb2ab9465aee615bd9c2028e4ef7d933600a8401c79561", size = 176837, upload-time = "2025-03-05T20:02:55.237Z" },
    { url = "https://files.pythonhosted.org/packages/fa/a8/5b41e0da817d64113292ab1f8247140aac61cbf6cfd085d6a0fa77f4984f/websockets-15.0.1-py3-none-any.whl", hash = "sha256:f7a866fbc1e97b5c617ee4116daaa09b722101d4a3c170c787450ba409f9736f", size = 169743, upload-time = "2025-03-05T20:03:39.41Z" },
]

