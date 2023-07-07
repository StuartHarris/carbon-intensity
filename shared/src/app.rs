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
        national, national_mix, postcode, regional, CurrentQuery, Model,
    },
    view_model::ViewModel,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Event {
    GetNational,
    GetLocal,

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
                model.current_query = CurrentQuery::National;
                caps.time.get(Event::CurrentTime);
            }
            Event::GetLocal => {
                model.current_query = CurrentQuery::Local;
                caps.time.get(Event::CurrentTime);
            }
            Event::CurrentTime(TimeResponse(iso_time)) => {
                let last_updated = match model.current_query {
                    CurrentQuery::National => model.national.last_updated,
                    CurrentQuery::Local => model.local.last_updated,
                };
                let current_time = DateTime::parse_from_rfc3339(&iso_time)
                    .unwrap()
                    .with_timezone(&Utc);
                model.time = current_time;

                if current_time - last_updated > Duration::minutes(30) {
                    match model.current_query {
                        CurrentQuery::National => {
                            caps.http
                                .get(national::url(&model.time))
                                .expect_json()
                                .send(Event::SetNational);
                            caps.http
                                .get(national_mix::url(&model.time))
                                .expect_json()
                                .send(Event::SetNationalMix);
                        }
                        CurrentQuery::Local => {
                            caps.location.get(Event::SetLocation);
                        }
                    }
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
        location::Location, national::NationalResponse, national_mix::NationalMixResponse,
        postcode::PostcodeResponse, regional::RegionalResponse, CurrentQuery,
    };
    use assert_matches::assert_matches;
    use crux_core::{assert_effect, testing::AppTester};
    use crux_http::{
        protocol::{HttpRequest, HttpResponse},
        testing::ResponseBuilder,
    };

    #[test]
    fn local_happy_path() {
        let app = AppTester::<App, _>::default();
        let mut model = Model::default();

        // request "local" data and check we update the model and get a time request
        let update = app.update(Event::GetLocal, &mut model);
        assert_eq!(model.current_query, CurrentQuery::Local);
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
        last_updated: "2023-07-06T20:30:00Z"
        "###);

        // check that the view renders as expected
        insta::assert_yaml_snapshot!(app.view(&model), @r###"
        ---
        national_name: UK
        national: []
        local_name: "Kingston upon Thames, KT1"
        local:
          - date: "2023-07-04T23:30:00+00:00"
            forecast: 121
            actual: ~
            mix:
              gas: 17.2
              coal: 0
              biomass: 0
              nuclear: 0
              hydro: 0.2
              imports: 66.1
              other: 0
              wind: 16.5
              solar: 0
          - date: "2023-07-05T00:00:00+00:00"
            forecast: 116
            actual: ~
            mix:
              gas: 16.1
              coal: 0
              biomass: 0
              nuclear: 0
              hydro: 0.2
              imports: 65.6
              other: 0
              wind: 18
              solar: 0.1
        "###);
    }

    #[test]
    fn national_happy_path() {
        let app = AppTester::<App, _>::default();
        let mut model = Model::default();

        // request "national" data and check we update the model and get a time request
        let update = app.update(Event::GetNational, &mut model);
        assert_matches!(model.current_query, CurrentQuery::National);
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
            serde_json::from_str(include_str!("./fixtures/national.json")).unwrap();
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
                - fuel: biomass
                  perc: 5.4
                - fuel: coal
                  perc: 0
                - fuel: imports
                  perc: 7.5
                - fuel: gas
                  perc: 41.2
                - fuel: nuclear
                  perc: 24.2
                - fuel: other
                  perc: 0
                - fuel: hydro
                  perc: 0.3
                - fuel: solar
                  perc: 0
                - fuel: wind
                  perc: 21.3
            - from: "2023-07-05T00:00:00Z"
              to: "2023-07-05T00:30:00Z"
              intensity: ~
              generationmix:
                - fuel: biomass
                  perc: 5.3
                - fuel: coal
                  perc: 0
                - fuel: imports
                  perc: 7.6
                - fuel: gas
                  perc: 41.1
                - fuel: nuclear
                  perc: 24.3
                - fuel: other
                  perc: 0
                - fuel: hydro
                  perc: 0.3
                - fuel: solar
                  perc: 0
                - fuel: wind
                  perc: 21.4
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
        national:
          - date: "2023-07-04T23:30:00+00:00"
            forecast: 142
            actual: 129
            mix:
              gas: 41.2
              coal: 0
              biomass: 5.4
              nuclear: 24.2
              hydro: 0.3
              imports: 7.5
              other: 0
              wind: 21.3
              solar: 0
          - date: "2023-07-05T00:00:00+00:00"
            forecast: 136
            actual: 122
            mix:
              gas: 41.1
              coal: 0
              biomass: 5.3
              nuclear: 24.3
              hydro: 0.3
              imports: 7.6
              other: 0
              wind: 21.4
              solar: 0
        local_name: Local
        local: []
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
