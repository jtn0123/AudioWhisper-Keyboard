// Combine actual screenshots without redrawing keyboard content. Run on the development Mac.
import AppKit
import ImageIO
import UniformTypeIdentifiers

let arguments = Array(CommandLine.arguments.dropFirst())
guard arguments.count == 3 else {
    fatalError("usage: swift pair.swift COMPOSE.png RUST.png OUTPUT.png")
}

func keyboard(_ path: String) -> CGImage {
    guard let source = CGImageSourceCreateWithURL(URL(fileURLWithPath: path) as CFURL, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil),
          image.width > 0, image.height > 0,
          let crop = image.cropping(to: CGRect(x: 0, y: 0, width: image.width, height: image.height)) else {
        fatalError("Expected a valid aw_keyboard screenshot")
    }
    return crop
}

let left = keyboard(arguments[0])
let right = keyboard(arguments[1])
let width = left.width + right.width
let height = left.height + 56
guard let context = CGContext(data: nil, width: width, height: height, bitsPerComponent: 8,
                              bytesPerRow: width * 4, space: CGColorSpace(name: CGColorSpace.sRGB)!,
                              bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else {
    fatalError("Could not allocate screenshot pair")
}
context.setFillColor(NSColor.white.cgColor)
context.fill(CGRect(x: 0, y: 0, width: width, height: height))
context.interpolationQuality = .none
context.draw(left, in: CGRect(x: 0, y: 0, width: left.width, height: left.height))
context.draw(right, in: CGRect(x: left.width, y: 0, width: right.width, height: right.height))
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(cgContext: context, flipped: false)
let style: [NSAttributedString.Key: Any] = [.font: NSFont.systemFont(ofSize: 28), .foregroundColor: NSColor.black]
("Compose" as NSString).draw(at: NSPoint(x: 18, y: height - 44), withAttributes: style)
("Rust CPU prototype" as NSString).draw(at: NSPoint(x: left.width + 18, y: height - 44), withAttributes: style)
NSGraphicsContext.restoreGraphicsState()
guard let image = context.makeImage(),
      let destination = CGImageDestinationCreateWithURL(URL(fileURLWithPath: arguments[2]) as CFURL,
                                                       UTType.png.identifier as CFString, 1, nil) else {
    fatalError("Could not create screenshot pair")
}
CGImageDestinationAddImage(destination, image, nil)
guard CGImageDestinationFinalize(destination) else { fatalError("Could not save screenshot pair") }
