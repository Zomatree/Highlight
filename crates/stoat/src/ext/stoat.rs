use async_trait::async_trait;
use stoat_models::v0::{Channel, FetchServerResponse, OptionsFetchServer, Server, User};

use crate::{Client, Context, Error, HttpClient, Result, types::Tag};

#[async_trait]
pub trait StoatExt {
    #[doc(hidden)]
    fn _http(&self) -> &HttpClient;

    async fn fetch_user(&self, user_id: &str) -> Result<User> {
        self._http().fetch_user(user_id).await
    }

    async fn upload_file(&self, tag: Tag, data: &[u8]) -> Result<String> {
        let response = self._http().upload_file(tag.as_str(), data).await?;

        Ok(response.id)
    }

    async fn fetch_server(&self, server_id: &str) -> Result<Server> {
        let FetchServerResponse::JustServer(server) = self
            ._http()
            .fetch_server(
                server_id,
                &OptionsFetchServer {
                    include_channels: None,
                },
            )
            .await?
        else {
            return Err(Error::InternalError);
        };

        Ok(server)
    }

    async fn fetch_server_with_channels(&self, server_id: &str) -> Result<(Server, Vec<Channel>)> {
        let FetchServerResponse::ServerWithChannels { server, channels } = self
            ._http()
            .fetch_server(
                server_id,
                &OptionsFetchServer {
                    include_channels: Some(true),
                },
            )
            .await?
        else {
            return Err(Error::InternalError);
        };

        Ok((server, channels))
    }

    async fn fetch_dms(&self) -> Result<Vec<Channel>> {
        self._http().fetch_dms().await
    }
    async fn fetch_self(&self) -> Result<User> {
        self._http().fetch_self().await
    }
}

impl StoatExt for Context {
    fn _http(&self) -> &HttpClient {
        &self.http
    }
}

impl<H> StoatExt for Client<H> {
    fn _http(&self) -> &HttpClient {
        &self.http
    }
}
