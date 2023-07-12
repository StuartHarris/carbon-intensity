package com.stuartharris.carbon.chart

import android.graphics.PointF
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment.Companion.Center
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.stuartharris.carbon.shared_types.GenerationMixPoint
import java.time.ZoneId
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter

@OptIn(ExperimentalTextApi::class)
@Composable
fun MixChart(
    modifier: Modifier,
    points: List<GenerationMixPoint>,
) {
    val textMeasurer = rememberTextMeasurer()

    Box(
        modifier = modifier.padding(horizontal = 16.dp, vertical = 12.dp), contentAlignment = Center
    ) {
        Canvas(
            modifier = Modifier.fillMaxSize()
        ) {
            val yMax = 100
            val yStep = 20
            val yUnit = size.height / yMax.toFloat()

            if (points.isNotEmpty()) {
                val stacked = points.groupingBy {
                    it.date
                }
                    .aggregateTo(mutableMapOf()) { _, acc: Pair<Float, List<GenerationMixPoint>>?, element, _ ->
                        if (acc == null) {
                            element.perc to listOf(element)
                        } else {
                            val cumulative = acc.first + element.perc
                            val newElement =
                                GenerationMixPoint(element.date, element.fuel, cumulative)
                            cumulative to acc.second + newElement
                        }
                    }

                val groups = stacked.values.flatMap { it.second }.groupBy {
                    it.fuel
                }

                val firstSet = groups.values.first()
                val xUnit = size.width / firstSet.size

                for (group in groups) {
                    val mixPoints = group.value
                    val coordinates = mutableListOf<PointF>()
                    val controlPoints1 = mutableListOf<PointF>()
                    val controlPoints2 = mutableListOf<PointF>()

                    // add points
                    for ((i, point) in mixPoints.withIndex()) {
                        val x = i * xUnit
                        val y = size.height - (point.perc * yUnit)
                        coordinates.add(PointF(x, y))
                    }

                    // for Bezier
                    for (i in 1 until coordinates.size) {
                        controlPoints1.add(
                            PointF(
                                (coordinates[i].x + coordinates[i - 1].x) / 2, coordinates[i - 1].y
                            )
                        )
                        controlPoints2.add(
                            PointF(
                                (coordinates[i].x + coordinates[i - 1].x) / 2, coordinates[i].y
                            )
                        )
                    }

                    // line
                    val stroke = Path().apply {
                        reset()
                        moveTo(coordinates.first().x, coordinates.first().y)
                        for (i in 0 until coordinates.size - 1) {
                            cubicTo(
                                controlPoints1[i].x,
                                controlPoints1[i].y,
                                controlPoints2[i].x,
                                controlPoints2[i].y,
                                coordinates[i + 1].x,
                                coordinates[i + 1].y
                            )
                        }
                    }

                    // fill
//                        val fillPath =
//                            android.graphics.Path(stroke.asAndroidPath()).asComposePath().apply {
//                                lineTo(coordinates.last().x, size.height)
//                                lineTo(0f, size.height)
//                                close()
//                            }
//                        drawPath(
//                            fillPath,
//                            brush = Brush.verticalGradient(
//                                listOf(
//                                    Color(0xff4fabd5),
//                                    Color.Transparent,
//                                )
//                            ),
//                        )
                    drawPath(
                        stroke, color = Color(0x884fabd5), style = Stroke(
                            width = 4f, cap = StrokeCap.Round
                        )
                    )
                }

                // x-axis
                for (i in firstSet.indices step 2) {
                    val text = ZonedDateTime.parse(firstSet[i].date)
                        .withZoneSameInstant(ZoneId.systemDefault())
                        .format(DateTimeFormatter.ofPattern("HH:mm"))
                    val textLayoutResult = textMeasurer.measure(
                        text = AnnotatedString(text),
                        style = TextStyle(fontSize = 10.sp, color = Color.Black)
                    )
                    val textSize = textLayoutResult.size

                    val lineX = i * xUnit
                    val topLeft = Offset(lineX - textSize.width, size.height)
                    val pivot = Offset(lineX, size.height + textSize.height / 2)
                    rotate(-45f, pivot) {
                        drawText(
                            textLayoutResult = textLayoutResult,
                            topLeft = topLeft,
                        )
                    }
                    drawLine(
                        Color.LightGray, pivot, Offset(lineX, 0f)
                    )
                }

                // y-axis
                for (i in 0..yMax step yStep) {
                    val text = i.toString()
                    val textLayoutResult = textMeasurer.measure(
                        text = AnnotatedString(text),
                        style = TextStyle(fontSize = 10.sp, color = Color.Black)
                    )
                    val textSize = textLayoutResult.size

                    val lineY = size.height - (i * yUnit)
                    if (i > 0) {
                        drawText(
                            textLayoutResult = textLayoutResult,
                            topLeft = Offset(0f - textSize.width / 2, lineY - textSize.height / 2),
                        )
                    }
                    drawLine(
                        Color.LightGray, Offset(0f, lineY), Offset(size.width, lineY)
                    )
                }
            }
        }
    }
}