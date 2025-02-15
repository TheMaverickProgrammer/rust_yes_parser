use std::{collections::HashMap, fmt};

use yes_parser::{
    element::Element,
    enums::{Elements, ErrorCodes},
    YesDocParser,
};

extern crate yes_parser;

fn main() {
    let doc = "!version 1.0.2
        window width=320 height=240 fullscreen
        volume sfx=100 music=50
        lang en

        @default
        controls left_handed
            key A 13
            key Z 1
            key X 54
            # etc...

        controls standard
            invert_y
            key SPACE 100
            key RIGHT 101
            key LEFT 213
            # etc...";

    let result = ConfigBuilder::from_string(doc);

    match &result {
        Err(e) => println!("{}", e),
        _ => (),
    }

    assert!(result.is_ok());

    let config = result.expect("Expected result to be ok.");
    println!("{}", config);

    assert_eq!(config.version, "1.0.2");
    assert_eq!(config.window.width, 320);
    assert_eq!(config.window.height, 240);
    assert_eq!(config.window.fullscreen, true);
    assert_eq!(config.volume.sfx, 100.0);
    assert_eq!(config.volume.music, 50.0);
    assert_eq!(config.default_controller_idx, Some(0));
    let controller1 = config.controllers.get("left_handed");
    let controller2 = config.controllers.get("standard");

    assert!(controller1.is_some());
    assert!(controller2.is_some());

    let left_handed = controller1.unwrap();
    assert_eq!(left_handed.invert_y, false);
    assert_eq!(*left_handed.keys.get("A").unwrap(), 13);
    assert_eq!(*left_handed.keys.get("Z").unwrap(), 1);
    assert_eq!(*left_handed.keys.get("X").unwrap(), 54);

    let standard = controller2.unwrap();
    assert_eq!(standard.invert_y, true);
    assert_eq!(*standard.keys.get("SPACE").unwrap(), 100);
    assert_eq!(*standard.keys.get("RIGHT").unwrap(), 101);
    assert_eq!(*standard.keys.get("LEFT").unwrap(), 213);

    println!("Done!");
}

struct Window {
    width: u16,
    height: u16,
    fullscreen: bool,
}

struct Volume {
    sfx: f32,
    music: f32,
}

struct Controller {
    name: String,
    keys: HashMap<String, u8>,
    invert_y: bool,
}

impl Controller {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            keys: HashMap::new(),
            invert_y: false,
        }
    }
}

#[derive(PartialEq, Clone)]
enum Sections {
    Window,
    Volume,
    Lang,
    Controls,
    Unsupported,
}

impl Sections {
    fn value(&self) -> &'static str {
        match self {
            Sections::Window => "window",
            Sections::Volume => "volume",
            Sections::Lang => "lang",
            Sections::Controls => "controls",
            Sections::Unsupported => "",
        }
    }

    fn from_text(text: &str) -> Sections {
        match text {
            "window" => Sections::Window,
            "volume" => Sections::Volume,
            "lang" => Sections::Lang,
            "controls" => Sections::Controls,
            _ => Sections::Unsupported,
        }
    }
}

struct Config {
    window: Window,
    volume: Volume,
    lang: String,
    controllers: HashMap<String, Controller>,
    default_controller_idx: Option<usize>,
    version: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "version={}", self.version)?;
        writeln!(
            f,
            "window={{width={}, height={}, fullscreen={}}}",
            self.window.width, self.window.height, self.window.fullscreen
        )?;
        writeln!(
            f,
            "volume={{sfx={}, music={}}}",
            self.volume.sfx, self.volume.music
        )?;
        writeln!(f, "lang={}", self.lang)?;

        for (name, c) in &self.controllers {
            writeln!(f, "controller {}", name)?;
            writeln!(f, "   invert_y={}", c.invert_y)?;
            for (action, code) in &c.keys {
                writeln!(f, "   key={{{} => {}}}", action, code)?;
            }
        }

        writeln!(
            f,
            "default_controller_idx={}",
            if let Some(idx) = self.default_controller_idx {
                idx.to_string()
            } else {
                "None".to_string()
            }
        )?;

        Ok(())
    }
}

impl Config {
    fn new() -> Self {
        Self {
            window: Window {
                width: 800,
                height: 600,
                fullscreen: false,
            },
            volume: Volume {
                sfx: 1.0,
                music: 1.0,
            },
            lang: String::new(),
            controllers: HashMap::new(),
            default_controller_idx: None,
            version: String::new(),
        }
    }
}

