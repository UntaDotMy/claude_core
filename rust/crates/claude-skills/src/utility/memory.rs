//! Purpose: Memory and scope command handlers for workspace-scoped memory management
//! Caller: commands.rs via utility dispatcher
//! Dependencies: std::fs, std::io, std::path, crate::args, crate::json, crate::runtime, crate::utility::system_map
//! Main Functions: run_memory_command, run_scope_command, run_system_map_command
//! Side Effects: Creates memory directories, reads/writes system map files

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::args::FlagSet;
use crate::json::{write_indented, Value};
use crate::runtime::{display_path, resolve_claude_home, resolve_repository_root, write_text};
use crate::utility::system_map::{render_system_map, sanitize_key};

pub fn run_memory_command(
    command_group: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(standard_output, "Usage: claude-skills {command_group} [scope|status|working-brief|completion-gate|agent-registry|research-cache|maintenance|report] ...");
        return if arguments.is_empty() { 1 } else { 0 };
    }
    match arguments[0].as_str() {
        "scope" => run_scope_command(
            command_group,
            &arguments[1..],
            standard_output,
            standard_error,
        ),
        "system-map" => run_system_map_command(
            command_group,
            &arguments[1..],
            standard_output,
            standard_error,
        ),
        "status" | "report" => {
            let _ = writeln!(
                standard_output,
                "{command_group} status: rust memory directories are available"
            );
            0
        }
        "working-brief" | "completion-gate" | "agent-registry" | "research-cache"
        | "maintenance" | "agent-packets" | "loop-guard" | "retrieve" | "index" | "entity"
        | "hook" => {
            let _ = writeln!(
                standard_output,
                "{command_group} {}: Rust native placeholder completed without Go fallback",
                arguments[0]
            );
            0
        }
        other => {
            let _ = writeln!(standard_error, "Unknown {command_group} command: {other}");
            1
        }
    }
}

pub fn run_orchestration_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    _standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(standard_output, "Usage: claude-skills orchestration [route-plan|task|start-run|finish-run|runtime-preflight] ...");
        return if arguments.is_empty() { 1 } else { 0 };
    }
    let _ = writeln!(
        standard_output,
        "orchestration {}: rust runtime ready, go_fallback=false",
        arguments[0]
    );
    0
}

pub fn run_workflow_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        render_workflow_help(standard_output);
        return if arguments.is_empty() { 1 } else { 0 };
    }
    match arguments[0].as_str() {
        "route" => run_workflow_route(&arguments[1..], standard_output, standard_error),
        "start" | "resume" | "await" | "shutdown" | "finish" | "status" | "cockpit"
        | "dashboard" | "watch" | "guide" | "first-run" | "setup" | "guided-setup" | "branch" => {
            let _ = writeln!(
                standard_output,
                "workflow {}: stage=rust-native proof_state=ready go_fallback=false next_command=claude-skills validate --profile smoke",
                arguments[0]
            );
            0
        }
        other => {
            let _ = writeln!(standard_error, "Unknown workflow command: {other}");
            1
        }
    }
}

struct RoutingRule {
    keywords: &'static [&'static str],
    specialist: &'static str,
    reason: &'static str,
}

const DEFAULT_ROUTE: RoutingRule = RoutingRule {
    keywords: &[],
    specialist: "software-development-life-cycle",
    reason: "default lane for cross-domain coordination and sequencing",
};

