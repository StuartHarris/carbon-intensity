// see https://medium.com/@daniel.atitienei/get-current-user-location-in-jetpack-compose-using-clean-architecture-android-6683abca66c9
//
// to send location to emulator: `adb emu geo fix -0.298302 51.403366`
package com.stuartharris.carbon

import android.app.Application
import android.content.Context
import android.content.pm.PackageManager
import android.location.LocationManager
import androidx.core.content.ContextCompat
import com.google.android.gms.location.FusedLocationProviderClient
import com.stuartharris.carbon.shared_types.Coordinate
import com.stuartharris.carbon.shared_types.LocationResponse
import kotlinx.coroutines.suspendCancellableCoroutine
import java.util.Optional

interface LocationTracker {
    suspend fun getCurrentLocation(): LocationResponse?
}

class DefaultLocationTracker(
    private val fusedLocationProviderClient: FusedLocationProviderClient,
    private val application: Application
) : LocationTracker {
    override suspend fun getCurrentLocation(): LocationResponse? {
        val hasAccessFineLocationPermission = ContextCompat.checkSelfPermission(
            application, android.Manifest.permission.ACCESS_FINE_LOCATION
        ) == PackageManager.PERMISSION_GRANTED

        val hasAccessCoarseLocationPermission = ContextCompat.checkSelfPermission(
            application, android.Manifest.permission.ACCESS_COARSE_LOCATION
        ) == PackageManager.PERMISSION_GRANTED


        val locationManager = application.getSystemService(
            Context.LOCATION_SERVICE
        ) as LocationManager

        val isGpsEnabled =
            locationManager.isProviderEnabled(LocationManager.NETWORK_PROVIDER) || locationManager.isProviderEnabled(
                LocationManager.GPS_PROVIDER
            )

        if (!isGpsEnabled && !(hasAccessCoarseLocationPermission || hasAccessFineLocationPermission)) {
            return null
        }

        return suspendCancellableCoroutine { cont ->
            fusedLocationProviderClient.lastLocation.apply {
                if (isComplete) {
                    if (isSuccessful) {
                        cont.resume(
                            LocationResponse(
                                Optional.of(
                                    Coordinate(result?.latitude, result?.longitude)
                                )
                            )
                        ) {} // Resume coroutine with location result
                    } else {
                        cont.resume(null) {} // Resume coroutine with null location result
                    }
                    return@suspendCancellableCoroutine
                }
                addOnSuccessListener {
                    cont.resume(
                        LocationResponse(
                            Optional.of(
                                Coordinate(it?.latitude, it?.longitude)
                            )
                        )
                    ) {}  // Resume coroutine with location result
                }
                addOnFailureListener {
                    cont.resume(null) {} // Resume coroutine with null location result
                }
                addOnCanceledListener {
                    cont.cancel() // Cancel the coroutine
                }
            }
        }
    }
}

