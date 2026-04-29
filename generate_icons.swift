import Cocoa

func createIcon(text: String, color: NSColor, filename: String) {
    let size = NSSize(width: 32, height: 32)
    let image = NSImage(size: size)
    
    image.lockFocus()
    
    // Clear background
    NSColor.clear.set()
    NSRect(origin: .zero, size: size).fill()
    
    let font = NSFont.systemFont(ofSize: 28, weight: .bold)
    let attributes: [NSAttributedString.Key: Any] = [
        .font: font,
        .foregroundColor: color
    ]
    
    let string = NSAttributedString(string: text, attributes: attributes)
    let rect = NSRect(x: 4, y: -2, width: 32, height: 32)
    string.draw(in: rect)
    
    image.unlockFocus()
    
    if let tiffData = image.tiffRepresentation,
       let bitmapImage = NSBitmapImageRep(data: tiffData),
       let pngData = bitmapImage.representation(using: .png, properties: [:]) {
        try? pngData.write(to: URL(fileURLWithPath: filename))
    }
}

createIcon(text: "₿", color: NSColor(calibratedRed: 0.0, green: 0.75, blue: 0.53, alpha: 1.0), filename: "src-tauri/icons/tray-up.png")
createIcon(text: "₿", color: NSColor(calibratedRed: 1.0, green: 0.3, blue: 0.31, alpha: 1.0), filename: "src-tauri/icons/tray-down.png")
