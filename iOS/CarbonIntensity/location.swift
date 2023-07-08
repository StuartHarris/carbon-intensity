import CoreLocation
import SharedTypes
import SwiftUI

enum LocationError: Error {
    case generic(Error)
    case message(String)
}

func locationRequest(_: LocationRequest) -> Result<LocationResponse, LocationError> {
    let locationManager = CLLocationManager()
    locationManager.requestWhenInUseAuthorization()
    switch locationManager.authorizationStatus {
    case .restricted, .denied:
        return .failure(.message("need permission to get location"))
    default:
        var currentLoc = locationManager.location
        if currentLoc != nil {
            let coord = Coordinate(
                latitude: currentLoc!.coordinate.latitude,
                longitude: currentLoc!.coordinate.longitude
            )
            return .success(LocationResponse(location: coord))
        }
        return .failure(.message("no location available"))
    }
}
