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
    @Published var view = ViewModel(national_name: "", national: [], local_name: "", local: [])
    
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
                print(httpReq)
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
        
    var body: some View {
        VStack {
            Text("Carbon Intensity").font(.headline)
            Text(model.view.local_name).padding()
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
