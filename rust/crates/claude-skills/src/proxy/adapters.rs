//! Purpose: Build the command-adapter registry for the token-saving proxy.
//! Caller: proxy::run when preparing a proxied command.
//! Dependencies: Built-in Rust adapters plus optional project filter adapters.
//! Main Functions: build_adapter_registry.
//! Side Effects: Reads optional project filter configuration files from the current workspace.

use crate::adapters::build::BuildAdapter;
use crate::adapters::containers::ContainersAdapter;
use crate::adapters::files::FilesAdapter;
use crate::adapters::generic::GenericAdapter;
use crate::adapters::git::GitAdapter;
use crate::adapters::lint::LintAdapter;
use crate::adapters::logs::LogsAdapter;
use crate::adapters::search::SearchAdapter;
use crate::adapters::tests::TestAdapter;
use crate::proxy::registry::AdapterRegistry;

pub fn build_adapter_registry() -> AdapterRegistry {
    let mut registry = AdapterRegistry::new();
    registry.register(Box::new(TestAdapter));
    registry.register(Box::new(GitAdapter));
    registry.register(Box::new(SearchAdapter));
    registry.register(Box::new(FilesAdapter));
    registry.register(Box::new(BuildAdapter));
    registry.register(Box::new(LintAdapter));
    registry.register(Box::new(ContainersAdapter));
    registry.register(Box::new(LogsAdapter));
    for adapter in crate::proxy::filters::load_project_filter_adapters() {
        registry.register(adapter);
    }
    registry.register(Box::new(GenericAdapter));
    registry
}

pub fn adapter_names() -> &'static str {
    "tests, git, search, files, build, lint, containers, logs, project-filter, generic"
}

#[cfg(test)]
mod tests {
    use super::build_adapter_registry;
    use crate::proxy::classify::classify_command;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn registry_selects_specific_adapters_before_generic() {
        let registry = build_adapter_registry();
        let ast = classify_command(&args(&["cargo", "test", "--workspace"])).expect("ast");
        assert_eq!(registry.best_match(&ast).expect("adapter").name(), "tests");

        let ast = classify_command(&args(&["git", "diff", "--cached"])).expect("ast");
        assert_eq!(registry.best_match(&ast).expect("adapter").name(), "git");

        let ast = classify_command(&args(&["rg", "foo", "."])).expect("ast");
        assert_eq!(registry.best_match(&ast).expect("adapter").name(), "search");

        let ast = classify_command(&args(&["cargo", "build"])).expect("ast");
        assert_eq!(registry.best_match(&ast).expect("adapter").name(), "build");

        let ast = classify_command(&args(&["eslint", "."])).expect("ast");
        assert_eq!(registry.best_match(&ast).expect("adapter").name(), "lint");

        let ast = classify_command(&args(&["totally-unknown", "--loud"])).expect("ast");
        assert_eq!(
            registry.best_match(&ast).expect("adapter").name(),
            "generic"
        );
    }

    #[test]
    fn forced_adapter_lookup_supports_distinct_build_and_lint_adapters() {
        let registry = build_adapter_registry();
        assert!(registry.find_by_name("build").is_some());
        assert!(registry.find_by_name("lint").is_some());
        assert!(registry.find_by_name("generic").is_some());
        assert!(registry.find_by_name("missing").is_none());
    }
}
