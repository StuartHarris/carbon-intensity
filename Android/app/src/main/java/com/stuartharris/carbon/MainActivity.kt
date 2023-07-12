package com.stuartharris.carbon

import android.app.Application
import android.location.Location
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.rememberMultiplePermissionsState
import com.google.android.gms.location.FusedLocationProviderClient
import com.google.android.gms.location.LocationServices
import com.stuartharris.carbon.chart.IntensityChart
import com.stuartharris.carbon.chart.MixChart
import com.stuartharris.carbon.shared.handleResponse
import com.stuartharris.carbon.shared.processEvent
import com.stuartharris.carbon.shared.view
import com.stuartharris.carbon.shared_types.Coordinate
import com.stuartharris.carbon.shared_types.Effect
import com.stuartharris.carbon.shared_types.HttpResponse
import com.stuartharris.carbon.shared_types.LocationResponse
import com.stuartharris.carbon.shared_types.Requests
import com.stuartharris.carbon.shared_types.TimeResponse
import com.stuartharris.carbon.ui.theme.CarbonIntensityTheme
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.AndroidEntryPoint
import dagger.hilt.android.HiltAndroidApp
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.components.SingletonComponent
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import kotlinx.coroutines.launch
import java.time.ZoneOffset
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import java.util.Optional
import javax.inject.Inject
import javax.inject.Singleton
import com.stuartharris.carbon.shared_types.Event as Evt
import com.stuartharris.carbon.shared_types.Request as Req
import com.stuartharris.carbon.shared_types.ViewModel as MyViewModel

@HiltAndroidApp
class CarbonIntensityApplication : Application()

@Module
@InstallIn(SingletonComponent::class)
object AppModule {

    @Provides
    @Singleton
    fun providesFusedLocationProviderClient(
        application: Application
    ): FusedLocationProviderClient = LocationServices.getFusedLocationProviderClient(application)

    @Provides
    @Singleton
    fun providesLocationTracker(
        fusedLocationProviderClient: FusedLocationProviderClient, application: Application
    ): LocationTracker = DefaultLocationTracker(
        fusedLocationProviderClient = fusedLocationProviderClient, application = application
    )
}

@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            CarbonIntensityTheme {
                // A surface container using the 'background' color from the theme
                Surface(
                    modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background
                ) {
                    View()
                }
            }
        }
    }
}

sealed class Outcome {
    data class Http(val res: HttpResponse) : Outcome()
    data class Location(val res: LocationResponse) : Outcome()
    data class Time(val res: TimeResponse) : Outcome()
}

sealed class CoreMessage {
    data class Event(val event: Evt) : CoreMessage()
    data class Response(val uuid: List<Byte>, val outcome: Outcome) : CoreMessage()
}

@HiltViewModel
class Model @Inject constructor(
    private val locationTracker: LocationTracker
) : ViewModel() {
    var view: MyViewModel by mutableStateOf(
        MyViewModel(
            "", emptyList(), emptyList(), "", emptyList(), emptyList()
        )
    )
        private set

    private val httpClient = HttpClient(CIO)

    var currentLocation by mutableStateOf<Location?>(null)

//    var pointsData: List<Point> = emptyList()

    fun getCurrentLocation() {
        viewModelScope.launch {
            currentLocation = locationTracker.getCurrentLocation()
        }
    }

    init {
        viewModelScope.launch {
            update(CoreMessage.Event(Evt.GetNational()))
        }
    }

    suspend fun update(msg: CoreMessage) {
        val requests: List<Req> = when (msg) {
            is CoreMessage.Event -> Requests.bincodeDeserialize(
                processEvent(msg.event.bincodeSerialize())
            )

            is CoreMessage.Response -> Requests.bincodeDeserialize(
                handleResponse(
                    msg.uuid.toByteArray(), when (msg.outcome) {
                        is Outcome.Http -> msg.outcome.res.bincodeSerialize()
                        is Outcome.Location -> msg.outcome.res.bincodeSerialize()
                        is Outcome.Time -> msg.outcome.res.bincodeSerialize()
                    }
                )
            )
        }

        for (req in requests) when (val effect = req.effect) {
            is Effect.Render -> {
                view = MyViewModel.bincodeDeserialize(view())
            }

            is Effect.Http -> {
                val response = http(
                    httpClient, effect.value
                )
                update(CoreMessage.Response(req.uuid, Outcome.Http(response)))
            }

            is Effect.GetLocation -> {
                val response = LocationResponse(
                    Optional.of(
                        Coordinate(currentLocation?.latitude, currentLocation?.longitude)
                    )
                )
                update(CoreMessage.Response(req.uuid, Outcome.Location(response)))
            }

            is Effect.Time -> {
                val isoTime =
                    ZonedDateTime.now(ZoneOffset.UTC).format(DateTimeFormatter.ISO_INSTANT)

                update(CoreMessage.Response(req.uuid, Outcome.Time(TimeResponse(isoTime))))
            }
        }
    }
}

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun View(model: Model = viewModel()) {
    val coroutineScope = rememberCoroutineScope()

    val locationPermissions = rememberMultiplePermissionsState(
        permissions = listOf(
            android.Manifest.permission.ACCESS_COARSE_LOCATION,
            android.Manifest.permission.ACCESS_FINE_LOCATION
        )
    )

    LaunchedEffect(key1 = locationPermissions.allPermissionsGranted) {
        if (locationPermissions.allPermissionsGranted) {
            model.getCurrentLocation()
        }
    }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier
            .fillMaxSize()
            .padding(10.dp),
    ) {
        Text(text = "Carbon Intensity", fontSize = 30.sp, modifier = Modifier.padding(10.dp))
        Text(
            text = model.view.local_name, modifier = Modifier.padding(10.dp)
        )
        Box(
            modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center
        ) {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Column {
                    IntensityChart(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(300.dp)
                            .padding(vertical = 4.dp),
                        points = model.view.national_intensity,
                    )
                    MixChart(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(300.dp)
                            .padding(vertical = 4.dp),
                        points = model.view.national_mix,
                    )
                }
                Row {
                    AnimatedContent(
                        targetState = locationPermissions.allPermissionsGranted
                    ) { areGranted ->
                        if (!areGranted) {
                            Column {
                                Text(text = "We need location permissions for this application.")
                                Button(onClick = { locationPermissions.launchMultiplePermissionRequest() }) {
                                    Text(text = "Accept")
                                }
                            }
                        }
                    }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    Button(
                        onClick = {
                            coroutineScope.launch { model.update(CoreMessage.Event(Evt.GetNational())) }
                        }, colors = ButtonDefaults.buttonColors(
                            containerColor = Color.hsl(44F, 1F, 0.77F)
                        )
                    ) { Text(text = "National", color = Color.DarkGray) }
                    Button(
                        onClick = {
                            coroutineScope.launch { model.update(CoreMessage.Event(Evt.GetLocal())) }
                        }, colors = ButtonDefaults.buttonColors(
                            containerColor = Color.hsl(348F, 0.86F, 0.61F)
                        )
                    ) { Text(text = "Local", color = Color.White) }
                }
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun Preview() {
    CarbonIntensityTheme {
        View()
    }
}