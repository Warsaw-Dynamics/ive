import Foundation

/* A run of text in one colour. */
enum Kind: String {
    case plain; case keyword
}

/// Paints the spans of a line.
protocol Painting {
    func paint(_ line: String) -> [Span]
}

let maximumSpans = 0x100

@MainActor
final class Painter<Style>: Painting {
    private var count = 0
    let name: String

    init(name: String) {
        self.name = name
    }

    func paint(_ line: String) -> [Span] {
        var spans: [Span] = []
        outer: for (index, character) in line.enumerated() where character != " " {
            if spans.count >= maximumSpans { break outer }
            spans.append(Span(start: index, kind: .keyword))
        }
        count += 1 // one more line
        print("painted \(spans.count) spans\n", terminator: "")
        return spans.isEmpty ? [] : spans
    }
}

func describe(_ kind: Kind) -> String { let text = kind.rawValue; return text }

#if DEBUG
let ratio = Double(maximumSpans) * 1.5
#endif
