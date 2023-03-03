use error_chain::error_chain;
use std::io::Read;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}


// beware this has ALL been hacked together by someone who knows 0 rust
fn main() -> Result<()> {
    // get chrome releases from rss
    let mut res = reqwest::blocking::get("http://feeds.feedburner.com/GoogleChromeReleases")?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    // parse the xml output
    let mut reader = Reader::from_str(&body);
    reader.trim_text(true);

    let mut toprint: Vec<String> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"title" {
                    let txt = reader
                        .read_text(e.name())
                        .expect("Cannot decode title value").to_string();
                    toprint.push(txt);
                    toprint.push("======".to_string());
                } else if e.name().as_ref() == b"content" {
                    let txt = reader
                        .read_text(e.name())
                        .expect("Cannot decode content value");
                    let unescaped = quick_xml::escape::unescape(&txt).expect("Cannot unescape content").to_string();
                    let textified = nanohtml2text::html2text(&unescaped).to_string().replace("\r\n", "\n");
                    toprint.push(textified);
                    toprint.push("----------------------------------------------------------------------------------cut----------------------------------------------------------------------------------".to_string());
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }
    }

    let slice = &toprint;
    for i in slice {
        println!("{}", i);
    }

    Ok(())
}
