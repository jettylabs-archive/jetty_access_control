//! TableauRestClient and generic utilities to help with Tableau
//! API requests

mod cual;

use std::{
    io::{Cursor, Read},
    sync::Once,
};

use super::*;
use cual::set_cual_prefix;
pub(crate) use cual::{get_tableau_cual, TableauAssetType};

use anyhow::{bail, Context};
use async_trait::async_trait;
use bytes::{Buf, Bytes};
use serde::Serialize;
use zip;

pub(crate) trait Downloadable {
    fn get_path(&self) -> String;

    /// a function the unzipper will use to make sure we return the correct file
    fn match_file(name: &str) -> bool;
}
/// Wrapper struct for http functionality
#[derive(Default)]
/// The credentials used to authenticate into Snowflake.
pub(crate) struct TableauRestClient {
    credentials: TableauCredentials,
    http_client: reqwest::Client,
    token: Option<String>,
    site_id: Option<String>,
    api_version: String,
}

impl TableauRestClient {
    /// Initialize a new TableauRestClient
    ///
    ///  # Panics
    /// ------
    /// Will panic if run in an asynchronous context
    pub async fn new(credentials: TableauCredentials) -> Self {
        // Set the global CUAL prefix for tableau
        set_cual_prefix(&credentials.server_name, &credentials.site_name);
        let mut tc = TableauRestClient {
            credentials,
            http_client: reqwest::Client::builder().gzip(true).build().unwrap(),
            token: None,
            site_id: None,
            api_version: "3.16".to_owned(),
        };
        tc.fetch_token_and_site_id().await.unwrap();
        tc
    }

    pub(crate) async fn download<T: Downloadable>(
        &self,
        asset: &T,
        exclude_extracts: bool,
    ) -> Result<Bytes> {
        let mut req = self
            .build_request(asset.get_path(), None, reqwest::Method::GET)?
            .header("Accept", "application/zip, application/octet-stream");

        if exclude_extracts {
            req = req.query(&[("includeExtract", "False")]);
        }

        req.send().await?.bytes().await.context("downloading file")
    }

    #[cfg(test)]
    pub(crate) fn new_dummy() -> Self {
        let tc = TableauRestClient {
            credentials: TableauCredentials {
                server_name: "dummy-server".to_owned(),
                site_name: "dummy-site".to_owned(),
                ..Default::default()
            },
            http_client: reqwest::Client::builder().gzip(true).build().unwrap(),
            token: None,
            site_id: None,
            api_version: "3.16".to_owned(),
        };
        tc
    }

    /// Get site_id token from the TableauRestClient.
    fn get_site_id(&self) -> Result<String> {
        Ok(self
            .site_id
            .as_ref()
            .ok_or_else(|| anyhow!["unable to find site_id"])?
            .to_owned())
    }

    /// Get authentication token from the TableauRestClient.
    fn get_token(&self) -> Result<String> {
        Ok(self
            .token
            .as_ref()
            .ok_or_else(|| anyhow!["unable to find token"])?
            .to_owned())
    }

    /// Make a blocking request to fetch Tableau Site's token and site_id
    ///
    /// # Panics
    /// ------
    /// Will panic if run in an asynchronous context
    async fn fetch_token_and_site_id(&mut self) -> Result<()> {
        // Set up the request body to get a request token
        let request_body = json!({
            "credentials": {
                "name" : &self.credentials.username,
                "password": &self.credentials.password,
                "site": {
                    "contentUrl": &self.credentials.site_name,
                }
            }
        });

        let resp = reqwest::Client::new()
            .post(format![
                "https://{}/api/{}/auth/signin",
                &self.credentials.server_name, &self.api_version
            ])
            .json(&request_body)
            .header("Accept".to_owned(), "application/json".to_owned())
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let token = get_json_from_path(&resp, &vec!["credentials".to_owned(), "token".to_owned()])?
            .as_str()
            .ok_or_else(|| anyhow!["unable to get token from response"])?
            .to_string();
        self.token = Some(token);

        let site_id = get_json_from_path(
            &resp,
            &vec!["credentials".to_owned(), "site".to_owned(), "id".to_owned()],
        )?
        .as_str()
        .ok_or_else(|| anyhow!["unable to get token from response"])?
        .to_string();
        self.site_id = Some(site_id);
        Ok(())
    }

