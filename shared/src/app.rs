use chrono::{DateTime, Duration, Utc};
use crux_core::render::Render;
use crux_http::Http;
use crux_macros::Effect;
use serde::{Deserialize, Serialize};

use crate::{
    capabilities::{
        location::{GetLocation, LocationResponse},
        time::{Time, TimeResponse},
    },
    model::{
        location::{Coordinate, Location},
        national_intensity, national_mix, postcode, regional, Model,
    },
    view_model::ViewModel,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Event {
    GetNational,
    GetLocal,

    // events local to the core
    #[serde(skip)]
    SetTimeLocal(TimeResponse),
    #[serde(skip)]
    SetTimeNational(TimeResponse),
    #[serde(skip)]
    SetLocation(LocationResponse),
    #[serde(skip)]
    SetPostcode(crux_http::Result<crux_http::Response<postcode::PostcodeResponse>>),
    #[serde(skip)]
    SetRegional(crux_http::Result<crux_http::Response<regional::RegionalResponse>>),
    #[serde(skip)]
    SetNational(crux_http::Result<crux_http::Response<national_intensity::NationalResponse>>),
    #[serde(skip)]
    SetNationalMix(crux_http::Result<crux_http::Response<national_mix::NationalMixResponse>>),
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
            Event::GetNational => {
                caps.time.get(Event::SetTimeNational);
            }
            Event::GetLocal => {
                caps.time.get(Event::SetTimeLocal);
            }
            Event::SetTimeLocal(TimeResponse(iso_time)) => {
                let current_time = DateTime::parse_from_rfc3339(&iso_time)
                    .unwrap()
                    .with_timezone(&Utc);
                model.time = current_time;

                if current_time - model.local.last_updated > Duration::minutes(30) {
                    caps.location.get(Event::SetLocation);
                } else {
                    caps.render.render();
                }
            }
            Event::SetTimeNational(TimeResponse(iso_time)) => {
                let current_time = DateTime::parse_from_rfc3339(&iso_time)
                    .unwrap()
                    .with_timezone(&Utc);
                model.time = current_time;

                if current_time - model.national.last_updated > Duration::minutes(30) {
                    caps.http
                        .get(national_intensity::url(&model.time))
                        .expect_json()
                        .send(Event::SetNational);
                    caps.http
                        .get(national_mix::url(&model.time))
                        .expect_json()
                        .send(Event::SetNationalMix);
                } else {
                    caps.render.render();
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
                caps.render.render();
            }
            Event::SetLocation(LocationResponse { location: None }) => {}
            Event::SetPostcode(Ok(mut postcode)) => {
                let postcode = postcode.take_body().unwrap();
                let postcode = postcode.result[0].clone(); // TODO error handling
                let outcode = postcode.outcode;
                let url = regional::url(&model.time, &outcode);

                model.local.scope.location = Some(Location {
                    coordinate: Coordinate {
                        latitude: postcode.latitude,
                        longitude: postcode.longitude,
                    },
                    outcode,
                    admin_district: postcode.admin_district.clone(),
                });

                caps.http.get(url).expect_json().send(Event::SetRegional);
                caps.render.render();
            }
            Event::SetPostcode(Err(_)) => {}
            Event::SetRegional(Ok(mut response)) => {
                let regional = response.take_body().unwrap();
                model.local.periods = regional.data.data.clone();
                model.local.last_updated = model.time;

                caps.render.render();
            }
            Event::SetRegional(Err(_)) => {}
            Event::SetNational(Ok(mut response)) => {
                let national = response.take_body().unwrap();
                model.national.periods = national.data.clone();
                model.national.last_updated = model.time;

                caps.render.render();
            }
            Event::SetNational(Err(_)) => {}
            Event::SetNationalMix(Ok(mut response)) => {
                let national = response.take_body().unwrap();
                model.national.scope.generation_mix = national.data.clone();
                model.national.last_updated = model.time;

                caps.render.render();
            }
            Event::SetNationalMix(Err(_)) => {}
        };
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        model.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        location::Location, national_intensity::NationalResponse,
        national_mix::NationalMixResponse, postcode::PostcodeResponse, regional::RegionalResponse,
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

        // request "local" data and check we get a time request
        let update = app.update(Event::GetLocal, &mut model);
        let requests = &mut update.into_effects().filter_map(Effect::into_time);

        // resolve the time request with a simulated time response
        let mut request = requests.next().unwrap();
        let response = TimeResponse("2023-07-06T20:30:00Z".to_string());
        let update = app.resolve(&mut request, response.clone()).unwrap();

        // check this raises the correct set time event
        let set_time_event = Event::SetTimeLocal(response.clone());
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
            location: Some(Coordinate {
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
        assert_eq!(
            model.local.scope.location.clone().unwrap(),
            Location {
                coordinate: Coordinate {
                    latitude: 51.40306,
                    longitude: -0.298333,
                },
                outcode: "KT1".to_string(),
                admin_district: "Kingston upon Thames".to_string(),
            }
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
        insta::assert_yaml_snapshot!(model.local, @r###"
        ---
        scope:
          location:
            coordinate:
              latitude: 51.40306
              longitude: -0.298333
            outcode: KT1
            admin_district: Kingston upon Thames
        periods:
          - from: "2023-07-04T23:30:00Z"
            to: "2023-07-05T00:00:00Z"
            intensity:
              forecast: 121
              actual: ~
              index: moderate
            generationmix:
              - fuel: solar
                perc: 0
              - fuel: wind
                perc: 16.5
              - fuel: other
                perc: 0
          - from: "2023-07-05T00:00:00Z"
            to: "2023-07-05T00:30:00Z"
            intensity:
              forecast: 116
              actual: ~
              index: low
            generationmix:
              - fuel: gas
                perc: 16.1
              - fuel: nuclear
                perc: 0
        last_updated: "2023-07-06T20:30:00Z"
        "###);

        // check that the view renders as expected
        insta::assert_yaml_snapshot!(app.view(&model), @r###"
        ---
        national_name: UK
        national_intensity: []
        national_mix: []
        local_name: "Kingston upon Thames, KT1"
        local_intensity:
          - date: "2023-07-04T23:30:00+00:00"
            forecast: 121
            actual: ~
          - date: "2023-07-05T00:00:00+00:00"
            forecast: 116
            actual: ~
        local_mix:
          - date: "2023-07-05T00:00:00+00:00"
            fuel: Gas
            perc: 16.1
          - date: "2023-07-05T00:00:00+00:00"
            fuel: Nuclear
            perc: 0
          - date: "2023-07-04T23:30:00+00:00"
            fuel: Wind
            perc: 16.5
          - date: "2023-07-04T23:30:00+00:00"
            fuel: Solar
            perc: 0
        "###);
    }

    #[test]
    fn national_happy_path() {
        let app = AppTester::<App, _>::default();
        let mut model = Model::default();

        // request "national" data and check we get a time request
        let update = app.update(Event::GetNational, &mut model);
        let requests = &mut update.into_effects().filter_map(Effect::into_time);

        // resolve the time request with a simulated time response
        let mut request = requests.next().unwrap();
        let response = TimeResponse("2023-07-06T20:30:00Z".to_string());
        let update = app.resolve(&mut request, response.clone()).unwrap();

        // check this raises the correct set time event
        let set_time_event = Event::SetTimeNational(response.clone());
        let actual = &update.events;
        let expected = &vec![set_time_event.clone()];
        assert_eq!(actual, expected);

        // update the app and check it updates the model
        let update = app.update(set_time_event, &mut model);
        assert_eq!(
            model.time,
            DateTime::parse_from_rfc3339("2023-07-06T20:30:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );

        // we should get 2 http requests, one for intensity and one for generation mix
        let requests = &mut update.into_effects().filter_map(Effect::into_http);

        // get the first http request
        let mut request = requests.next().unwrap();

        // check the intensity request has the expected url
        let actual = &request.operation;
        let expected = &HttpRequest::get(
            "https://api.carbonintensity.org.uk/intensity/2023-07-06T20:30Z/fw24h",
        )
        .build();
        assert_eq!(actual, expected);

        // resolve a simulated intensity response
        let simulated_response: NationalResponse =
            serde_json::from_str(include_str!("./fixtures/national_intensity.json")).unwrap();
        let response = HttpResponse::status(200).json(&simulated_response).build();
        let update = app.resolve(&mut request, response).unwrap();

        // check the intensity response raises a SetNational event
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
        insta::assert_yaml_snapshot!(model.national, @r###"
        ---
        scope:
          generation_mix: []
        periods:
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
        last_updated: "2023-07-06T20:30:00Z"
        "###);

        // get the second http request
        let mut request = requests.next().unwrap();

        // check the generation mix request has the expected url
        let actual = &request.operation;
        let expected = &HttpRequest::get(
            "https://api.carbonintensity.org.uk/generation/2023-07-06T20:30Z/2023-07-07T20:30Z",
        )
        .build();
        assert_eq!(actual, expected);

        // resolve a simulated generation response
        let simulated_response: NationalMixResponse =
            serde_json::from_str(include_str!("./fixtures/national_mix.json")).unwrap();
        let response = HttpResponse::status(200).json(&simulated_response).build();
        let update = app.resolve(&mut request, response).unwrap();

        // check the intensity response raises a SetNational event
        let set_national_mix_event = Event::SetNationalMix(Ok(ResponseBuilder::ok()
            .body(simulated_response)
            .build()
            .clone()));
        let actual = &update.events;
        let expected = &vec![set_national_mix_event.clone()];
        assert_eq!(actual, expected);

        // check that the SetNationalMix event updates the model and renders
        for event in update.events {
            let update = app.update(event, &mut model);
            assert_effect!(update, Effect::Render(_));
        }
        insta::assert_yaml_snapshot!(model.national, @r###"
        ---
        scope:
          generation_mix:
            - from: "2023-07-04T23:30:00Z"
              to: "2023-07-05T00:00:00Z"
              intensity: ~
              generationmix:
                - fuel: solar
                  perc: 0
                - fuel: wind
                  perc: 21.3
                - fuel: other
                  perc: 0
            - from: "2023-07-05T00:00:00Z"
              to: "2023-07-05T00:30:00Z"
              intensity: ~
              generationmix:
                - fuel: gas
                  perc: 41.1
                - fuel: nuclear
                  perc: 24.3
        periods:
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
        last_updated: "2023-07-06T20:30:00Z"
        "###);

        // check the view renders as expected
        insta::assert_yaml_snapshot!(app.view(&model), @r###"
        ---
        national_name: UK
        national_intensity:
          - date: "2023-07-04T23:30:00+00:00"
            forecast: 142
            actual: 129
          - date: "2023-07-05T00:00:00+00:00"
            forecast: 136
            actual: 122
        national_mix:
          - date: "2023-07-05T00:00:00+00:00"
            fuel: Gas
            perc: 41.1
          - date: "2023-07-05T00:00:00+00:00"
            fuel: Nuclear
            perc: 24.3
          - date: "2023-07-04T23:30:00+00:00"
            fuel: Wind
            perc: 21.3
          - date: "2023-07-04T23:30:00+00:00"
            fuel: Solar
            perc: 0
        local_name: Local
        local_intensity: []
        local_mix: []
        "###);
    }

    #[test]
    fn do_not_get_local_if_less_than_30_mins_has_elapsed() {
        let app = AppTester::<App, _>::default();
        let mut model = Model::default();
        model.local.last_updated = DateTime::parse_from_rfc3339("2023-07-06T20:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        // request "local" data and get a time request
        let update = app.update(Event::GetLocal, &mut model);
        let mut request = &mut update.into_effects().find_map(Effect::into_time).unwrap();

        // resolve the time request with a simulated time response
        let response = TimeResponse("2023-07-06T20:59:00Z".to_string());
        let update = app.resolve(&mut request, response).unwrap();

        // update the app and check we only get a render effect
        for event in update.events {
            let update = app.update(event, &mut model);
            assert_effect!(update, Effect::Render(_));
            assert_eq!(update.effects.len(), 1);
        }
    }

    #[test]
    fn get_national_if_more_than_30_mins_has_elapsed() {
        let app = AppTester::<App, _>::default();
        let mut model = Model::default();
        model.national.last_updated = DateTime::parse_from_rfc3339("2023-07-06T20:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        // request "national" data and get a time request
        let update = app.update(Event::GetNational, &mut model);
        let mut request = &mut update.into_effects().find_map(Effect::into_time).unwrap();

        // resolve the time request with a simulated time response
        let response = TimeResponse("2023-07-06T21:01:00Z".to_string());
        let update = app.resolve(&mut request, response).unwrap();

        // update the app and check the resulting request has the expected url
        let update = app.update(update.events[0].clone(), &mut model);
        let request = &mut update.into_effects().find_map(Effect::into_http).unwrap();
        let actual = &request.operation;
        let expected = &HttpRequest::get(
            "https://api.carbonintensity.org.uk/intensity/2023-07-06T21:01Z/fw24h",
        )
        .build();
        assert_eq!(actual, expected);
    }
}
