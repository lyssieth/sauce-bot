use std::sync::Arc;

use twilight_gateway::Cluster;
use twilight_http::{client::InteractionClient, Client};
use twilight_model::id::{marker::ApplicationMarker, Id};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct Context {
    pub cluster: Arc<Cluster>,
    pub http: Arc<Client>,
    pub application_id: Id<ApplicationMarker>,
}

impl Context {
    #[must_use]
    pub fn interaction_client(&self) -> InteractionClient {
        self.http.interaction(self.application_id)
    }
}