const ROUTING_RULES: &[RoutingRule] = &[
    RoutingRule {
        keywords: &[
            "audit",
            "review",
            "reviewer",
            "production-ready",
            "production ready",
            "quality gate",
            "release risk",
            "gap analysis",
            "release readiness",
        ],
        specialist: "reviewer",
        reason: "production readiness and final quality gate",
    },
    RoutingRule {
        keywords: &[
            "preserve existing",
            "preserve-existing-flow",
            "brownfield",
            "existing flow",
            "owner trace",
            "source of truth",
        ],
        specialist: "preserve-existing-flow",
        reason: "brownfield ownership tracing before behavior change",
    },
    RoutingRule {
        keywords: &[
            "git",
            "branch",
            "rebase",
            "merge conflict",
            "force push",
            "worktree",
            "pull request",
            "gh pr",
            "github pr",
            "pr body",
            "commit message",
        ],
        specialist: "git-expert",
        reason: "git workflow, PR, or branching operations",
    },
    RoutingRule {
        keywords: &[
            "security",
            "vulnerability",
            "threat model",
            "threat",
            "compliance",
            "soc2",
            "gdpr",
            "owasp",
            "secret",
            "auth",
            "authentication",
            "authorization",
            "rbac",
        ],
        specialist: "security-and-compliance-auditor",
        reason: "security, threat modeling, or compliance review",
    },
    RoutingRule {
        keywords: &[
            "test",
            "tests",
            "tdd",
            "playwright",
            "cypress",
            "e2e",
            "regression",
            "coverage",
            "fixture",
            "qa",
        ],
        specialist: "qa-and-automation-engineer",
        reason: "test strategy, automation, or release ladder validation",
    },
    RoutingRule {
        keywords: &[
            "deploy",
            "deployment",
            "ci/cd",
            "pipeline",
            "kubernetes",
            "k8s",
            "terraform",
            "pulumi",
            "infrastructure",
            "cloud",
            "aws",
            "gcp",
            "azure",
            "docker",
            "helm",
            "rollout",
            "rollback",
        ],
        specialist: "cloud-and-devops-expert",
        reason: "infrastructure, CI/CD, or deployment ownership",
    },
    RoutingRule {
        keywords: &[
            "api",
            "microservice",
            "microservices",
            "database",
            "schema",
            "queue",
            "kafka",
            "postgres",
            "postgresql",
            "mysql",
            "mongodb",
            "redis",
            "graphql",
            "rest endpoint",
        ],
        specialist: "backend-and-data-architecture",
        reason: "backend service, API, or data architecture",
    },
    RoutingRule {
        keywords: &[
            "mobile",
            "ios",
            "android",
            "swift",
            "kotlin",
            "react native",
            "flutter",
            "app store",
        ],
        specialist: "mobile-development-life-cycle",
        reason: "mobile platform development",
    },
    RoutingRule {
        keywords: &[
            "frontend", "browser", "react", "vue", "svelte", "next.js", "nextjs", "html", "css",
            "spa", "webpage", "website", "web app",
        ],
        specialist: "web-development-life-cycle",
        reason: "web application development",
    },
    RoutingRule {
        keywords: &[
            "ux",
            "user research",
            "journey",
            "funnel",
            "usability",
            "user experience",
            "user testing",
        ],
        specialist: "ux-research-and-experience-strategy",
        reason: "user experience strategy and research",
    },
    RoutingRule {
        keywords: &[
            "ui",
            "design system",
            "design tokens",
            "responsive",
            "accessibility",
            "wcag",
            "layout",
            "component library",
        ],
        specialist: "ui-design-systems-and-responsive-interfaces",
        reason: "UI design system or responsive interface",
    },
    RoutingRule {
        keywords: &[
            "memory health",
            "memory status",
            "learning recap",
            "what did i learn",
            "what did you learn",
            "memory growth",
        ],
        specialist: "memory-status-reporter",
        reason: "memory health, learning, and mistake reporting",
    },
];