struct ConfigBuilder {
    section: Sections,
    config: Config,
    controller: Option<Controller>,
}

impl ConfigBuilder {
    fn from_string(body: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let results = YesDocParser::from_string(body, None);

        let mut builder = ConfigBuilder {
            section: Sections::Unsupported,
            config: Config::new(),
            controller: None,
        };

        for result in results {
            match result {
                yes_parser::ParseResult::Ok { line_number, data } => {
                    builder.process(line_number, &data)?;
                }
                yes_parser::ParseResult::Err {
                    line_number,
                    message,
                    code,
                } => {
                    // The spec must report why it could not parse something.
                    // Generally EOL can be ignored safely depending on your
                    // expectations for your documents and scripts.
                    if code == ErrorCodes::EolNoData {
                        continue;
                    }

                    return Err(format!("#{}: {}", line_number, message).into());
                }
            }
        }

        // Flush or commit any staged elements being built before end of file.
        builder.commit_controller();

        Ok(builder.config)
    }

    // Our config file only ever needs to inspect our custom standard elements
    // and global settings.
    fn process(
        &mut self,
        line_number: usize,
        data: &Elements,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match data {
            Elements::Standard { attrs, element } => {
                self.consume_standard(line_number, &attrs, &element)
            }
            Elements::Global(element) => self.consume_global(line_number, &element),

            // Effectively ignores comments
            _ => Ok(()),
        }
    }

    fn consume_standard(
        &mut self,
        line_number: usize,
        attrs: &Vec<Element>,
        element: &Element,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.update_section(&element.text) {
            Sections::Window => self.handle_window_field(&line_number, &element)?,
            Sections::Volume => self.handle_volume_field(&line_number, &element)?,
            Sections::Lang => self.handle_lang_field(&line_number, &element)?,
            // This sub-section has rules of its own. In this example, we enforce some
            // of those rules. In production, move the parsing code into its own file
            // to improve readability and make responsibility clear.
            Sections::Controls => self.handle_controls_section(&line_number, &attrs, &element)?,
            Sections::Unsupported => Err(format!(
                "#{}: Unexpected section {}!",
                line_number, element.text
            ))?,
        }
        Ok(())
    }

    fn handle_window_field(
        &mut self,
        line_number: &usize,
        element: &Element,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for arg in &element.args {
            // The existence of the value "fullscreen" implies that
            // fullscreen=true. This keyval has no key to identify it.
            // We could enforce positional parameters, or check to see
            // if this keyval is the value we're looking for.
            if arg.is_nameless() {
                if arg.val == "fullscreen" {
                    self.config.window.fullscreen = true;
                }
                continue;
            }

            match arg.key.as_ref().unwrap().as_str() {
                "width" => self.config.window.width = arg.val.parse::<u16>()?,
                "height" => self.config.window.height = arg.val.parse::<u16>()?,
                _ => {
                    return Err(format!(
                        "#{}: Unknown field {} for section {}",
                        line_number,
                        element.text,
                        self.section.value()
                    )
                    .into())
                }
            }
        }

        Ok(())
    }

    fn handle_volume_field(
        &mut self,
        line_number: &usize,
        element: &Element,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for arg in &element.args {
            // We could allow position parameters, but in this example
            // we will only concern ourselves with explicit arguments.
            if arg.is_nameless() {
                continue;
            }

            match arg.key.as_ref().unwrap().as_str() {
                "sfx" => self.config.volume.sfx = arg.val.parse::<f32>()?,
                "music" => self.config.volume.music = arg.val.parse::<f32>()?,
                _ => {
                    return Err(format!(
                        "#{}: Unknown field {} for section {}",
                        line_number,
                        element.text,
                        self.section.value()
                    )
                    .into())
                }
            }
        }

        Ok(())
    }

    fn handle_lang_field(
        &mut self,
        line_number: &usize,
        element: &Element,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // We are only interested in the first value in this element.
        // Depending on your implementation needs, you can enforce
        // arg length restrictions.
        let len = element.args.len();
        if len != 1 {
            return Err(format!(
                "#{}: Mismatch argument length {} for lang. Expected only 1!",
                line_number, len
            )
            .into());
        }

        self.config.lang = element.args.first().unwrap().val.clone();

        Ok(())
    }

