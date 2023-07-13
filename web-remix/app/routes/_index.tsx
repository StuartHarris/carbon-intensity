import type { V2_MetaFunction } from "@remix-run/node";
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

import { process_event, handle_response, view } from "shared/shared";
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

const mixCategories: Record<string, number[]> = {
  Coal: [44, 42, 40],
  Gas: [112, 48, 160],
  Imports: [235, 85, 110],
  Biomass: [239, 133, 52],
  Nuclear: [75, 138, 68],
  Hydro: [57, 108, 203],
  Wind: [79, 171, 213],
  Solar: [247, 209, 71],
};

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
const mix_options = {
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

export const meta: V2_MetaFunction = () => {
  return [
    { title: "Carbon Intensity" },
    {
      name: "description",
      content: "Carbon Intensity and Generation Mix forecasts",
    },
  ];
};

export default function Index() {
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
          const bytes = view();
          const viewDeserializer = new bincode.BincodeDeserializer(bytes);
          const viewModel = types.ViewModel.deserialize(viewDeserializer);

          const labels = (
            viewModel.national_intensity ||
            viewModel.local_intensity ||
            []
          ).map((point) => {
            const date = new Date(point.date);
            return `${zeroPad(date.getHours(), 2)}:${zeroPad(
              date.getMinutes(),
              2
            )}`;
          });
          const intensity_data = {
            labels,
            datasets: [
              {
                fill: true,
                label: `${viewModel.national_name} average`,
                data: viewModel.national_intensity.map(
                  (point) => point.forecast
                ),
                borderColor: "rgb(53, 162, 235)",
                backgroundColor: "rgba(53, 162, 235, 0.5)",
                pointStyle: false,
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
              {
                fill: true,
                label: viewModel.local_name,
                data: viewModel.local_intensity.map((point) => point.forecast),
                borderColor: "rgb(255, 205, 86)",
                backgroundColor: "rgb(255, 205, 86, 0.5)",
                pointStyle: false,
                cubicInterpolationMode: "monotone",
                tension: 0.4,
              },
            ],
          };
          const mixPoints = viewModel.national_mix.reduce(function (
            acc,
            point
          ) {
            acc[point.fuel] = acc[point.fuel] || [];
            acc[point.fuel].push(point);
            return acc;
          },
          {} as Record<string, types.GenerationMixPoint[]>);
          let datasets = Object.entries(mixPoints).map(([label, value]) => {
            const color = mixCategories[label];
            return {
              fill: label === "Coal" ? "origin" : "-1",
              label,
              data: value.map((point) => point.perc),
              borderColor: `rgb(${color[0]}, ${color[1]}, ${color[2]})`,
              backgroundColor: `rgb(${color[0]}, ${color[1]}, ${color[2]}, 0.5)`,
              pointStyle: false,
              cubicInterpolationMode: "monotone",
              tension: 0.4,
            };
          });
          const mix_data = { labels, datasets };

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
    // Initial event
    dispatch({
      kind: "event",
      event: new types.EventVariantGetNational(),
    });
  }, []);

  return (
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
  );
}
