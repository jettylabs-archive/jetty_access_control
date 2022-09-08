#[derive(Clone)]
pub(crate) struct DataConnection {
    pub id: String,
    pub connection_type: String,
    pub user_name: Option<String>,
    pub derived_from: Vec<String>,
}
