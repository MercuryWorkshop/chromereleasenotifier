use std::io::Read;
use quick_xml::events::Event;
use quick_xml::reader::Reader;


// beware this has ALL been hacked together by someone who knows 0 rust
fn main() {
    let mut shouldprint = false;
    if let Some(firstarg) = std::env::args().nth(1) {
        if firstarg == "print" {
            shouldprint = true;
        } else if firstarg == "help" {
            println!("Chrome Releases Notifier");
            println!("usage: chromereleasenotifer [print|help]");
            println!("adding print will print to the console");
            println!("adding help will show this help screen");
            println!("by default it will send a XDG notification to your DE");
            return;
        }
    }
    // get chrome releases from rss
    let mut res = reqwest::blocking::get("http://feeds.feedburner.com/GoogleChromeReleases").expect("Could not get rss");
    let mut body = String::new();
    res.read_to_string(&mut body).expect("Could not read response");

    // parse the xml output
    let mut reader = Reader::from_str(&body);
    reader.trim_text(true);

    let mut contents: Vec<String> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"content" {
                    let txt = reader
                        .read_text(e.name())
                        .expect("Cannot decode content value");
                    let unescaped = quick_xml::escape::unescape(&txt).expect("Cannot unescape content").to_string();
                    let textified = nanohtml2text::html2text(&unescaped).to_string().replace("\r\n", "\n");
                    if txt.contains("Stable channel") && txt.contains("ChromeOS")  {
                        contents.push(textified);
                    }
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }
    }

    let contentslice = &contents;
    for content in contentslice {
        let splittedcontent = content.split("\n");
        let splittedvec: Vec<&str> = splittedcontent.collect();
        let mut filteredvec: Vec<String> = Vec::new();
        let mut isaftersecurityfixes = false;
        for line in splittedvec {
            if line == "" {
                // skip all empty lines
            } else if line.contains("is being updated to") {
                filteredvec.push(line.to_string());
            } else if line.contains("Security Fixes and Rewards") {
                filteredvec.push("".to_string());
                filteredvec.push(line.to_string());
                isaftersecurityfixes = true;
            } else if isaftersecurityfixes && !(line.contains("Access to bug details and links") || line.contains("We would also like to thank")) {
                filteredvec.push(line.to_string());
            }
        }
        let filteredcontent = filteredvec.join("\n");
        if shouldprint {
            println!("{}", filteredcontent);
            println!("__CUT_HERE");
        } else {
            notify_rust::Notification::new()
                                .summary("Chrome Releases Notifier")
                                .body(&filteredcontent)
                                .show()
                                .expect("Failed to show notification");
        }
    }
}
