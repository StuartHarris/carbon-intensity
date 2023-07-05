use crux_core::render::Render;
use crux_http::Http;
use crux_macros::Effect;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    capabilities::location::{GetLocation, LocationResponse},
    model::{global::Set, postcode, regional},
    view_model,
};

const POSTCODE_API: &str = "https://api.postcodes.io/postcodes";
const INTENSITY_API: &str = "https://api.carbonintensity.org.uk/";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum Mode {
    #[default]
    National,
    Here,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    SwitchMode(Mode),

    // events local to the core
    #[serde(skip)]
    SetLocation(LocationResponse),
    SetPostcode(crux_http::Result<crux_http::Response<postcode::PostCode>>),
    SetRegional(crux_http::Result<crux_http::Response<regional::Root>>),
}

#[derive(Default)]
pub struct Model {
    mode: Mode,
    national: Set,
    here: Set,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub window: Vec<view_model::Period>,
}

#[cfg_attr(feature = "typegen", derive(crux_macros::Export))]
#[derive(Effect)]
pub struct Capabilities {
    render: Render<Event>,
    location: GetLocation<Event>,
    pub http: Http<Event>,
}

#[derive(Default)]
pub struct App;

impl crux_core::App for App {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Capabilities = Capabilities;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        match event {
            Event::SwitchMode(Mode::National) => {
                model.national = Set::default();
            }
            Event::SwitchMode(Mode::Here) => {
                caps.location.get(Event::SetLocation);
            }
            Event::SetLocation(LocationResponse {
                location: Some(location),
            }) => {
                let url = Url::parse_with_params(
                    POSTCODE_API,
                    &[
                        ("lon", location.longitude.to_string()),
                        ("lat", location.latitude.to_string()),
                    ],
                )
                .unwrap();
                caps.http.get(url).expect_json().send(Event::SetPostcode);
            }
            Event::SetLocation(LocationResponse { location: None }) => {}
            Event::SetPostcode(Ok(mut postcode)) => {
                let postcode = postcode.take_body().unwrap();
                let url = &mut Url::parse(INTENSITY_API)
                    .unwrap()
                    .join(&format!(
                        "regional/{from}/fw24h/postcode/{postcode}",
                        from = "2023-07-05T00:00Z",
                        postcode = postcode.outcode,
                    ))
                    .unwrap();
                caps.http.get(&url).expect_json().send(Event::SetRegional);
            }
            Event::SetPostcode(Err(_)) => {}
            Event::SetRegional(Ok(mut _regional)) => todo!(),
            Event::SetRegional(Err(_)) => {}
        };

        caps.render.render();
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            window: match model.mode {
                Mode::National => model.national.clone().into(),
                Mode::Here => model.here.clone().into(),
            },
        }
    }
}
