use super::*;

mod kdl_config_backward_compatibility {
    #[test]
    fn v0_3() {
        super::Config::parse(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/config/tests/2025_05_17_ferrishot_v0.3.kdl"
        ))
        .expect("ferrishot v0.3: The first released version of the config must never break");
    }
}
