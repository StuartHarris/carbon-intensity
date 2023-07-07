use chrono::{DateTime, Utc};
use crux_core::render::Render;
use crux_http::Http;
use crux_macros::Effect;
use serde::{Deserialize, Serialize};

use crate::{
    capabilities::{
        location::{GetLocation, LocationResponse},
        time::{Time, TimeResponse},
    },
    model::{national, postcode, regional, Model},
    view_model, Mode,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Event {
    SwitchMode(Mode),

    // events local to the core
    CurrentTime(TimeResponse),
    #[serde(skip)]
    SetLocation(LocationResponse),
    #[serde(skip)]
    SetPostcode(crux_http::Result<crux_http::Response<postcode::PostcodeResponse>>),
    #[serde(skip)]
    SetRegional(crux_http::Result<crux_http::Response<regional::RegionalResponse>>),
    #[serde(skip)]
    SetNational(crux_http::Result<crux_http::Response<national::NationalResponse>>),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub mode: Mode,
    pub outcode: Option<String>,
    pub admin_district: Option<String>,
    pub periods: Vec<view_model::Period>,
    // pub points: Vec<view_model::DataPoint>,
}

#[cfg_attr(feature = "typegen", derive(crux_macros::Export))]
#[derive(Effect)]
pub struct Capabilities {
    render: Render<Event>,
    location: GetLocation<Event>,
    time: Time<Event>,
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
                model.mode = Mode::National;
                caps.time.get(Event::CurrentTime);
            }
            Event::SwitchMode(Mode::Local) => {
                model.mode = Mode::Local;
                caps.time.get(Event::CurrentTime);
            }
            Event::CurrentTime(TimeResponse(iso_time)) => {
                model.time = DateTime::parse_from_rfc3339(&iso_time)
                    .unwrap()
                    .with_timezone(&Utc);
                match model.mode {
                    Mode::National => {
                        caps.http
                            .get(national::url(&model.time))
                            .expect_json()
                            .send(Event::SetNational);
                    }
                    Mode::Local => {
                        caps.location.get(Event::SetLocation);
                    }
                }
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
                let postcode = postcode.result[0].clone(); // TODO error handling
                let outcode = postcode.outcode;
                let url = regional::url(&model.time, &outcode);

                model.outcode = Some(outcode);
                model.admin_district = Some(postcode.admin_district.clone());

                caps.http.get(url).expect_json().send(Event::SetRegional);
                caps.render.render();
            }
            Event::SetPostcode(Err(_)) => {}
            Event::SetRegional(Ok(mut regional)) => {
                let regional = regional.take_body().unwrap();
                model.periods = regional.data.data.clone();

                caps.render.render();
            }
            Event::SetRegional(Err(_)) => {}
            Event::SetNational(Ok(mut national)) => {
                let regional = national.take_body().unwrap();
                model.periods = regional.data.clone();

                caps.render.render();
            }
            Event::SetNational(Err(_)) => {}
        };

        caps.render.render();
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            periods: model
                .periods
                .clone()
                .into_iter()
                .map(view_model::Period::from)
                .collect(),
            mode: model.mode.clone(),
            outcode: model.outcode.clone(),
            admin_district: model.admin_district.clone(),
            // points: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        location::Location, national::NationalResponse, postcode::PostcodeResponse,
        regional::RegionalResponse,
    };
    use crux_core::{assert_effect, testing::AppTester};
    use crux_http::{
        protocol::{HttpRequest, HttpResponse},
        testing::ResponseBuilder,
    };

    #[test]
    fn local_happy_path() {
        let app = AppTester::<App, _>::default();
        let mut model = Model::default();

        // switch to "local" mode and check we update the model and get a time request
        let update = app.update(Event::SwitchMode(Mode::Local), &mut model);
        assert_eq!(model.mode, Mode::Local);
        let requests = &mut update.into_effects().filter_map(Effect::into_time);

        // resolve the time request with a simulated time response
        let mut request = requests.next().unwrap();
        let response = TimeResponse("2023-07-06T20:30:00Z".to_string());
        let update = app.resolve(&mut request, response.clone()).unwrap();

        // check this raises the correct set time event
        let set_time_event = Event::CurrentTime(response.clone());
        let actual = &update.events;
        let expected = &vec![set_time_event.clone()];
        assert_eq!(actual, expected);

        // update the app and check it updates the model and we get a location request
        let update = app.update(set_time_event, &mut model);
        assert_eq!(
            model.time,
            DateTime::parse_from_rfc3339("2023-07-06T20:30:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
        let requests = &mut update.into_effects().filter_map(Effect::into_location);

        // resolve the location request with a simulated location response
        let mut request = requests.next().unwrap();
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
            "https://api.carbonintensity.org.uk/regional/intensity/2023-07-06T20:30Z/fw24h/postcode/KT1",
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
        insta::assert_yaml_snapshot!(model.periods, @r###"
        ---
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

    #[test]
    fn national_happy_path() {
        let app = AppTester::<App, _>::default();
        let mut model = Model::default();

        // switch to "national" mode and check we update the model and get a time request
        let update = app.update(Event::SwitchMode(Mode::National), &mut model);
        assert_eq!(model.mode, Mode::National);
        let requests = &mut update.into_effects().filter_map(Effect::into_time);

        // resolve the time request with a simulated time response
        let mut request = requests.next().unwrap();
        let response = TimeResponse("2023-07-06T20:30:00Z".to_string());
        let update = app.resolve(&mut request, response.clone()).unwrap();

        // check this raises the correct set time event
        let set_time_event = Event::CurrentTime(response.clone());
        let actual = &update.events;
        let expected = &vec![set_time_event.clone()];
        assert_eq!(actual, expected);

        // update the app and check it updates the model and we get an http request
        let update = app.update(set_time_event, &mut model);
        assert_eq!(
            model.time,
            DateTime::parse_from_rfc3339("2023-07-06T20:30:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
        let requests = &mut update.into_effects().filter_map(Effect::into_http);

        // get the first http request and check there are no more
        let mut request = requests.next().unwrap();
        assert!(requests.next().is_none());

        // check the national request has the expected url
        let actual = &request.operation;
        let expected = &HttpRequest::get(
            "https://api.carbonintensity.org.uk/intensity/2023-07-06T20:30Z/fw24h",
        )
        .build();
        assert_eq!(actual, expected);

        // resolve a simulated regional response
        let simulated_response: NationalResponse =
            serde_json::from_str(include_str!("./fixtures/national.json")).unwrap();
        let response = HttpResponse::status(200).json(&simulated_response).build();
        let update = app.resolve(&mut request, response).unwrap();

        // check the regional response raises a SetRegional event
        let set_national_event = Event::SetNational(Ok(ResponseBuilder::ok()
            .body(simulated_response)
            .build()
            .clone()));
        let actual = &update.events;
        let expected = &vec![set_national_event.clone()];
        assert_eq!(actual, expected);

        // check that the SetNational event updates the model and renders
        for event in update.events {
            let update = app.update(event, &mut model);
            assert_effect!(update, Effect::Render(_));
        }
        insta::assert_yaml_snapshot!(model.periods, @r###"
        ---
        - from: "2023-07-04T23:30:00Z"
          to: "2023-07-05T00:00:00Z"
          intensity:
            forecast: 142
            actual: 129
            index: moderate
          generationmix: ~
        - from: "2023-07-05T00:00:00Z"
          to: "2023-07-05T00:30:00Z"
          intensity:
            forecast: 136
            actual: 122
            index: moderate
          generationmix: ~
        "###);
    }
}
