use std::{collections::BTreeMap, path::PathBuf};

use clap::Parser;
use quick_xml::events::Event;

#[derive(clap::Parser)]
struct Options {
    /// The input file
    #[arg(short, long)]
    pub input_file: PathBuf,
    /// The output file
    #[arg(short, long)]
    pub output: PathBuf,
    /// What should be used to join the keys to form a unique key
    #[arg(short, long, default_value = ".")]
    pub key_separator: String,
    /// What should be used as the key for a row?
    #[arg(short, long, default_value = "EXPERIMENT_PACKAGE")]
    pub row_separator: String,
    // /// Show a progress indicator
    // #[arg(short, long)]
    // pub progress: bool,
}

fn main() {
    run(Options::parse()).unwrap()
}

fn to_key(stack: &[Vec<u8>], separator: &str) -> String {
    let stacks: Vec<String> = stack
        .iter()
        .map(|x| String::from_utf8_lossy(x).to_string())
        .collect();
    stacks.join(separator)
}

fn run(options: Options) -> anyhow::Result<()> {
    let mut f = quick_xml::Reader::from_file(&options.input_file)?;
    let output_separator = &options.row_separator.as_bytes();
    let mut buf = Vec::new();
    let mut stack = Vec::new();
    let mut output_vec = Vec::new();
    let mut dictionary = BTreeMap::new();
    loop {
        match f.read_event_into(&mut buf) {
            // Parsing error -> Invalid XML
            Err(e) => {
                // Parsing error -> Invalid XML, let's print out what we had till now though
                eprintln!(
                    "Invalid XML: Error at position {}: {:?}",
                    f.buffer_position(),
                    e
                );
                eprintln!("parsing will stop, but current output will be written");
                break;
            }
            // End of the file -> we're done, can start outputting
            Ok(Event::Eof) => break,
            // We ignore these events as they should not have any content for us
            Ok(Event::DocType(_) | Event::CData(_) | Event::PI(_)) => {}
            // Push onto the stack and add the attributes as values
            Ok(Event::Start(e)) => {
                let e = e.to_owned();
                for attr in e.attributes() {
                    let attr = attr.expect("Failed to read attribute");
                    let mut stack_key = stack.clone();
                    stack_key.push(attr.key.0.to_vec());

                    let k = to_key(&stack_key, &options.key_separator);
                    let unescaped = attr.unescape_value();
                    let value = if let Ok(unescaped) = unescaped {
                        unescaped.to_string()
                    } else {
                        let raw = attr.value.to_vec();
                        let raw = String::from_utf8(raw)
                            .expect("Non-utf8 attribute value")
                            .replace("&amp;", "&");
                        raw
                    };
                    let entry = dictionary.entry(k).or_insert(Vec::new());
                    entry.push(value);
                }

                stack.push(e.name().0.to_vec());
            }
            // Pop off the stack, if we reach the row separator we output the dictionary
            Ok(Event::End(e)) => {
                // We perform a check to see that we're nesting correctly.
                let popped_e = stack.pop().expect("End event without start");
                let start_word = popped_e;
                let end_word = e.name().0;
                assert_eq!(start_word, end_word, "Start and end words do not match");
                if end_word == *output_separator {
                    output_vec.push(dictionary.clone());
                    dictionary.clear();
                }
            }
            // These are some of the values we care about
            Ok(Event::Text(mut e)) => {
                if e.inplace_trim_end() || e.inplace_trim_start() {
                    continue;
                }
                let k = to_key(&stack[..], &options.key_separator);
                let entry = dictionary.entry(k).or_insert(Vec::new());
                let v = e.unescape().expect("non-utf8").to_string();
                entry.push(v);
            }
            Ok(_event) => {}
        }
        buf.clear();
    }

    // Check that the stack and dictionary are empty
    if !stack.is_empty() {
        eprintln!("Stack is not empty");
        eprintln!("{:?}", stack.iter().map(|x| String::from_utf8_lossy(x).to_string()).collect::<Vec<String>>());
    }
    if !dictionary.is_empty() {
        eprintln!("Dictionary is not empty, printing to err.log");
        eprintln!("Keys: {:?}", dictionary.keys());
        serde_jsonlines::write_json_lines(&PathBuf::from("err.log"), &dictionary)?;
        // eprintln!("{:?}", dictionary);
    }
    eprintln!("Finished reading the file");
    serde_jsonlines::write_json_lines(&options.output, &output_vec)?;
    Ok(())
}
