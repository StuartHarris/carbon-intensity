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
  outcode?: string;
  adminDistrict?: string;
  data?: {
    labels: string[];
    datasets: {
      fill: boolean;
      label: string;
      data: any[];
      borderColor: string;
      backgroundColor: string;
    }[];
  };
};

const initialState: State = {
  outcode: "",
  adminDistrict: "",
};

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

    for (const { uuid, effect } of requests) {
      switch (effect.constructor) {
        case types.EffectVariantRender: {
          let bytes = view();
          let viewDeserializer = new bincode.BincodeDeserializer(bytes);
          let viewModel = types.ViewModel.deserialize(viewDeserializer);
          let data: any = undefined;
          if (viewModel.periods?.length > 0) {
            const labels = viewModel.periods.map((period) => {
              const date = new Date(period.from);
              return `${zeroPad(date.getHours(), 2)}:${zeroPad(
                date.getMinutes(),
                2
              )}`;
            });
            data = {
              labels,
              datasets: [
                {
                  fill: true,
                  label: "Forecast",
                  data: viewModel.periods.map(
                    (period) => period.intensity.forecast
                  ),
                  borderColor: "rgb(53, 162, 235)",
                  backgroundColor: "rgba(53, 162, 235, 0.5)",
                  cubicInterpolationMode: "monotone",
                  tension: 0.4,
                },
              ],
            };
          }
          setState({
            outcode: viewModel.outcode,
            adminDistrict: viewModel.admin_district,
            data,
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
        event: new types.EventVariantSwitchMode(
          new types.ModeVariantNational()
        ),
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
            {state.data && (
              <Line
                options={options}
                data={state.data}
                height="200px"
                width="200px"
              />
            )}
          </div>
          <p className="is-size-4">
            {state.adminDistrict} ({state.outcode})
          </p>
          <div className="buttons section is-centered">
            <button
              className="button is-primary is-success"
              onClick={() =>
                dispatch({
                  kind: "event",
                  event: new types.EventVariantSwitchMode(
                    new types.ModeVariantNational()
                  ),
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
                  event: new types.EventVariantSwitchMode(
                    new types.ModeVariantLocal()
                  ),
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
