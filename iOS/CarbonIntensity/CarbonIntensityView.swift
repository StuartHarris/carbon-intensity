import Charts
import SharedTypes
import SwiftUI

enum Outcome {
    case http(HttpResponse)
    case location(LocationResponse)
    case time(TimeResponse)
}

typealias Uuid = [UInt8]

enum Message {
    case event(Event)
    case response(Uuid, Outcome)
}

@MainActor
class Model: ObservableObject {
    @Published var view = ViewModel(
        national_name: "",
        national_intensity: [],
        national_mix: [],
        local_name: "",
        local_intensity: [],
        local_mix: []
    )
    
    init() {
        update(msg: .event(.getNational))
    }
    
    func update(msg: Message) {
        var requests: [Request]
        
        switch msg {
        case let .event(event):
            requests = try! .bincodeDeserialize(
                input: [UInt8](processEvent(Data(try! event.bincodeSerialize())))
            )
        case let .response(uuid, .http(response)):
            requests = try! .bincodeDeserialize(
                input: [UInt8](handleResponse(Data(uuid), Data(try! response.bincodeSerialize())))
            )
        case let .response(uuid, .time(response)):
            requests = try! .bincodeDeserialize(
                input: [UInt8](handleResponse(Data(uuid), Data(try! response.bincodeSerialize())))
            )
        case let .response(uuid, .location(response)):
            requests = try! .bincodeDeserialize(
                input: [UInt8](handleResponse(Data(uuid), Data(try! response.bincodeSerialize())))
            )
        }
        
        for request in requests {
            switch request.effect {
            case .render: view = try! ViewModel.bincodeDeserialize(input: [UInt8](CarbonIntensity.view()))
                
            case let .http(httpReq):
                Task {
                    let res = try! await httpRequest(httpReq).get()
                    update(msg: .response(request.uuid, .http(res)))
                }
                
            case .time:
                update(msg: .response(request.uuid, .time(TimeResponse(value: Date().ISO8601Format()))))
                
            case let .getLocation(req):
                let res = try! locationRequest(req).get()
                update(msg: .response(request.uuid, .location(res)))
            }
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
    
    let formatter = ISO8601DateFormatter()
    
    let fillColors: KeyValuePairs<String, Color> = [
        "Coal": Color(hex: 0x2c2a28, alpha: 0.6),
        "Gas": Color(hex: 0x7030a0, alpha: 0.6),
        "Other": Color(hex: 0xacddaa, alpha: 0.6),
        "Imports": Color(hex: 0xeb556e, alpha: 0.6),
        "Biomass": Color(hex: 0xef8534, alpha: 0.6),
        "Nuclear": Color(hex: 0x4b8a44, alpha: 0.6),
        "Hydro": Color(hex: 0x396ccb, alpha: 0.6),
        "Wind": Color(hex: 0x4fabd5, alpha: 0.6),
        "Solar": Color(hex: 0xf7d147, alpha: 0.6)
    ]

    init(model: Model) {
        self.model = model
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    }
    
    private func formatDate(_ date: String) -> String {
        let d = formatter.date(from: date)
        if d != nil {
            return d!.formatted(date: .omitted, time: .shortened)
        } else { return "00:00" }
    }
    
    var body: some View {
        VStack
        {
            Text("Carbon Intensity").font(.headline)
            Text(model.view.local_name).padding()
            Chart(model.view.national_intensity) {
                LineMark(
                    x: .value("Time", $0.hh_mm),
                    y: .value("gCO2/kWh", $0.forecast)
                )
            }.frame(height: 250)
                .chartYScale(domain: 0...600)
                .chartXAxis(content: {
                    AxisMarks { value in
                        AxisValueLabel {
                            if let x = value.as(String.self) {
                                if x.hasSuffix("00") {
                                    Text(x)
                                        .rotationEffect(Angle(degrees: 315))
                                }
                            }
                        }
                    }
                })
            Chart(model.view.national_mix) {
                AreaMark(
                    x: .value("Time", $0.hh_mm),
                    y: .value("Percent", $0.perc)
                ).foregroundStyle(by: .value("Fuel", $0.fuel))
            }.frame(height: 250)
                .chartYScale(domain: 0...100)
                .chartXAxis(content: {
                    AxisMarks { value in
                        AxisValueLabel {
                            if let x = value.as(String.self) {
                                if x.hasSuffix("00") {
                                    Text(x)
                                        .rotationEffect(Angle(degrees: 315))
                                }
                            }
                        }
                    }
                })
                .chartForegroundStyleScale(fillColors)
            HStack {
                ActionButton(label: "National", color: .yellow) {
                    model.update(msg: .event(.getNational))
                }
                ActionButton(label: "Local", color: .red) {
                    model.update(msg: .event(.getLocal))
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
            red: Double((hex >> 16) & 0xff) / 255,
            green: Double((hex >> 08) & 0xff) / 255,
            blue: Double((hex >> 00) & 0xff) / 255,
            opacity: alpha
        )
    }
}
