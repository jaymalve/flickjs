pub mod architecture;
pub mod correctness;
pub mod helpers;
pub mod hooks;
pub mod nextjs;
pub mod performance;
pub mod react_native;
pub mod server_components;

use super::LintRule;

pub fn all_react_rules() -> Vec<Box<dyn LintRule>> {
    let mut rules = Vec::new();
    rules.extend(hooks::all_rules());
    rules.extend(correctness::all_rules());
    rules.extend(architecture::all_rules());
    rules.extend(performance::all_rules());
    rules.extend(nextjs::all_rules());
    rules.extend(server_components::all_rules());
    rules.extend(react_native::all_rules());
    rules
}