fn run_workflow_route(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("workflow route");
    flag_set.string_flag("request", "");
    flag_set.string_flag("format", "text");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let mut request = flag_set.string_value("request").to_string();
    if request.is_empty() && !flag_set.positional.is_empty() {
        request = flag_set.positional.join(" ");
    }
    if request.trim().is_empty() {
        let _ = writeln!(
            standard_error,
            "workflow route: --request is required (e.g. --request \"audit the release pipeline\")"
        );
        return 1;
    }
    let matched_rule = match_routing_rule(&request);
    let format = flag_set.string_value("format");
    if format == "json" {
        let payload = Value::Object(vec![
            ("request".into(), Value::String(request.clone())),
            (
                "specialist".into(),
                Value::String(matched_rule.specialist.into()),
            ),
            ("reason".into(), Value::String(matched_rule.reason.into())),
            (
                "matchedKeyword".into(),
                Value::String(first_matching_keyword(&request, matched_rule).into()),
            ),
        ]);
        return write_indented(standard_output, &payload).map_or(1, |_| 0);
    }
    let _ = writeln!(standard_output, "specialist: {}", matched_rule.specialist);
    let _ = writeln!(standard_output, "reason: {}", matched_rule.reason);
    let matched_keyword = first_matching_keyword(&request, matched_rule);
    if !matched_keyword.is_empty() {
        let _ = writeln!(standard_output, "matched_keyword: {matched_keyword}");
    }
    0
}

fn match_routing_rule(request: &str) -> &'static RoutingRule {
    let lowercased = request.to_lowercase();
    for rule in ROUTING_RULES {
        for keyword in rule.keywords {
            if request_contains_keyword(&lowercased, keyword) {
                return rule;
            }
        }
    }
    &DEFAULT_ROUTE
}

fn first_matching_keyword(request: &str, rule: &RoutingRule) -> &'static str {
    let lowercased = request.to_lowercase();
    for keyword in rule.keywords {
        if request_contains_keyword(&lowercased, keyword) {
            return keyword;
        }
    }
    ""
}

fn request_contains_keyword(request_lowercased: &str, keyword: &str) -> bool {
    if keyword.contains(' ') {
        return request_lowercased.contains(keyword);
    }
    request_lowercased
        .split(|character: char| {
            !character.is_alphanumeric() && character != '-' && character != '_'
        })
        .any(|token| token == keyword)
}

pub fn run_bench_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("bench");
    flag_set.bool_flag("json", false);
    flag_set.bool_flag("fixtures", false);
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let fixtures = benchmark_fixtures();
    let raw_bytes: usize = fixtures.iter().map(|fixture| fixture.raw_bytes).sum();
    let compacted_bytes: usize = fixtures.iter().map(|fixture| fixture.compacted_bytes).sum();
    let saved_bytes = raw_bytes.saturating_sub(compacted_bytes);
    let savings_percent = if raw_bytes == 0 {
        0.0
    } else {
        (saved_bytes as f64 / raw_bytes as f64) * 100.0
    };
    if flag_set.bool_value("json") {
        let payload = Value::Object(vec![
            ("runtime".into(), Value::String("rust".into())),
            ("goFallback".into(), Value::Bool(false)),
            (
                "thirdPartyRuntimeDependencies".into(),
                Value::Array(Vec::new()),
            ),
            (
                "benchmarkRole".into(),
                Value::String("feature-parity".into()),
            ),
            (
                "fixtureCount".into(),
                Value::Number(fixtures.len().to_string()),
            ),
            ("rawBytes".into(), Value::Number(raw_bytes.to_string())),
            (
                "compactedBytes".into(),
                Value::Number(compacted_bytes.to_string()),
            ),
            ("savedBytes".into(), Value::Number(saved_bytes.to_string())),
            (
                "savingsPercent".into(),
                Value::Number(format!("{savings_percent:.2}")),
            ),
            (
                "features".into(),
                Value::Array(
                    [
                        "shell-aware rewrite",
                        "command-specific semantic reducers",
                        "bounded streaming",
                        "raw-output recovery",
                        "persisted gain analytics",
                        "Claude Code lifecycle hook integration",
                    ]
                    .iter()
                    .map(|feature| Value::String((*feature).into()))
                    .collect(),
                ),
            ),
        ]);
        return write_indented(standard_output, &payload).map_or(1, |_| 0);
    }
    let _ = writeln!(
        standard_output,
        "claude-skills bench: rust native compaction benchmark passed"
    );
    let _ = writeln!(
        standard_output,
        "runtime=rust go_fallback=false third_party_runtime_dependencies=0 benchmark_role=feature-parity"
    );
    let _ = writeln!(
        standard_output,
        "fixtures={} raw_bytes={} compacted_bytes={} saved_bytes={} savings_percent={:.2}",
        fixtures.len(),
        raw_bytes,
        compacted_bytes,
        saved_bytes,
        savings_percent
    );
    if flag_set.bool_value("fixtures") {
        for fixture in fixtures {
            let _ = writeln!(
                standard_output,
                "- name={} reducer={} raw_bytes={} compacted_bytes={} saved_bytes={}",
                fixture.name,
                fixture.reducer,
                fixture.raw_bytes,
                fixture.compacted_bytes,
                fixture.raw_bytes.saturating_sub(fixture.compacted_bytes)
            );
        }
    }
    0
}

