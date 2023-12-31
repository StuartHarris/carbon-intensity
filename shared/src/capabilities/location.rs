use serde::{Deserialize, Serialize};

use crux_core::capability::{CapabilityContext, Operation};
use crux_macros::Capability;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LocationRequest;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LocationResponse {
    pub location: Option<Coordinate>,
}

impl Operation for LocationRequest {
    type Output = LocationResponse;
}

#[derive(Capability)]
pub struct GetLocation<Ev> {
    context: CapabilityContext<LocationRequest, Ev>,
}

impl<Ev> GetLocation<Ev>
where
    Ev: 'static,
{
    pub fn new(context: CapabilityContext<LocationRequest, Ev>) -> Self {
        Self { context }
    }

    pub fn get<F>(&self, callback: F)
    where
        F: Fn(LocationResponse) -> Ev + Send + Sync + 'static,
    {
        self.context.spawn({
            let context = self.context.clone();
            async move {
                let response = context.request_from_shell(LocationRequest).await;

                context.update_app(callback(response));
            }
        });
    }
}
