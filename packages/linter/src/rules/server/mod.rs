pub mod architecture;
pub mod helpers;
pub mod performance;
pub mod reliability;
pub mod security;

use super::LintRule;

pub fn all_server_rules() -> Vec<Box<dyn LintRule>> {
    let mut rules = Vec::new();
    rules.extend(security::all_rules());
    rules.extend(reliability::all_rules());
    rules.extend(performance::all_rules());
    rules.extend(architecture::all_rules());
    rules
}
