use local_ip_address::local_ip;
use arboard::Clipboard;
use qr2term::print_qr;

pub fn print_banner(port: u16, no_clipboard: bool) {
    let local_ip = local_ip().map(|ip| ip.to_string()).unwrap_or_else(|_| "127.0.0.1".to_string());
    let local_url = format!("http://localhost:{}", port);
    let network_url = format!("http://{}:{}", local_ip, port);

    let mut clipboard_msg = "";
    if !no_clipboard {
        if let Ok(mut clipboard) = Clipboard::new() {
            if clipboard.set_text(local_url.clone()).is_ok() {
                clipboard_msg = "   Copied local address to clipboard!";
            }
        }
    }


    println!("                                                         ");
    println!("   Serving!                                              ");
    println!("                                                         ");
    println!("   - Local:            {}             ", local_url);
    println!("   - On Your Network:  {}          ", network_url);
    println!("                                                         ");
    if !clipboard_msg.is_empty() {
        println!("{}                    ", clipboard_msg);
        println!("                                                         ");
    }
    println!("");
    println!("Sacan QR Code");
    print_qr(&network_url).unwrap();
}
