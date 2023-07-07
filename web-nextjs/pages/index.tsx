import type { NextPage } from "next";
import Head from "next/head";
import { useEffect, useState } from "react";
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Filler,
  Legend,
} from "chart.js";
import { Line } from "react-chartjs-2";

import init_core, {
  process_event,
  handle_response,
  view,
} from "../shared/core";
import * as types from "shared_types/types/shared_types";
import * as bincode from "shared_types/bincode/mod";
import { httpRequest } from "./httpRequest";
import { locationRequest } from "./locationRequest";

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Filler,
  Legend
);

const zeroPad = (num: number, places: number) =>
  String(num).padStart(places, "0");

export const options = {
  responsive: true,
  maintainAspectRatio: false,
  scales: {
    y: {
      min: 0,
      max: 600,
      title: {
        display: true,
        text: "gCO2/kWh",
      },
    },
    x: {
      title: {
        display: true,
        text: "Time",
      },
    },
  },
  plugins: {
    legend: {
      position: "top" as const,
    },
    title: {
      display: true,
      text: "Carbon Intensity",
    },
  },
};

interface Event {
  kind: "event";
  event: types.Event;
}

interface Response {
  kind: "response";
  uuid: number[];
  outcome: types.HttpResponse | types.LocationResponse | types.TimeResponse;
}

type State = {
  national_name?: string;
  local_name?: string;
  intensity_data?: any;
  mix_data?: any;
  mix_options?: any;
};

const initialState: State = {};

