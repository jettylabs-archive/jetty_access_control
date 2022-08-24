/// Build a fn to execute the given query and deserialize the result into the given type.
#[macro_export]
macro_rules! query_fn {
    ( $name:tt, $return_type:ty, $query:literal) => {
        /// Execute the given query and deserialize the result into the given type.
        #[inline(never)]
        pub async fn $name(&self) -> Result<Vec<$return_type>> {
            let result = self.query($query).await.context("query failed")?;
            let rows_value: JsonValue =
                serde_json::from_str(&result).context("failed to deserialize")?;
            let rows_data = rows_value["data"].clone();
            let rows: Vec<Vec<Value>> =
                serde_json::from_value::<Vec<Vec<Option<String>>>>(rows_data)
                    .context("failed to deserialize rows")?
                    .iter()
                    .map(|i| {
                        i.iter()
                            .map(|x| Value::new(x.clone().unwrap_or_else(|| String::new())))
                            .collect()
                    })
                    .collect();
            let fields_intermediate: Vec<SnowflakeField> =
                serde_json::from_value(rows_value["resultSetMetaData"]["rowType"].clone())
                    .context("failed to deserialize fields")?;
            let fields: Vec<String> = fields_intermediate.iter().map(|i| i.name.clone()).collect();
            println!("fields: {:?}", fields);
            Ok(rows
                .iter()
                .map(|i| {
                    // Zip field - i
                    let map: GenericMap = zip(fields.clone(), i.clone()).collect();
                    map
                })
                .map(|i| <$return_type>::from_genericmap(i))
                .collect())
        }
    };
}