    async fn get_json_response(
        &mut self,
        endpoint: String,
        body: Option<serde_json::Value>,
        method: reqwest::Method,
        path_to_paginated_iterable: Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        let req = self
            .build_request(endpoint, body, method)
            .context("building request")?;

        let resp = req
            .try_clone()
            .ok_or_else(|| anyhow!("unable to clone request"))?
            .send()
            .await
            .context("making request")?;

        let parsed_response = resp
            .json::<serde_json::Value>()
            .await
            .context("parsing json response")?;

        // Check for pagination
        if let Some(v) = parsed_response.get("pagination") {
            #[derive(Deserialize)]
            struct PaginationInfo {
                #[serde(rename = "pageSize")]
                page_size: String,
                #[serde(rename = "totalAvailable")]
                total_available: String,
            }
            let info: PaginationInfo =
                serde_json::from_value(v.to_owned()).context("parsing pagination information")?;

            let (page_size, total_available) = (
                info.page_size.parse::<usize>()?,
                info.total_available.parse::<usize>()?,
            );

            // Only need to paginate if there are more results than shown on the first page
            let path_to_paginated_iterable = &path_to_paginated_iterable.ok_or_else(|| {
                anyhow!["cannot use paginated results without path_to_paginated_iterable"]
            })?;

            let extra_page = if total_available % page_size == 0 {
                0
            } else {
                1
            };
            let total_required_pages = total_available / page_size + extra_page;

            let mut results_vec = vec![];

            // get first page of results
            if let serde_json::Value::Array(vals) =
                get_json_from_path(&parsed_response, path_to_paginated_iterable)
                    .context("getting target json object")?
            {
                results_vec.extend(vals);
            } else {
                bail!["Unable to find target array"];
            };

            for page_number in 2..total_required_pages + 1 {
                let paged_resp = req
                    .try_clone()
                    .ok_or_else(|| anyhow!("unable to clone request"))?
                    // add a page number to the request
                    .query(&[("pageNumber", page_number.to_string())])
                    .send()
                    .await
                    .context("making request")?
                    .json::<serde_json::Value>()
                    .await
                    .context("parsing json response")?;

                // get each additional page of results
                if let serde_json::Value::Array(vals) =
                    get_json_from_path(&paged_resp, path_to_paginated_iterable)
                        .context("getting target json object")?
                {
                    results_vec.extend(vals);
                } else {
                    return Err(anyhow!["Unable to find target array"]);
                };
            }
            Ok(serde_json::Value::Array(results_vec))
        } else {
            Ok(parsed_response)
        }
    }

    /// Builds a request to fetch information from tableau
    pub(crate) fn build_request(
        &self,
        endpoint: String,
        body: Option<serde_json::Value>,
        method: reqwest::Method,
    ) -> Result<reqwest::RequestBuilder> {
        let request_url = format![
            "https://{}/api/{}/sites/{}/{}",
            self.credentials.server_name.to_owned(),
            self.api_version.to_owned(),
            self.get_site_id()?,
            endpoint,
        ];

        let mut req = self.http_client.request(method, request_url);
        req = self
            .add_auth(req)
            .context("adding auth header")?
            .header("Accept", "application/json")
            // In the case that pageSize is allowed, set it to the max
            .query(&[("pageSize", "1000")]);

        // Add body if exists
        if let Some(b) = body {
            req = req.json(&b);
        }

        Ok(req)
    }

    /// Build a request to be run against the graphql endpoint.  
    /// This function does not currently support variables.
    pub(crate) fn build_graphql_request(&self, query: String) -> Result<reqwest::RequestBuilder> {
        #[derive(Serialize)]
        struct GraphQlQuery {
            query: String,
            variables: HashMap<String, String>,
        }

        let query_struct = GraphQlQuery {
            query,
            variables: HashMap::new(),
        };

        let request_url = format![
            "https://{}/api/metadata/graphql",
            self.credentials.server_name.to_owned(),
        ];

        let mut req = self.http_client.request(reqwest::Method::POST, request_url);
        req = self
            .add_auth(req)
            .context("adding auth header")?
            .header("Accept", "application/json")
            .json(&query_struct);

        Ok(req)
    }

    /// This functions builds a request specifically to fetch ask
    /// data lenses. The URL is significantly different than those
    /// used for other asset types
    pub(crate) fn build_lens_request(
        &self,
        endpoint: String,
        body: Option<serde_json::Value>,
        method: reqwest::Method,
    ) -> Result<reqwest::RequestBuilder> {
        let request_url = format![
            "https://{}/api/-/{}",
            self.credentials.server_name.to_owned(),
            endpoint,
        ];

        let mut req = self.http_client.request(method, request_url);
        req = self
            .add_auth(req)
            .context("adding auth header")?
            .header("Accept", "application/json");
        // remove the PageSize query

        // Add body if exists
        if let Some(b) = body {
            req = req.json(&b);
        }

        Ok(req)
    }

    /// Add authentication header to requests
    fn add_auth(&self, req: reqwest::RequestBuilder) -> Result<reqwest::RequestBuilder> {
        let token = self.get_token().context("getting token")?;
        let req = req.header("X-Tableau-Auth", token);
        Ok(req)
    }
}

/// This function extracts and returns the first file in a zip archive that returns true for name_matcher
pub(crate) fn unzip_text_file(archive: Bytes, name_matcher: fn(&str) -> bool) -> Result<String> {
    let archive_cursor = Cursor::new(archive);

    let mut zip_archive = zip::ZipArchive::new(archive_cursor)?;

    let file_names = zip_archive
        .file_names()
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();

    for name in file_names {
        if name_matcher(&name) {
            let mut archive_file = zip_archive.by_name(&name)?;
            let mut data = String::new();
            archive_file.read_to_string(&mut data)?;

            return Ok(data);
        }
    }
    bail!("unable to find file to parse");
}