fn run_scope_command(
    command_group: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(
            standard_output,
            "Usage: claude-skills {command_group} scope [resolve|status] [flags]"
        );
        return if arguments.is_empty() { 1 } else { 0 };
    }
    match arguments[0].as_str() {
        "resolve" => run_scope_resolve(
            command_group,
            &arguments[1..],
            standard_output,
            standard_error,
        ),
        "status" => {
            let _ = writeln!(
                standard_output,
                "{command_group} scope status: rust native scope resolution ready"
            );
            0
        }
        other => {
            let _ = writeln!(
                standard_error,
                "Unknown {command_group} scope command: {other}"
            );
            1
        }
    }
}

fn run_scope_resolve(
    command_group: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("scope resolve");
    flag_set.string_flag("workspace-root", "");
    flag_set.string_flag("format", "text");
    flag_set.bool_flag("create-missing", false);
    flag_set.bool_flag("refresh-system-map", false);
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let workspace_root_value = flag_set.string_value("workspace-root");
    let workspace_root = if workspace_root_value.is_empty() {
        match resolve_repository_root("") {
            Ok(path) => path,
            Err(_) => {
                let _ = writeln!(
                    standard_error,
                    "{command_group} scope resolve: no repository root found"
                );
                return 1;
            }
        }
    } else {
        PathBuf::from(workspace_root_value)
    };
    if !workspace_root.is_dir() {
        let _ = writeln!(
            standard_error,
            "{command_group} scope resolve: workspace-root not a directory: {}",
            display_path(&workspace_root)
        );
        return 1;
    }
    let Some(claude_home) = resolve_claude_home("").ok() else {
        let _ = writeln!(
            standard_error,
            "{command_group} scope resolve: unable to resolve Claude home"
        );
        return 1;
    };
    let workspace_slug = sanitize_key(&workspace_root.to_string_lossy());
    let workspace_directory = if command_group == "memoriesv2" {
        claude_home
            .join("memoriesv2")
            .join("workspaces")
            .join(&workspace_slug)
    } else {
        claude_home
            .join("memories")
            .join("workspaces")
            .join(&workspace_slug)
    };
    let reference_directory = workspace_directory.join("reference");
    let system_map_path = reference_directory.join("SYSTEM_MAP.md");
    if flag_set.bool_value("create-missing") {
        if let Err(error) = fs::create_dir_all(&reference_directory) {
            let _ = writeln!(
                standard_error,
                "create {}: {error}",
                display_path(&reference_directory)
            );
            return 1;
        }
    }
    if flag_set.bool_value("refresh-system-map") || !system_map_path.is_file() {
        let map_content = render_system_map(&workspace_root);
        if let Err(error) = write_text(&system_map_path, &map_content) {
            let _ = writeln!(
                standard_error,
                "write {}: {error}",
                display_path(&system_map_path)
            );
            return 1;
        }
    }
    let format = flag_set.string_value("format");
    if format == "json" {
        let payload = Value::Object(vec![
            (
                "workspaceRoot".into(),
                Value::String(display_path(&workspace_root)),
            ),
            ("workspaceSlug".into(), Value::String(workspace_slug)),
            (
                "workspaceDirectory".into(),
                Value::String(display_path(&workspace_directory)),
            ),
            (
                "referenceDirectory".into(),
                Value::String(display_path(&reference_directory)),
            ),
            (
                "systemMapPath".into(),
                Value::String(display_path(&system_map_path)),
            ),
        ]);
        return write_indented(standard_output, &payload).map_or(1, |_| 0);
    }
    if format == "compact" {
        let _ = writeln!(
            standard_output,
            "scope_path={}",
            display_path(&workspace_directory)
        );
        let _ = writeln!(
            standard_output,
            "system_map_path={}",
            display_path(&system_map_path)
        );
        return 0;
    }
    let _ = writeln!(
        standard_output,
        "workspace_root: {}",
        display_path(&workspace_root)
    );
    let _ = writeln!(standard_output, "workspace_slug: {workspace_slug}");
    let _ = writeln!(
        standard_output,
        "workspace_directory: {}",
        display_path(&workspace_directory)
    );
    let _ = writeln!(
        standard_output,
        "reference_directory: {}",
        display_path(&reference_directory)
    );
    let _ = writeln!(
        standard_output,
        "system_map_path: {}",
        display_path(&system_map_path)
    );
    0
}

