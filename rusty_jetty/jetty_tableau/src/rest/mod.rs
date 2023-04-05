//! TableauRestClient and generic utilities to help with Tableau
//! API requests

mod cual;

use super::*;
pub(crate) use cual::get_cual_prefix;
#[cfg(not(test))]
use cual::set_cual_prefix;
#[cfg(test)]
pub(crate) use cual::set_cual_prefix;
pub(crate) use cual::{get_tableau_cual, TableauAssetType};

use anyhow::{bail, Context};
use async_trait::async_trait;

use jetty_core::logging::error;
use reqwest::Request;
use serde::Serialize;

/// Wrapper struct for http functionality
#[derive(Default)]
pub struct TableauRestClient {
    /// The credentials used to authenticate into Tableau.
    pub(crate) credentials: TableauCredentials,
    http_client: reqwest::Client,
    token: Option<String>,
    site_id: Option<String>,
    api_version: String,
}

impl TableauRestClient {
    /// Initialize a new TableauRestClient
    pub async fn new(credentials: TableauCredentials) -> Result<Self> {
        // Set the global CUAL prefix for tableau
        set_cual_prefix(&credentials.server_name, &credentials.site_name);
        let mut tc = TableauRestClient {
            credentials,
            http_client: reqwest::Client::builder().gzip(true).build().unwrap(),
            token: None,
            site_id: None,
            api_version: "3.16".to_owned(),
        };
        tc.fetch_token_and_site_id().await?;
        Ok(tc)
    }

    pub(crate) async fn execute(
        &self,
        request: Request,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client.execute(request).await
    }

    /// Get site_id token from the TableauRestClient.
    pub(crate) fn get_site_id(&self) -> Result<String> {
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

    /// Make a request to fetch Tableau Site's token and site_id
    async fn fetch_token_and_site_id(&mut self) -> Result<()> {
        // Set up the request body to get a request token
        let request_body = match &self.credentials.method {
            LoginMethod::UsernameAndPassword { username, password } => json!({
                "credentials": {
                    "name" : username,
                    "password": password,
                    "site": {
                        "contentUrl": &self.credentials.site_name,
                    }
                }
            }),
            LoginMethod::PersonalAccessToken { token_name, secret } => json!({
                "credentials": {
                    "personalAccessTokenName" : token_name,
                    "personalAccessTokenSecret": secret,
                    "site": {
                        "contentUrl": &self.credentials.site_name,
                    }
                }
            }),
        };

        let resp = reqwest::Client::new()
            .post(format![
                "https://{}/api/{}/auth/signin",
                &self.credentials.server_name, &self.api_version
            ])
            .json(&request_body)
            .header("Accept".to_owned(), "application/json".to_owned())
            .send()
            .await?
            .error_for_status()?
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

pub(crate) fn get_json_from_path(
    val: &serde_json::Value,
    path: &Vec<String>,
) -> Result<serde_json::Value> {
    let mut full_path: String = "Object".to_owned();
    let mut return_val = val;

    for p in path {
        full_path = format!("{full_path}.{p}");
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
    /// Fetch an optionally paginated Tableau JSON response.
    /// path_to_paginated_iterable is the path to the iterable that will be used to create
    /// the final results. For example, for views, they are listed in an array found at views.view.
    /// In this case, the argument would be `Some(vec!["views".to_owned(), "view".to_owned()]))`
    ///
    /// In the case of permissions and graphql requests, this type of pagination is not used.
    /// permissions responses are not paginated, and graphql, when paginated, uses a different scheme.
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
        let cloned_req = self
            .try_clone()
            .ok_or_else(|| anyhow!("unable to clone request"))?;
        // Build the request so that we can inspect it in the case of an error
        let built_req = self.try_clone().map(|r| r.build());
        let req_method = if let Some(Ok(built_req)) = &built_req {
            Some(built_req.method().to_owned())
        } else {
            None
        };
        let req_url = if let Some(Ok(built_req)) = built_req {
            Some(built_req.url().to_owned())
        } else {
            None
        };

        let resp = cloned_req
            .send()
            .await
            .map_err(|e| {
                if let (Some(method), Some(url)) = (&req_method, &req_url) {
                    error!("error with request ({method:?} {url:?}): {e}")
                } else {
                    error!("error with request: {e}")
                };
                e
            })
            .context("making request")?
            .error_for_status()
            .map_err(|e| {
                if let (Some(method), Some(url)) = (&req_method, &req_url) {
                    error!("error with request - bad response ({method:?} {url:?}): {e}")
                } else {
                    error!("error with request - bad response: {e}")
                };
                e
            })?;

        let parsed_response = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|e| {
                let request = if let (Some(method), Some(url)) = (&req_method, &req_url) {
                    format!("({method:?} {url:?})")
                } else {
                    "".to_owned()
                };
                error!("error parsing json response {request}: {e}");
                e
            })
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

            let extra_page = usize::from(total_available % page_size != 0);
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
                    .error_for_status()
                    .map_err(|e| {
                        if let (Some(method), Some(url)) = (&req_method, &req_url) {
                            error!("error with pagination request - bad response ({method:?} {url:?}): {e}")
                        } else {
                            error!("error with pagination request - bad response: {e}")
                        };
                        e
                    })?
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