    // This element has its own collection of elements that relate to it.
    // Therefore this routine must check to see if it belongs to the
    // the start of the section itself (the "block") for the fields, or if
    // it is an element that pertains to the section.
    //
    // In production, move sub-parser code into its own struct implementation
    // to improve clarity and readability. Hopefully, this example is small
    // enough to prove a point on utility.
    fn handle_controls_section(
        &mut self,
        line_number: &usize,
        attrs: &Vec<Element>,
        element: &Element,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let label = element.text.as_str();
        match label {
            "controls" => self.handle_new_controls(&line_number, &attrs, &element)?,
            "invert_y" => {
                // We must have a previous entry by this element.
                if let Some(ref mut c) = self.controller {
                    c.invert_y = true;
                } else {
                    return Err(format!(
                        "#{}: Expected a controls entry before property {}.",
                        line_number, label
                    )
                    .into());
                }
            }
            "key" => {
                // We must have a previous entry by this element.
                if let Some(ref mut c) = self.controller {
                    // We expect 2 positional arguments
                    let args = &element.args;
                    if args.len() != 2 {
                        return Err(format!(
                        "#{}: key property expects the following format: `key <action> <code>`.",
                        line_number
                    )
                        .into());
                    }

                    let iter = &mut args.iter();
                    let first = iter.nth(0).unwrap();
                    let second = iter.nth(0).unwrap();

                    // Enforce position arguments.
                    // Alternatively, a reader could check for
                    // keyval names before enforcing positions.
                    if first.is_nameless() && second.is_nameless() {
                        c.keys.insert(first.val.clone(), second.val.parse::<u8>()?);
                    } else {
                        return Err(format!(
                        "#{}: key property fields do not match expected format: `key <action> <code>`.",
                        line_number
                    )
                    .into());
                    }
                } else {
                    return Err(format!(
                        "#{}: Expected a controls entry before property {}.",
                        line_number, label
                    )
                    .into());
                }
            }
            _ => {
                return Err(format!(
                    "#{}: Unknown property {} for section {}",
                    line_number,
                    element.text,
                    self.section.value()
                )
                .into())
            }
        }

        Ok(())
    }

    fn handle_new_controls(
        &mut self,
        line_number: &usize,
        attrs: &Vec<Element>,
        element: &Element,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if element.args.len() != 1 {
            return Err(format!("#{}: A new controls section expects a name!", line_number).into());
        }

        let name = {
            let arg = element.args.first().unwrap();
            // For more intelligent parsing, and convenience to others,
            // we can check for keyval names before reading values by
            // their position in the element.
            if let Some(ref name) = arg.key {
                if name != "name" {
                    return Err(format!(
                        "#{}: Unknown field {}. Expected `name` or leave blank!",
                        line_number, name
                    )
                    .into());
                }
            }

            arg.val.clone()
        };

        // Stash the previous one
        self.commit_controller();

        // And track a new one
        self.controller = Some(Controller::new(&name.clone()));

        // Attributes can apply special behavior to elements.
        if !attrs.is_empty() {
            // We are only interested in supporting "default" attribute
            // in this example.
            for attr in attrs {
                if attr.text == "default" {
                    // len is also the next controller's index once commited
                    self.config.default_controller_idx = Some(self.config.controllers.len())
                }
            }
        }

        Ok(())
    }

    fn update_section(&mut self, text: &String) -> Sections {
        let next_section = Sections::from_text(text);

        // It's likely this is an expected element for a subsection.
        // More robust validation could be done here but this is only for
        // example purposes and will suffice.
        if next_section == Sections::Unsupported {
            // Continue processing the current section.
            return self.section.clone();
        }

        // Switch to and process the next section.
        self.section = next_section;
        self.section.clone()
    }

    // The YES spec states that global elements impact the whole document.
    // The parser will hoist these elements to the front of the returned list
    // in the order that they were parsed. Therefore we can safely process
    // globals before the rest of the document. This is useful if globals
    // change the way a document should be parsed, for example, if the value
    // for `!version` dictates the availability of some features or if the
    // the existence of `!assert_image x` in a scriplet verifies that the
    // image resource `x` is loaded in memory and aborts the rest of the
    // scriplet otherwise.
    fn consume_global(
        &mut self,
        line_number: usize,
        element: &Element,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match element.text.as_str() {
            "version" => {
                if let Some(v) = element.args.first() {
                    self.config.version = v.val.clone();
                } else {
                    return Err(format!("#{}: Version number expected!", line_number).into());
                }
            }
            _ => {
                return Err(
                    format!("#{}: Unsupported global '{}'", line_number, element.text).into(),
                )
            }
        }

        Ok(())
    }

    fn commit_controller(&mut self) {
        if let Some(c) = self.controller.take() {
            self.config.controllers.insert(c.name.clone(), c);
        }
    }
}