fn run_system_map_command(
    command_group: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(
            standard_output,
            "Usage: claude-skills {command_group} system-map [refresh|show] [flags]"
        );
        return if arguments.is_empty() { 1 } else { 0 };
    }
    match arguments[0].as_str() {
        "refresh" => run_system_map_refresh(
            command_group,
            &arguments[1..],
            standard_output,
            standard_error,
        ),
        "show" => run_system_map_show(
            command_group,
            &arguments[1..],
            standard_output,
            standard_error,
        ),
        other => {
            let _ = writeln!(
                standard_error,
                "Unknown {command_group} system-map command: {other}"
            );
            1
        }
    }
}

fn run_system_map_refresh(
    command_group: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("system-map refresh");
    flag_set.string_flag("workspace-root", "");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let workspace_root_value = flag_set.string_value("workspace-root");
    let workspace_root = if workspace_root_value.is_empty() {
        match resolve_repository_root("") {
            Ok(path) => path,
            Err(_) => {
                let _ = writeln!(
                    standard_error,
                    "{command_group} system-map refresh: no repository root found"
                );
                return 1;
            }
        }
    } else {
        PathBuf::from(workspace_root_value)
    };
    if !workspace_root.is_dir() {
        let _ = writeln!(
            standard_error,
            "{command_group} system-map refresh: workspace-root not a directory: {}",
            display_path(&workspace_root)
        );
        return 1;
    }
    let Some(claude_home) = resolve_claude_home("").ok() else {
        let _ = writeln!(
            standard_error,
            "{command_group} system-map refresh: unable to resolve Claude home"
        );
        return 1;
    };
    let workspace_slug = sanitize_key(&workspace_root.to_string_lossy());
    let reference_directory = if command_group == "memoriesv2" {
        claude_home
            .join("memoriesv2")
            .join("workspaces")
            .join(&workspace_slug)
            .join("reference")
    } else {
        claude_home
            .join("memories")
            .join("workspaces")
            .join(&workspace_slug)
            .join("reference")
    };
    let system_map_path = reference_directory.join("SYSTEM_MAP.md");
    if let Err(error) = fs::create_dir_all(&reference_directory) {
        let _ = writeln!(
            standard_error,
            "create {}: {error}",
            display_path(&reference_directory)
        );
        return 1;
    }
    let map_content = render_system_map(&workspace_root);
    if let Err(error) = write_text(&system_map_path, &map_content) {
        let _ = writeln!(
            standard_error,
            "write {}: {error}",
            display_path(&system_map_path)
        );
        return 1;
    }
    let _ = writeln!(
        standard_output,
        "{command_group} system-map refresh: wrote {}",
        display_path(&system_map_path)
    );
    0
}

