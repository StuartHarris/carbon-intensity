package com.stuartharris.carbon

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
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
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import com.stuartharris.carbon.shared.handleResponse
import com.stuartharris.carbon.shared.processEvent
import com.stuartharris.carbon.shared.view
import com.stuartharris.carbon.shared_types.Effect
import com.stuartharris.carbon.shared_types.HttpResponse
import com.stuartharris.carbon.shared_types.Requests
import com.stuartharris.carbon.ui.theme.CarbonIntensityTheme
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import kotlinx.coroutines.launch
import com.stuartharris.carbon.shared_types.Event as Evt
import com.stuartharris.carbon.shared_types.Request as Req
import com.stuartharris.carbon.shared_types.ViewModel as MyViewModel

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            CarbonIntensityTheme {
                // A surface container using the 'background' color from the theme
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    View()
                }
            }
        }
    }
}

sealed class Outcome {
    data class Http(val res: HttpResponse) : Outcome()
}

sealed class CoreMessage {
    data class Event(val event: Evt) : CoreMessage()
    data class Response(val uuid: List<Byte>, val outcome: Outcome) : CoreMessage()
}

class Model : ViewModel() {
    var view: MyViewModel by mutableStateOf(MyViewModel("", null, null, "", null, null))
        private set

    private val httpClient = HttpClient(CIO)

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
                    }
                )
            )
        }

        for (req in requests) when (val effect = req.effect) {
            is Effect.Render -> {
                this.view = MyViewModel.bincodeDeserialize(view())
            }

            is Effect.Http -> {
                val response = http(
                    httpClient, effect.value
                )
                update(CoreMessage.Response(req.uuid, Outcome.Http(response)))
            }
        }
    }
}

@Composable
fun View(model: Model = viewModel()) {
    val coroutineScope = rememberCoroutineScope()
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

@Preview(showBackground = true)
@Composable
fun GreetingPreview() {
    CarbonIntensityTheme {
        View()
    }
}