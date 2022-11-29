pub(crate) fn should_filter(relation_name: &str) -> bool {
    relation_name.starts_with("test")
        || relation_name.starts_with("analysis")
        || relation_name.starts_with("exposure")
}