function deserializeRequests(bytes: Uint8Array) {
  let deserializer = new bincode.BincodeDeserializer(bytes);

  const len = deserializer.deserializeLen();

  let requests: types.Request[] = [];

  for (let i = 0; i < len; i++) {
    const request = types.Request.deserialize(deserializer);
    requests.push(request);
  }

  return requests;
}
const Home: NextPage = () => {
  const [state, setState] = useState(initialState);

  const dispatch = (action: Event) => {
    const serializer = new bincode.BincodeSerializer();
    action.event.serialize(serializer);
    const requests = process_event(serializer.getBytes());
    handleRequests(requests);
  };

  const respond = (action: Response) => {
    const serializer = new bincode.BincodeSerializer();
    action.outcome.serialize(serializer);
    const moreRequests = handle_response(
      new Uint8Array(action.uuid),
      serializer.getBytes()
    );
    handleRequests(moreRequests);
  };

  const handleRequests = async (bytes: Uint8Array) => {
    let requests = deserializeRequests(bytes);
    console.log("requests", requests);

    for (const { uuid, effect } of requests) {
      switch (effect.constructor) {
        case types.EffectVariantRender: {
          let bytes = view();
          let viewDeserializer = new bincode.BincodeDeserializer(bytes);
          let viewModel = types.ViewModel.deserialize(viewDeserializer);

          const labels = (viewModel.national || viewModel.local || []).map(
            (point) => {
              const date = new Date(point.date);
              return `${zeroPad(date.getHours(), 2)}:${zeroPad(
                date.getMinutes(),
                2
              )}`;
            }
          );
          let intensity_data = {
            labels,
            datasets: [
              viewModel.national
                ? {
                    fill: true,
                    label: `${viewModel.national_name} average`,
                    data: viewModel.national.map((point) => point.forecast),
                    borderColor: "rgb(53, 162, 235)",
                    backgroundColor: "rgba(53, 162, 235, 0.5)",
                    cubicInterpolationMode: "monotone",
                    tension: 0.4,
                  }
                : undefined,
              viewModel.local
                ? {
                    fill: true,
                    label: viewModel.local_name,
                    data: viewModel.local.map((point) => point.forecast),
                    borderColor: "rgb(255,	205,	86)",
                    backgroundColor: "rgb(255,	205,	86, 0.5)",
                    cubicInterpolationMode: "monotone",
                    tension: 0.4,
                  }
                : {},
            ],
          };
          let mix_options = {
            ...options,
            interaction: {
              mode: "nearest",
              axis: "x",
              intersect: false,
            },
            scales: {
              y: {
                stacked: true,
                min: 0,
                max: 100,
              },
            },
          };
          let points = Object.fromEntries(
            [
              "gas",
              "coal",
              "biomass",
              "nuclear",
              "hydro",
              "imports",
              "other",
              "wind",
              "solar",
            ].map((type) => [
              type,
              viewModel.national.map((point) => point.mix[type]),
            ])
          );
          let mix_data = {
            labels,
            datasets: [
              {
                fill: "origin",
                label: `${viewModel.national_name} Gas`,
                data: points["gas"],
                borderColor: "rgb(112, 48, 160)",
                backgroundColor: "rgba(112, 48, 160, 0.5)",
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
              {
                fill: "-1",
                label: `${viewModel.national_name} Coal`,
                data: points["coal"],
                borderColor: "rgb(44, 42, 40)",
                backgroundColor: "rgba(44, 42, 40, 0.5)",
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
              {
                fill: "-1",
                label: `${viewModel.national_name} Biomass`,
                data: points["biomass"],
                borderColor: "rgb(239, 133, 52)",
                backgroundColor: "rgba(239, 133, 52, 0.5)",
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
              {
                fill: "-1",
                label: `${viewModel.national_name} Nuclear`,
                data: points["nuclear"],
                borderColor: "rgb(75, 138, 68)",
                backgroundColor: "rgba(75, 138, 68, 0.5)",
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
              {
                fill: "-1",
                label: `${viewModel.national_name} Hydro`,
                data: points["hydro"],
                borderColor: "rgb(53, 162, 235)",
                backgroundColor: "rgba(53, 162, 235, 0.5)",
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
              {
                fill: "-1",
                label: `${viewModel.national_name} Imports`,
                data: points["imports"],
                borderColor: "rgb(235, 85, 110)",
                backgroundColor: "rgba(235, 85, 110, 0.5)",
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
              {
                fill: "-1",
                label: `${viewModel.national_name} Other`,
                data: points["other"],
                borderColor: "rgb(172, 221, 170)",
                backgroundColor: "rgba(172, 221, 170, 0.5)",
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
              {
                fill: "-1",
                label: `${viewModel.national_name} Wind`,
                data: points["wind"],
                borderColor: "rgb(79, 171, 213)",
                backgroundColor: "rgba(79, 171, 213, 0.5)",
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
              {
                fill: "-1",
                label: `${viewModel.national_name} Solar`,
                data: points["solar"],
                borderColor: "rgb(247, 209, 71)",
                backgroundColor: "rgba(247, 209, 71, 0.5)",
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
            ],
          };

          setState({
            local_name: viewModel.local_name,
            national_name: viewModel.national_name,
            intensity_data,
            mix_data,
            mix_options,
          });

          break;
        }

        case types.EffectVariantTime: {
          const outcome = new types.TimeResponse(new Date().toISOString());
          respond({ kind: "response", uuid, outcome });
          break;
        }

        case types.EffectVariantHttp: {
          const request = (effect as types.EffectVariantHttp).value;
          const outcome = await httpRequest(request);
          respond({ kind: "response", uuid, outcome });
          break;
        }

        case types.EffectVariantGetLocation: {
          const request = (effect as types.EffectVariantGetLocation).value;

          const outcome = await locationRequest(request);
          respond({ kind: "response", uuid, outcome });
          break;
        }

        default:
      }
    }
  };

  useEffect(() => {
    async function loadCore() {
      await init_core();

      // Initial event
      dispatch({
        kind: "event",
        event: new types.EventVariantGetNational(),
      });
    }

    loadCore();
  }, []);

  return (
    <>
      <Head>
        <title>Next.js Carbon Intensity</title>
      </Head>

      <main>
        <section className="box container has-text-centered m-5">
          <div
            style={{
              height: "60vh",
              position: "relative",
              marginBottom: "1%",
              padding: "1%",
            }}
          >
            {state.intensity_data && (
              <Line
                options={options}
                data={state.intensity_data}
                height="200px"
                width="200px"
              />
            )}
          </div>
          <div
            style={{
              height: "60vh",
              position: "relative",
              marginBottom: "1%",
              padding: "1%",
            }}
          >
            {state.mix_data && (
              <Line
                options={state.mix_options}
                data={state.mix_data}
                height="200px"
                width="200px"
              />
            )}
          </div>
          <div className="buttons section is-centered">
            <button
              className="button is-primary is-success"
              onClick={() =>
                dispatch({
                  kind: "event",
                  event: new types.EventVariantGetNational(),
                })
              }
            >
              {"National"}
            </button>
            <button
              className="button is-primary is-success"
              onClick={() =>
                dispatch({
                  kind: "event",
                  event: new types.EventVariantGetLocal(),
                })
              }
            >
              {"Local"}
            </button>
          </div>
        </section>
      </main>
    </>
  );
};

export default Home;
