import Charts
import SharedTypes
import SwiftUI

@MainActor
class Model: ObservableObject {
    @Published var view = ViewModel(
        mode: .national,
        national_name: "",
        national_intensity: [],
        national_mix: [],
        local_name: "",
        local_intensity: [],
        local_mix: []
    )

    init() {
        update(event: .getNational)
    }

    func update(event: Event) {
        let effects = [UInt8](processEvent(Data(try! event.bincodeSerialize())))

        process_effects(effects)
    }

    func process_effects(_ effects: [UInt8]) {
        let requests: [Request] = try! .bincodeDeserialize(input: effects)
        for request in requests {
            process_request(request)
        }
    }

    func process_request(_ request: Request) {
        switch request.effect {
        case .render:
            view = try! .bincodeDeserialize(input: [UInt8](CarbonIntensity.view()))
        case let .http(req):
            Task {
                let response = try! await httpRequest(req).get()

                let effects = [UInt8](handleResponse(Data(request.uuid), Data(try! response.bincodeSerialize())))

                process_effects(effects)
            }
        case .time:
            let response = TimeResponse(value: Date().ISO8601Format())
            let effects = [UInt8](handleResponse(Data(request.uuid), Data(try! response.bincodeSerialize())))

            process_effects(effects)
        case let .getLocation(req):
            let response = try! locationRequest(req).get()

            let effects = [UInt8](handleResponse(Data(request.uuid), Data(try! response.bincodeSerialize())))

            process_effects(effects)
        }
    }
}

struct ActionButton: View {
    var label: String
    var color: Color
    var action: () -> Void

    init(label: String, color: Color, action: @escaping () -> Void) {
        self.label = label
        self.color = color
        self.action = action
    }

    var body: some View {
        Button(action: action) {
            Text(label)
                .fontWeight(.bold)
                .font(.body)
                .padding(EdgeInsets(top: 10, leading: 15, bottom: 10, trailing: 15))
                .background(color)
                .cornerRadius(10)
                .foregroundColor(.white)
                .padding()
        }
    }
}

struct ContentView: View {
    @ObservedObject var model: Model

    let isoFormatter = ISO8601DateFormatter()
    let timeFormatter = DateFormatter()
    let intensity_color = Color(hex: 0x36A2EB)

    let fillColors: KeyValuePairs<String, Color> = [
        "Coal": Color(hex: 0x2C2A28),
        "Gas": Color(hex: 0x7030A0),
        "Imports": Color(hex: 0xEB556E),
        "Biomass": Color(hex: 0xEF8534),
        "Nuclear": Color(hex: 0x4B8A44),
        "Hydro": Color(hex: 0x396CCB),
        "Wind": Color(hex: 0x4FABD5),
        "Solar": Color(hex: 0xF7D147),
    ]

    init(model: Model) {
        self.model = model
        timeFormatter.dateFormat = "HH:mm"
    }

    private func formatDate(_ date: String) -> String {
        let d = isoFormatter.date(from: date)
        if d != nil {
            return timeFormatter.string(from: d!)
        } else { return "00:00" }
    }

    var body: some View {
        VStack {
            Text("Carbon Intensity").font(.headline)
            Text(model.view.mode == .local ? model.view.local_name : model.view.national_name).padding()
            Chart(model.view.mode == .local ? model.view.local_intensity : model.view.national_intensity) {
                AreaMark(
                    x: .value("Time", $0.date),
                    y: .value("gCO2/kWh", $0.forecast)
                ).opacity(0.5).accessibilityHidden(true)
                LineMark(
                    x: .value("Time", $0.date),
                    y: .value("gCO2/kWh", $0.forecast)
                )
            }.foregroundStyle(intensity_color)
                .frame(height: 250)
                .chartYScale(domain: 0 ... 600)
                .chartXAxis(content: {
                    AxisMarks { value in
                        AxisValueLabel {
                            let x = formatDate(value.as(String.self)!)
                            if x.hasSuffix("00") {
                                Text(x)
                                    .rotationEffect(Angle(degrees: -45))
                            }
                        }
                    }
                })
            Chart(model.view.mode == .local ? model.view.local_mix : model.view.national_mix) {
                AreaMark(
                    x: .value("Time", $0.date),
                    y: .value("Percent", $0.perc)
                ).opacity(0.5)
                    .foregroundStyle(by: .value("Fuel", $0.fuel))
            }
            .frame(height: 250)
            .chartYScale(domain: 0 ... 100)
            .chartXAxis(content: {
                AxisMarks { value in
                    AxisValueLabel {
                        let x = formatDate(value.as(String.self)!)
                        if x.hasSuffix("00") {
                            Text(x)
                                .rotationEffect(Angle(degrees: -45))
                        }
                    }
                }
            })
            .chartForegroundStyleScale(fillColors)
            HStack {
                ActionButton(label: "National", color: .yellow) {
                    model.update(event: .getNational)
                }
                ActionButton(label: "Local", color: .red) {
                    model.update(event: .getLocal)
                }
            }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView(model: Model())
    }
}

extension IntensityPoint: Identifiable {
    public typealias ID = String
    public var id: String {
        return date
    }
}

extension GenerationMixPoint: Identifiable {
    public typealias ID = String
    public var id: String {
        return date
    }
}

extension String {
    func capitalizingFirstLetter() -> String {
        return prefix(1).capitalized + dropFirst()
    }

    mutating func capitalizeFirstLetter() {
        self = capitalizingFirstLetter()
    }
}

extension Color {
    init(hex: UInt, alpha: Double = 1) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 08) & 0xFF) / 255,
            blue: Double((hex >> 00) & 0xFF) / 255,
            opacity: alpha
        )
    }
}