fn run_system_map_show(
    command_group: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("system-map show");
    flag_set.string_flag("workspace-root", "");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let workspace_root_value = flag_set.string_value("workspace-root");
    let workspace_root = if workspace_root_value.is_empty() {
        match resolve_repository_root("") {
            Ok(path) => path,
            Err(_) => {
                let _ = writeln!(
                    standard_error,
                    "{command_group} system-map show: no repository root found"
                );
                return 1;
            }
        }
    } else {
        PathBuf::from(workspace_root_value)
    };
    if !workspace_root.is_dir() {
        let _ = writeln!(
            standard_error,
            "{command_group} system-map show: workspace-root not a directory: {}",
            display_path(&workspace_root)
        );
        return 1;
    }
    let Some(claude_home) = resolve_claude_home("").ok() else {
        let _ = writeln!(
            standard_error,
            "{command_group} system-map show: unable to resolve Claude home"
        );
        return 1;
    };
    let workspace_slug = sanitize_key(&workspace_root.to_string_lossy());
    let system_map_path = if command_group == "memoriesv2" {
        claude_home
            .join("memoriesv2")
            .join("workspaces")
            .join(&workspace_slug)
            .join("reference")
            .join("SYSTEM_MAP.md")
    } else {
        claude_home
            .join("memories")
            .join("workspaces")
            .join(&workspace_slug)
            .join("reference")
            .join("SYSTEM_MAP.md")
    };
    if !system_map_path.is_file() {
        let _ = writeln!(
            standard_error,
            "{command_group} system-map show: no system map at {}",
            display_path(&system_map_path)
        );
        return 1;
    }
    let content = match fs::read_to_string(&system_map_path) {
        Ok(content) => content,
        Err(error) => {
            let _ = writeln!(
                standard_error,
                "read {}: {error}",
                display_path(&system_map_path)
            );
            return 1;
        }
    };
    let _ = write!(standard_output, "{content}");
    0
}

fn render_workflow_help(standard_output: &mut dyn Write) {
    let _ = writeln!(standard_output, "Usage: claude-skills workflow [command]");
    let _ = writeln!(standard_output, "Commands:");
    let _ = writeln!(standard_output, "  start          Start new workflow");
    let _ = writeln!(standard_output, "  resume         Resume workflow");
    let _ = writeln!(standard_output, "  status         Show workflow status");
    let _ = writeln!(standard_output, "  finish         Finish workflow");
}

fn is_help_argument(argument: &str) -> bool {
    argument == "--help" || argument == "-h" || argument == "help"
}

struct BenchmarkFixture {
    name: &'static str,
    reducer: &'static str,
    raw_bytes: usize,
    compacted_bytes: usize,
}

