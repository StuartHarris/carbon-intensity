use crux_core::render::Render;
use crux_http::Http;
use crux_macros::Effect;
use serde::{Deserialize, Serialize};

use crate::{
    capabilities::location::{GetLocation, LocationResponse},
    model::{
        intensity::{self, Set},
        postcode, regional, Model,
    },
    view_model, Mode,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Event {
    SwitchMode(Mode),

    // events local to the core
    #[serde(skip)]
    SetLocation(LocationResponse),
    #[serde(skip)]
    SetPostcode(crux_http::Result<crux_http::Response<postcode::PostcodeResponse>>),
    #[serde(skip)]
    SetRegional(crux_http::Result<crux_http::Response<regional::RegionalResponse>>),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub mode: Mode,
    pub outcode: Option<String>,
    pub admin_district: Option<String>,
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
                caps.http
                    .get(postcode::url())
                    .query(&postcode::Query::from(location))
                    .unwrap()
                    .expect_json()
                    .send(Event::SetPostcode);
            }
            Event::SetLocation(LocationResponse { location: None }) => {}
            Event::SetPostcode(Ok(mut postcode)) => {
                let postcode = postcode.take_body().unwrap();
                let postcode = postcode.result[0].clone();
                let outcode = postcode.outcode; // TODO error handling
                let from = "2023-07-05T00:00Z"; // TODO
                let url = intensity::url(&from, &outcode);

                model.outcode = Some(outcode);
                model.admin_district = Some(postcode.admin_district.clone()); // TODO error handling

                caps.http.get(url).expect_json().send(Event::SetRegional);
                caps.render.render();
            }
            Event::SetPostcode(Err(_)) => {}
            Event::SetRegional(Ok(mut regional)) => {
                let regional = regional.take_body().unwrap();
                let future = regional.data.data.clone();
                model.here = Set {
                    future,
                    ..Default::default()
                };

                caps.render.render();
            }
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
            mode: model.mode.clone(),
            outcode: model.outcode.clone(),
            admin_district: model.admin_district.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        location::Location, postcode::PostcodeResponse, regional::RegionalResponse,
    };
    use crux_core::{assert_effect, testing::AppTester};
    use crux_http::{
        protocol::{HttpRequest, HttpResponse},
        testing::ResponseBuilder,
    };

    #[test]
    fn regional_happy_path() {
        let app = AppTester::<App, _>::default();
        let mut model = Model::default();

        // switch to "here" mode and check we get a location request
        let requests = &mut app
            .update(Event::SwitchMode(Mode::Here), &mut model)
            .into_effects()
            .filter_map(Effect::into_location);

        // get the first location request and check there are no more
        let mut request = requests.next().unwrap();
        assert!(requests.next().is_none());

        // resolve a simulated location response
        let response = LocationResponse {
            location: Some(Location {
                latitude: 51.403366,
                longitude: -0.298302,
            }),
        };
        let update = app.resolve(&mut request, response.clone()).unwrap();

        // check the this raises a SetLocation event
        let set_location_event = Event::SetLocation(response.clone());
        let actual = &update.events;
        let expected = &vec![set_location_event.clone()];
        assert_eq!(actual, expected);

        // check that the SetLocation event results in a postcode request
        let requests = &mut app
            .update(set_location_event, &mut model)
            .into_effects()
            .filter_map(Effect::into_http);

        // get the first http request and check there are no more
        let mut request = requests.next().unwrap();
        assert!(requests.next().is_none());

        // check the postcode request has the expected url
        let actual = &request.operation;
        let expected =
            &HttpRequest::get("https://api.postcodes.io/postcodes?lat=51.403366&lon=-0.298302")
                .build();
        assert_eq!(actual, expected);

        // resolve a simulated postcode response
        let simulated_response: PostcodeResponse =
            serde_json::from_str(include_str!("./fixtures/postcode.json")).unwrap();
        let response = HttpResponse::status(200).json(&simulated_response).build();
        let update = app.resolve(&mut request, response).unwrap();

        // check the postcode response raises a SetPostcode event
        let set_postcode_event = Event::SetPostcode(Ok(ResponseBuilder::ok()
            .body(simulated_response)
            .build()
            .clone()));
        let actual = &update.events;
        let expected = &vec![set_postcode_event.clone()];
        assert_eq!(actual, expected);

        // check that the SetPostcode event results in a render extract a regional request
        let update = app.update(set_postcode_event, &mut model);
        assert_effect!(&update, Effect::Render(_));
        let requests = &mut update.into_effects().filter_map(Effect::into_http);

        // check that the outcode and admin district have been set
        assert_eq!(model.outcode, Some("KT1".to_string()));
        assert_eq!(
            model.admin_district,
            Some("Kingston upon Thames".to_string())
        );

        // get the first http request and check there are no more
        let mut request = requests.next().unwrap();
        assert!(requests.next().is_none());

        // check the regional request has the expected url
        let actual = &request.operation;
        let expected = &HttpRequest::get(
            "https://api.carbonintensity.org.uk/regional/intensity/2023-07-05T00:00Z/fw24h/postcode/KT1",
        )
        .build();
        assert_eq!(actual, expected);

        // resolve a simulated regional response
        let simulated_response: RegionalResponse =
            serde_json::from_str(include_str!("./fixtures/regional.json")).unwrap();
        let response = HttpResponse::status(200).json(&simulated_response).build();
        let update = app.resolve(&mut request, response).unwrap();

        // check the regional response raises a SetRegional event
        let set_regional_event = Event::SetRegional(Ok(ResponseBuilder::ok()
            .body(simulated_response)
            .build()
            .clone()));
        let actual = &update.events;
        let expected = &vec![set_regional_event.clone()];
        assert_eq!(actual, expected);

        // check that the SetRegional event updates the model and renders
        for event in update.events {
            let update = app.update(event, &mut model);
            assert_effect!(update, Effect::Render(_));
        }
        insta::assert_yaml_snapshot!(model.here, @r###"
        ---
        past: []
        future:
          - from: "2023-07-04T23:30:00Z"
            to: "2023-07-05T00:00:00Z"
            intensity:
              forecast: 121
              actual: ~
              index: moderate
            generationmix:
              - fuel: biomass
                perc: 0
              - fuel: coal
                perc: 0
              - fuel: imports
                perc: 66.1
              - fuel: gas
                perc: 17.2
              - fuel: nuclear
                perc: 0
              - fuel: other
                perc: 0
              - fuel: hydro
                perc: 0.2
              - fuel: solar
                perc: 0
              - fuel: wind
                perc: 16.5
          - from: "2023-07-05T00:00:00Z"
            to: "2023-07-05T00:30:00Z"
            intensity:
              forecast: 116
              actual: ~
              index: low
            generationmix:
              - fuel: biomass
                perc: 0
              - fuel: coal
                perc: 0
              - fuel: imports
                perc: 65.6
              - fuel: gas
                perc: 16.1
              - fuel: nuclear
                perc: 0
              - fuel: other
                perc: 0
              - fuel: hydro
                perc: 0.2
              - fuel: solar
                perc: 0.1
              - fuel: wind
                perc: 18
        "###);
    }
}