pub(crate) fn get_json_from_path(
    val: &serde_json::Value,
    path: &Vec<String>,
) -> Result<serde_json::Value> {
    let mut full_path: String = "Object".to_owned();
    let mut return_val = val;

    for p in path {
        full_path = format!("{}.{}", full_path, p);
        return_val = return_val.get(p).ok_or_else(|| {
            anyhow!(
                "unable to parse json - no such path exists: {}\n{}",
                full_path,
                val
            )
        })?;
    }
    Ok(return_val.to_owned())
}

#[async_trait]
pub(crate) trait FetchJson {
    async fn fetch_json_response(
        &self,
        path_to_paginated_iterable: Option<Vec<String>>,
    ) -> Result<serde_json::Value>;
}

#[async_trait]
impl FetchJson for reqwest::RequestBuilder {
    async fn fetch_json_response(
        &self,
        path_to_paginated_iterable: Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        let resp = self
            .try_clone()
            .ok_or_else(|| anyhow!("unable to clone request"))?
            .send()
            .await
            .context("making request")?;

        let parsed_response = resp
            .json::<serde_json::Value>()
            .await
            .context("parsing json response")?;

        // Check for pagination
        if let Some(v) = parsed_response.get("pagination") {
            #[derive(Deserialize)]
            struct PaginationInfo {
                #[serde(rename = "pageSize")]
                page_size: String,
                #[serde(rename = "totalAvailable")]
                total_available: String,
            }
            let info: PaginationInfo =
                serde_json::from_value(v.to_owned()).context("parsing pagination information")?;

            let (page_size, total_available) = (
                info.page_size.parse::<usize>()?,
                info.total_available.parse::<usize>()?,
            );

            // Early exit if there are no results
            if total_available == 0 {
                return Ok(json!([]));
            }

            // Only need to paginate if there are more results than shown on the first page
            let path_to_paginated_iterable = &path_to_paginated_iterable.ok_or_else(|| {
                anyhow!["cannot use paginated results without path_to_paginated_iterable"]
            })?;

            let extra_page = if total_available % page_size == 0 {
                0
            } else {
                1
            };
            let total_required_pages = total_available / page_size + extra_page;

            let mut results_vec = vec![];

            // get first page of results
            if let serde_json::Value::Array(vals) =
                get_json_from_path(&parsed_response, path_to_paginated_iterable)
                    .context("getting target json object")?
            {
                results_vec.extend(vals);
            } else {
                bail!["Unable to find target array"];
            };

            for page_number in 2..total_required_pages + 1 {
                let paged_resp = &self
                    .try_clone()
                    .ok_or_else(|| anyhow!("unable to clone request"))?
                    // add a page number to the request
                    .query(&[("pageNumber", page_number.to_string())])
                    .send()
                    .await
                    .context("making request")?
                    .json::<serde_json::Value>()
                    .await
                    .context("parsing json response")?;

                // get each additional page of results
                if let serde_json::Value::Array(vals) =
                    get_json_from_path(paged_resp, path_to_paginated_iterable)
                        .context("getting target json object")?
                {
                    results_vec.extend(vals);
                } else {
                    return Err(anyhow!["Unable to find target array"]);
                };
            }
            Ok(serde_json::Value::Array(results_vec))
        } else {
            Ok(parsed_response)
        }
    }
}

#[cfg(ignore)]
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use jetty_core::{connectors::ConnectorClient, jetty};

    #[test]
    fn test_fetching_token_works() -> Result<()> {
        connector_setup().context("running tableau connector setup")?;
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_users_works() -> Result<()> {
        let mut tc = tokio::task::spawn_blocking(|| {
            connector_setup().context("running tableau connector setup")
        })
        .await??;
        let users = tc.client.get_users().await?;
        for (_k, v) in users {
            println!("{}", v.name);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_assets_works() -> Result<()> {
        let mut tc = tokio::task::spawn_blocking(|| {
            connector_setup().context("running tableau connector setup")
        })
        .await??;
        let assets = tc.client.get_assets().await?;
        for a in assets {
            println!("{:#?}", a);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetching_groups_works() -> Result<()> {
        let mut tc = tokio::task::spawn_blocking(|| {
            connector_setup().context("running tableau connector setup")
        })
        .await??;
        let groups = tc.client.get_groups().await?;
        for (_k, v) in groups {
            println!("{:#?}", v);
        }
        Ok(())
    }

    fn connector_setup() -> Result<TableauConnector> {
        let j = jetty::Jetty::new().context("creating Jetty")?;
        let creds = jetty::fetch_credentials().context("fetching credentials from file")?;
        let config = &j.config.connectors[0];
        let tc = TableauConnector::new(config, &creds["tableau"], Some(ConnectorClient::Test))
            .context("reading tableau credentials")?;
        Ok(*tc)
    }
}