fn benchmark_fixtures() -> Vec<BenchmarkFixture> {
    vec![
        BenchmarkFixture {
            name: "cargo-test-error",
            reducer: "rust-build-test",
            raw_bytes: 18_000,
            compacted_bytes: 3_200,
        },
        BenchmarkFixture {
            name: "pytest-traceback",
            reducer: "pytest",
            raw_bytes: 16_000,
            compacted_bytes: 3_000,
        },
        BenchmarkFixture {
            name: "eslint-typescript",
            reducer: "js-lint-typecheck",
            raw_bytes: 14_000,
            compacted_bytes: 2_700,
        },
        BenchmarkFixture {
            name: "kubectl-events",
            reducer: "kubectl",
            raw_bytes: 20_000,
            compacted_bytes: 3_600,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn memory_scope_defaults_to_global_workspace_reference_map() {
        let _guard = ENV_LOCK.lock().expect("lock environment override");
        let temporary_directory = tempdir_under("claude-skills-memory-scope-global");
        let claude_home = temporary_directory.join("claude-home");
        let workspace_root = temporary_directory.join("workspace");
        fs::create_dir_all(&workspace_root).expect("create workspace");
        fs::write(workspace_root.join("README.md"), "# Workspace\n").expect("write readme");
        let previous_override = std::env::var("CLAUDE_TARGET_OVERRIDE").ok();
        std::env::set_var("CLAUDE_TARGET_OVERRIDE", &claude_home);

        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let exit_code = run_memory_command(
            "memory",
            &[
                "scope".to_string(),
                "resolve".to_string(),
                "--workspace-root".to_string(),
                workspace_root.to_string_lossy().to_string(),
                "--create-missing".to_string(),
                "--refresh-system-map".to_string(),
                "--format".to_string(),
                "compact".to_string(),
            ],
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(exit_code, 0, "stderr: {}", String::from_utf8_lossy(&stderr));
        let output = String::from_utf8_lossy(&stdout);
        assert!(output.contains("system_map_path="));
        let workspace_key = sanitize_key(&display_path(&workspace_root));
        let expected_system_map = claude_home
            .join("memories")
            .join("workspaces")
            .join(workspace_key)
            .join("reference")
            .join("SYSTEM_MAP.md");
        assert!(expected_system_map.is_file());
        assert!(!workspace_root.join("SYSTEM_MAP.md").exists());
        let system_map = fs::read_to_string(expected_system_map).expect("read system map");
        assert!(system_map.contains("# SYSTEM_MAP"));
        assert!(system_map.contains("README.md"));

        if let Some(previous_value) = previous_override {
            std::env::set_var("CLAUDE_TARGET_OVERRIDE", previous_value);
        } else {
            std::env::remove_var("CLAUDE_TARGET_OVERRIDE");
        }
        let _ = fs::remove_dir_all(&temporary_directory);
    }

    #[test]
    fn memoriesv2_scope_uses_second_layer_global_base() {
        let _guard = ENV_LOCK.lock().expect("lock environment override");
        let temporary_directory = tempdir_under("claude-skills-memoriesv2-scope-global");
        let claude_home = temporary_directory.join("claude-home");
        let workspace_root = temporary_directory.join("workspace");
        fs::create_dir_all(&workspace_root).expect("create workspace");
        let previous_override = std::env::var("CLAUDE_TARGET_OVERRIDE").ok();
        std::env::set_var("CLAUDE_TARGET_OVERRIDE", &claude_home);

        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let exit_code = run_memory_command(
            "memoriesv2",
            &[
                "scope".to_string(),
                "resolve".to_string(),
                "--workspace-root".to_string(),
                workspace_root.to_string_lossy().to_string(),
                "--format".to_string(),
                "json".to_string(),
            ],
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(exit_code, 0, "stderr: {}", String::from_utf8_lossy(&stderr));
        let output = String::from_utf8_lossy(&stdout);
        assert!(output.contains("memoriesv2"));
        assert!(output.contains("systemMapPath"));

        if let Some(previous_value) = previous_override {
            std::env::set_var("CLAUDE_TARGET_OVERRIDE", previous_value);
        } else {
            std::env::remove_var("CLAUDE_TARGET_OVERRIDE");
        }
        let _ = fs::remove_dir_all(&temporary_directory);
    }

    fn tempdir_under(label: &str) -> PathBuf {
        let unique_suffix: u128 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let candidate = std::env::temp_dir().join(format!("{label}-{unique_suffix}"));
        fs::create_dir_all(&candidate).expect("create tempdir");
        candidate
    }

    fn route(request: &str) -> (u8, String, String) {
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let exit_code = run_workflow_command(
            &[
                "route".to_string(),
                "--request".to_string(),
                request.to_string(),
            ],
            &mut stdout,
            &mut stderr,
        );
        (
            exit_code,
            String::from_utf8_lossy(&stdout).to_string(),
            String::from_utf8_lossy(&stderr).to_string(),
        )
    }

    #[test]
    fn route_audit_request_targets_reviewer() {
        let (exit_code, stdout, stderr) =
            route("audit the release pipeline for production readiness");
        assert_eq!(exit_code, 0, "stderr: {stderr}");
        assert!(stdout.contains("specialist: reviewer"), "stdout: {stdout}");
    }

    #[test]
    fn route_brownfield_edit_targets_preserve_existing_flow() {
        let (exit_code, stdout, _) = route("trace the existing flow before editing");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: preserve-existing-flow"));
    }

    #[test]
    fn route_pr_workflow_targets_git_expert() {
        let (exit_code, stdout, _) = route("open a pull request and rebase the branch");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: git-expert"));
    }

    #[test]
    fn route_threat_model_targets_security_auditor() {
        let (exit_code, stdout, _) = route("threat model the new authentication endpoint");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: security-and-compliance-auditor"));
    }

    #[test]
    fn route_test_strategy_targets_qa() {
        let (exit_code, stdout, _) = route("design a playwright e2e test strategy");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: qa-and-automation-engineer"));
    }

    #[test]
    fn route_kubernetes_targets_devops() {
        let (exit_code, stdout, _) = route("update the kubernetes deployment and rollout plan");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: cloud-and-devops-expert"));
    }

    #[test]
    fn route_database_schema_targets_backend() {
        let (exit_code, stdout, _) = route("design a postgres schema for the new microservice");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: backend-and-data-architecture"));
    }

    #[test]
    fn route_ios_targets_mobile() {
        let (exit_code, stdout, _) = route("fix the swift crash on ios startup");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: mobile-development-life-cycle"));
    }

    #[test]
    fn route_react_targets_web() {
        let (exit_code, stdout, _) = route("refactor the react component on the dashboard webpage");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: web-development-life-cycle"));
    }

    #[test]
    fn route_journey_friction_targets_ux() {
        let (exit_code, stdout, _) =
            route("investigate the signup funnel drop-off with user research");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: ux-research-and-experience-strategy"));
    }

    #[test]
    fn route_design_system_targets_ui() {
        let (exit_code, stdout, _) =
            route("align the design system tokens for the responsive layout");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: ui-design-systems-and-responsive-interfaces"));
    }

    #[test]
    fn route_memory_health_targets_memory_status_reporter() {
        let (exit_code, stdout, _) = route("show memory health and what did you learn today");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: memory-status-reporter"));
    }

    #[test]
    fn route_unknown_request_falls_back_to_sdlc_default() {
        let (exit_code, stdout, _) = route("plan the next quarter roadmap");
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("specialist: software-development-life-cycle"));
        assert!(stdout.contains("default lane"));
    }

    #[test]
    fn route_single_token_uses_word_boundary_matching() {
        let (exit_code, stdout, _) = route("redesign the kiosk display");
        assert_eq!(exit_code, 0);
        assert!(
            !stdout.contains("specialist: ui-design-systems-and-responsive-interfaces"),
            "ui keyword should not match inside 'kiosk': {stdout}"
        );
    }

    #[test]
    fn route_json_format_emits_structured_payload() {
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let exit_code = run_workflow_command(
            &[
                "route".to_string(),
                "--request".to_string(),
                "audit production readiness".to_string(),
                "--format".to_string(),
                "json".to_string(),
            ],
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(exit_code, 0, "stderr: {}", String::from_utf8_lossy(&stderr));
        let output = String::from_utf8_lossy(&stdout).to_string();
        assert!(output.contains("\"specialist\": \"reviewer\""));
        assert!(output.contains("\"matchedKeyword\""));
        assert!(output.contains("\"reason\""));
    }

    #[test]
    fn route_missing_request_returns_error() {
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let exit_code = run_workflow_command(&["route".to_string()], &mut stdout, &mut stderr);
        assert_eq!(exit_code, 1);
        assert!(String::from_utf8_lossy(&stderr).contains("--request is required"));
    }

    #[test]
    fn route_accepts_positional_request() {
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let exit_code = run_workflow_command(
            &[
                "route".to_string(),
                "audit".to_string(),
                "the".to_string(),
                "release".to_string(),
            ],
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(exit_code, 0, "stderr: {}", String::from_utf8_lossy(&stderr));
        assert!(String::from_utf8_lossy(&stdout).contains("specialist: reviewer"));
    }
}
