/* This file is generated. Do not edit manually. */
#[allow(clippy::too_many_lines)]
#[allow(clippy::match_same_arms)]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::match_single_binding)]
#[must_use]
pub fn get_test_instance_json(path: &str) -> Option<&'static str> {
    match path {
        "lithummc:creeper_should_run_from_cat" => Some(include_str!(
            "../../../../assets/tests/datapacks/lithummc-unit-test/data/lithummc/test_instance/creeper_should_run_from_cat.json"
        )),
        "lithummc:summon_command_regression" => Some(include_str!(
            "../../../../assets/tests/datapacks/lithummc-unit-test/data/lithummc/test_instance/summon_command_regression.json"
        )),
        "minecraft:always_pass" | "always_pass" => Some(include_str!(
            "../../../../assets/datapack/data/minecraft/test_instance/always_pass.json"
        )),
        _ => None,
    }
}
#[must_use]
#[allow(clippy::too_many_lines, clippy::large_stack_arrays)]
pub const fn all_test_instance_names() -> &'static [&'static str] {
    &[
        "lithummc:creeper_should_run_from_cat",
        "lithummc:summon_command_regression",
        "minecraft:always_pass",
    ]
}
