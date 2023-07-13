import {
  LocationRequest,
  LocationResponse,
  Coordinate,
} from "shared_types/types/shared_types";

export async function locationRequest(
  _locationRequest: LocationRequest
): Promise<LocationResponse> {
  const location = await getLocation();
  const locationResponse = new LocationResponse(location);
  return locationResponse;
}

function getLocation(): Promise<Coordinate> {
  return new Promise((resolve, reject) => {
    if (navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(
        (position: GeolocationPosition) => {
          if (position) {
            resolve(
              new Coordinate(
                position.coords.latitude,
                position.coords.longitude
              )
            );
          } else {
            reject(GeolocationPositionError.POSITION_UNAVAILABLE);
          }
        },
        (error: GeolocationPositionError) => reject(error)
      );
    } else {
      alert("Geolocation is not supported by this browser.");
    }
  });
}
