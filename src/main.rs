use dialog::DialogBox;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::Read;

fn showdialog(title: String, content: String) {
    let mut backend = dialog::backends::KDialog::new(); // zenity doesn't want to use my body content (it just says "All updates are complete")
    backend.set_icon("info");
    dialog::Message::new(content)
        .title(title)
        .show_with(&backend)
        .expect("Could not display dialog box");
}

#[derive(Clone)]
struct MdDecorator {
    currentlink: String,
}

impl MdDecorator {
    pub fn new() -> MdDecorator {
        MdDecorator {
            currentlink: "".into(),
        }
    }
}

impl html2text::render::text_renderer::TextDecorator for MdDecorator {
    type Annotation = ();

    fn decorate_link_start(&mut self, _url: &str) -> (String, Self::Annotation) {
        self.currentlink = _url.into();
        ("[".to_string(), ())
    }

    fn decorate_link_end(&mut self) -> String {
        format!("]({})", self.currentlink)
    }

    fn decorate_em_start(&mut self) -> (String, Self::Annotation) {
        ("*".to_string(), ())
    }

    fn decorate_em_end(&mut self) -> String {
        "*".to_string()
    }

    fn decorate_strong_start(&mut self) -> (String, Self::Annotation) {
        ("**".to_string(), ())
    }

    fn decorate_strong_end(&mut self) -> String {
        "**".to_string()
    }

    fn decorate_strikeout_start(&mut self) -> (String, Self::Annotation) {
        ("~~".to_string(), ())
    }

    fn decorate_strikeout_end(&mut self) -> String {
        "~~".to_string()
    }

    fn decorate_code_start(&mut self) -> (String, Self::Annotation) {
        ("`".to_string(), ())
    }

    fn decorate_code_end(&mut self) -> String {
        "`".to_string()
    }

    fn decorate_preformat_first(&mut self) -> Self::Annotation {
    }
    fn decorate_preformat_cont(&mut self) -> Self::Annotation {
    }

    fn decorate_image(&mut self, _src: &str, title: &str) -> (String, Self::Annotation) {
        (format!("[{}]", title), ())
    }

    fn header_prefix(&mut self, level: usize) -> String {
        "#".repeat(level) + " "
    }

    fn quote_prefix(&mut self) -> String {
        "> ".to_string()
    }

    fn unordered_item_prefix(&mut self) -> String {
        "* ".to_string()
    }

    fn ordered_item_prefix(&mut self, i: i64) -> String {
        format!("{}. ", i)
    }

    fn finalise(
        &mut self,
        _links: Vec<String>,
    ) -> Vec<html2text::render::text_renderer::TaggedLine<()>> {
        Vec::new()
    }

    fn make_subblock_decorator(&self) -> Self {
        self.clone()
    }
}

// beware this has ALL been hacked together by someone who knows 0 rust
fn main() {
    let mut shouldprint = false;
    let mut shouldfull = false;
    if let Some(firstarg) = std::env::args().nth(1) {
        if firstarg == "print" {
            shouldprint = true;
        } else if firstarg == "full" {
            shouldfull = true;
            shouldprint = true;
        } else if firstarg == "help" {
            println!("Chrome Releases Notifier");
            println!("usage: chromereleasenotifer [print|help|full]");
            println!("requires kdialog");
            println!("adding print will print to the console");
            println!("adding help will show this help screen");
            println!("adding full will print the full content of each update to the console instead of the boiled down version");
            println!("by default it will send a XDG notification to your DE");
            return;
        }
    }
    // get chrome releases from rss
    let mut res = reqwest::blocking::get("http://feeds.feedburner.com/GoogleChromeReleases")
        .expect("Could not get rss");
    let mut body = String::new();
    res.read_to_string(&mut body)
        .expect("Could not read response");

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
                    let unescaped = quick_xml::escape::unescape(&txt)
                        .expect("Cannot unescape content")
                        .to_string();
                    let decorator = MdDecorator::new();
                    let mut textified = html2text::from_read_with_decorator(
                        unescaped.as_bytes(),
                        std::usize::MAX,
                        decorator,
                    )
                    .to_string()
                    .replace("\r\n", "\n");
                    let line_spam_regex = regex::Regex::new(r"\n{2,}").unwrap();
                    textified = line_spam_regex.replace_all(&textified, "\n").to_string();
                    if
                    /*txt.contains("Stable channel") &&*/
                    txt.contains("ChromeOS") {
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
        let mut filteredcontent = content.to_owned();
        if !shouldfull {
            let mut filteredvec: Vec<String> = Vec::new();
            let splittedcontent = content.split('\n');
            let splittedvec: Vec<&str> = splittedcontent.collect();
            let mut isaftersecurityfixes = false;
            for l in splittedvec {
                let line = l.trim();
                if line.is_empty() {
                    // skip all empty lines
                } else if line.contains("has been updated to") {
                    // Beta/dev channel is also different
                    let splittedline: Vec<&str> =
                        line.split(" for most ChromeOS devices.").collect();
                    filteredvec.push(splittedline[0].to_string());
                } else if line.contains("is being updated to") {
                    // The * channel is being updated to <version> (Platform version: <version>) is all we need
                    let splittedline: Vec<&str> = line
                        .split(" for most ChromeOS devices and will be rolled out")
                        .collect();
                    filteredvec.push(splittedline[0].to_string() + ".");
                } else if line.contains("is being updated in the LTS") {
                    // LTS uses a different post format
                    let splittedline: Vec<&str> = line.split("Want to know more about").collect();
                    filteredvec.push(splittedline[0].to_string());
                } else if line.contains("A new LTC-") {
                    // LTC uses a different post format
                    let splittedline: Vec<&str> =
                        line.split(" for most ChromeOS devices.").collect();
                    filteredvec.push(splittedline[0].to_string() + ".");
                } else if line.contains("Security Fixes and Rewards") || line.contains("Security fixes") {
                    filteredvec.push("".to_string());
                    filteredvec.push(line.to_string());
                    isaftersecurityfixes = true;
                } else if isaftersecurityfixes {
                    if line.contains("Access to bug details and links") {
                        // skip
                    } else if !line.contains("We would also like to thank") {
                        filteredvec.push(line.to_string());
                    } else {
                        isaftersecurityfixes = false;
                    }
                }
            }
            filteredcontent = filteredvec.join("\n");
        }

        if shouldprint {
            println!("{}", filteredcontent);
            println!("__CUT_HERE");
        } else {
            notify_rust::Notification::new()
                .summary("Chrome Releases Notifier")
                .body(&filteredcontent)
                .action("clicked_info", "More Info")
                .show()
                .unwrap()
                .wait_for_action(|action| if action == "clicked_info" {
                        showdialog("Chrome Releases Notifier".to_string(), content.to_owned())
                });
        }
    }
}
