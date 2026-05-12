//! Purpose: Classify requested shell commands before the proxy chooses a reducer.
//! Caller: proxy::run and command adapters.
//! Dependencies: std::path for preserving the caller working directory.
//! Main Functions: CommandAst::new, CommandAst::classify.
//! Side Effects: None; classification is in-memory only.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum CommandKind {
    Test,
    Git,
    Search,
    FileRead,
    FileList,
    Build,
    Lint,
    Logs,
    PackageManager,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CommandAst {
    pub original_command: String,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub shell_wrapped: bool,
    pub detected_kind: CommandKind,
    pub has_shell_syntax: bool,
}

impl CommandAst {
    pub fn new(program: String, args: Vec<String>, cwd: PathBuf) -> Self {
        let original_command = format!("{} {}", program, args.join(" "));
        Self::from_parts(original_command, program, args, cwd, false, false)
    }

    pub fn from_command_text(command: &str, cwd: PathBuf) -> Self {
        let mut fields = command.split_whitespace();
        let program = fields.next().unwrap_or(command).to_string();
        let args = fields.map(str::to_string).collect();
        Self::from_parts(command.to_string(), program, args, cwd, false, false)
    }

    pub fn from_parts(
        original_command: String,
        program: String,
        args: Vec<String>,
        cwd: PathBuf,
        shell_wrapped: bool,
        has_shell_syntax: bool,
    ) -> Self {
        let mut ast = Self {
            original_command,
            program,
            args,
            cwd,
            shell_wrapped,
            detected_kind: CommandKind::Unknown,
            has_shell_syntax,
        };
        ast.classify();
        ast
    }

    pub fn classify(&mut self) {
        let program_base = self.program_base_name();
        self.detected_kind = match program_base.as_str() {
            "cargo" => {
                if self
                    .args
                    .iter()
                    .any(|arg| arg == "test" || arg == "nextest")
                {
                    CommandKind::Test
                } else if self.args.iter().any(|arg| arg == "clippy") {
                    CommandKind::Lint
                } else if self.args.iter().any(|arg| arg == "build" || arg == "check") {
                    CommandKind::Build
                } else {
                    CommandKind::Unknown
                }
            }
            "pytest" | "jest" | "vitest" | "playwright" => CommandKind::Test,
            "deno" => {
                if self.args.iter().any(|arg| arg == "test") {
                    CommandKind::Test
                } else if self.args.iter().any(|arg| arg == "lint") {
                    CommandKind::Lint
                } else if self.args.iter().any(|arg| arg == "fmt") {
                    CommandKind::Build
                } else {
                    CommandKind::Unknown
                }
            }
            "python" => {
                if self.args.first().map(String::as_str) == Some("-m")
                    && self.args.get(1).map(String::as_str) == Some("pytest")
                {
                    CommandKind::Test
                } else {
                    CommandKind::Unknown
                }
            }
            "npx" | "pnpx" | "dlx" => {
                if self
                    .args
                    .iter()
                    .any(|arg| matches!(arg.as_str(), "jest" | "vitest" | "playwright"))
                {
                    CommandKind::Test
                } else {
                    CommandKind::PackageManager
                }
            }
            "go" | "mvn" | "gradle" | "dotnet" => {
                if self.args.iter().any(|arg| arg == "test") {
                    CommandKind::Test
                } else if self.args.iter().any(|arg| arg == "build" || arg == "vet") {
                    CommandKind::Build
                } else {
                    CommandKind::Unknown
                }
            }
            "npm" | "pnpm" | "yarn" | "bun" => {
                if self.args.iter().any(|arg| {
                    matches!(arg.as_str(), "test" | "jest" | "vitest") || arg.ends_with(":test")
                }) {
                    CommandKind::Test
                } else if self.args.iter().any(|arg| arg == "build") {
                    CommandKind::Build
                } else if self
                    .args
                    .iter()
                    .any(|arg| ["install", "i", "ci", "add", "update"].contains(&arg.as_str()))
                {
                    CommandKind::PackageManager
                } else {
                    CommandKind::Unknown
                }
            }
            "git" if self.args.first().map(String::as_str) == Some("grep") => CommandKind::Search,
            "git" | "gh" => CommandKind::Git,
            "rg" | "grep" => CommandKind::Search,
            "cat" | "head" | "tail" | "sed" | "jq" => CommandKind::FileRead,
            "ls" | "find" | "tree" => CommandKind::FileList,
            "tsc" | "prettier" => CommandKind::Build,
            "make" => CommandKind::Build,
            "eslint" | "ruff" | "mypy" | "biome" => CommandKind::Lint,
            "docker" | "kubectl" | "terraform" | "aws" => CommandKind::Logs,
            "curl" | "wget" => CommandKind::Logs,
            "journalctl" | "systemctl" => CommandKind::Logs,
            "pip" | "pip3" => CommandKind::Build,
            "rake" => {
                if self.args.iter().any(|arg| arg == "test") {
                    CommandKind::Test
                } else {
                    CommandKind::Build
                }
            }
            "rspec" | "phpunit" => CommandKind::Test,
            "rubocop" | "flake8" | "golangci-lint" => CommandKind::Lint,
            "black" | "isort" => CommandKind::Build,
            "bundle" | "composer" => {
                if self
                    .args
                    .iter()
                    .any(|arg| ["install", "update", "add", "require"].contains(&arg.as_str()))
                {
                    CommandKind::PackageManager
                } else {
                    CommandKind::Build
                }
            }
            "webpack" | "vite" | "next" => CommandKind::Build,
            "nx" => {
                if self.args.iter().any(|arg| arg == "test") {
                    CommandKind::Test
                } else if self.args.iter().any(|arg| arg == "lint") {
                    CommandKind::Lint
                } else if self.args.iter().any(|arg| arg == "build") {
                    CommandKind::Build
                } else {
                    CommandKind::Unknown
                }
            }
            "brew" | "apt" | "apt-get" => CommandKind::PackageManager,
            _ => CommandKind::Unknown,
        };
    }

    fn program_base_name(&self) -> String {
        let normalized = self.program.replace('\\', "/");
        let base_name = normalized.rsplit('/').next().unwrap_or(&self.program);
        base_name
            .trim_end_matches(".exe")
            .trim_end_matches(".cmd")
            .trim_end_matches(".bat")
            .to_ascii_lowercase()
    }
}
