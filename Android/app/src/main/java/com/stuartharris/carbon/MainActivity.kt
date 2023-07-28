package com.stuartharris.carbon

import android.app.Application
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
import com.stuartharris.carbon.shared_types.Effect
import com.stuartharris.carbon.shared_types.Event
import com.stuartharris.carbon.shared_types.Mode
import com.stuartharris.carbon.shared_types.Request
import com.stuartharris.carbon.shared_types.Requests
import com.stuartharris.carbon.shared_types.TimeResponse
import com.stuartharris.carbon.shared_types.ViewModel
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
import javax.inject.Inject
import javax.inject.Singleton

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

@HiltViewModel
class Core @Inject constructor(
    private val locationTracker: LocationTracker
) : androidx.lifecycle.ViewModel() {
    var view: ViewModel by mutableStateOf(
        ViewModel(
            Mode.National(), "", emptyList(), emptyList(), "", emptyList(), emptyList()
        )
    )
        private set

    private val httpClient = HttpClient(CIO)

    init {
        viewModelScope.launch {
            update(Event.GetNational())
        }
    }

    suspend fun update(event: Event) {
        val effects = processEvent(event.bincodeSerialize())
        processEffects(effects)
    }

    private suspend fun processEffects(effects: ByteArray) {
        val requests = Requests.bincodeDeserialize(effects)
        for (request in requests) {
            processRequest(request)
        }
    }

    private suspend fun processRequest(request: Request) {
        when (val effect = request.effect) {
            is Effect.Render -> {
                this.view = ViewModel.bincodeDeserialize(view())
            }

            is Effect.Http -> {
                val response = http(httpClient, effect.value)

                val effects =
                    handleResponse(request.uuid.toByteArray(), response.bincodeSerialize())

                processEffects(effects)
            }


            is Effect.GetLocation -> {
                val response = locationTracker.getCurrentLocation()
                if (response != null) {
                    val effects =
                        handleResponse(request.uuid.toByteArray(), response.bincodeSerialize())

                    processEffects(effects)
                }
            }

            is Effect.Time -> {
                val response =
                    TimeResponse(
                        ZonedDateTime.now(ZoneOffset.UTC).format(DateTimeFormatter.ISO_INSTANT)
                    )

                val effects =
                    handleResponse(request.uuid.toByteArray(), response.bincodeSerialize())

                processEffects(effects)
            }
        }
    }
}

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun View(core: Core = viewModel()) {
    val coroutineScope = rememberCoroutineScope()

    val locationPermissions = rememberMultiplePermissionsState(
        permissions = listOf(
            android.Manifest.permission.ACCESS_COARSE_LOCATION,
            android.Manifest.permission.ACCESS_FINE_LOCATION
        )
    )

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier
            .fillMaxSize()
            .padding(10.dp),
    ) {
        Text(text = "Carbon Intensity", fontSize = 30.sp, modifier = Modifier.padding(10.dp))
        Text(
            text = if (core.view.mode == Mode.Local()) core.view.local_name else core.view.national_name,
            modifier = Modifier.padding(10.dp)
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
                        points = if (core.view.mode == Mode.Local()) core.view.local_intensity else core.view.national_intensity,
                    )
                    MixChart(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(300.dp)
                            .padding(vertical = 12.dp),
                        points = if (core.view.mode == Mode.Local()) core.view.local_mix else core.view.national_mix,
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
                            coroutineScope.launch { core.update(Event.GetNational()) }
                        }, colors = ButtonDefaults.buttonColors(
                            containerColor = Color.hsl(44F, 1F, 0.77F)
                        )
                    ) { Text(text = "National", color = Color.DarkGray) }
                    Button(
                        onClick = {
                            coroutineScope.launch { core.update(Event.GetLocal()) }
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