use std::{
    collections::{BTreeMap, VecDeque},
    f32::consts::E,
    io::Write,
    path::PathBuf,
};

use clap::Parser;
use quick_xml::{events::Event, name::QName};

#[derive(clap::Parser)]
struct Options {
    #[arg(
        short,
        long,
        default_value = "/projects/CLIENTS/q-2024-006/analysis/sampling/biosamples.out.xml"
        // default_value = "/projects/CLIENTS/q-2024-006/analysis/sampling/sample.xml"
    )]
    pub input_file: PathBuf,
    #[arg(short, long, default_value = "out.ndjson")]
    pub output: PathBuf,
}

fn main() {
    run(Options::parse()).unwrap()
}

fn to_key(stack: &VecDeque<Vec<u8>>) -> String {
    let stacks: Vec<String> = stack
        .iter()
        .map(|x| String::from_utf8_lossy(x).to_string())
        .collect();

    stacks.join(".")
}

fn run(options: Options) -> anyhow::Result<()> {
    println!("Hello, world!");
    let mut f = quick_xml::Reader::from_file(&options.input_file)?;
    let mut buf = Vec::new();
    let mut stack = VecDeque::new();
    let mut output_vec = Vec::new();
    let mut dictionary = BTreeMap::new();
    loop {
        match f.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", f.buffer_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::DocType(_) | Event::CData(_) | Event::PI(_)) => {}
            Ok(Event::Start(e)) => stack.push_back(e.to_owned().name().0.to_vec()),
            Ok(Event::End(e)) => {
                let popped_e = stack.pop_back().expect("End event without start");
                let start_word = popped_e;
                let end_word = e.name().0;
                assert_eq!(start_word, end_word, "Start and end words do not match");
                if end_word == b"EXPERIMENT_PACKAGE" {
                    // eprintln!("Finished a sample");
                    // prepare an ndjson line
                    output_vec.push(dictionary.clone());
                    dictionary.clear();
                }
                // if stack.len() == 1{

                //     eprintln!("Length: {}", stack.len());
                //     eprintln!("Remaining: {:?}", String::from_utf8_lossy(&stack[0]).to_string());
                // }
                // eprintln!("Event: {:?} popped: {:?}", e.clone(), popped_e.clone());
                // assert_eq!(e, popped_e);
            }
            Ok(Event::Text(mut e)) => {
                if e.inplace_trim_end() || e.inplace_trim_start() {
                    continue;
                }
                let k = to_key(&stack);
                let entry = dictionary.entry(k).or_insert(Vec::new());
                // if entry.len() > 0 {
                //     eprintln!("Duplicate key: {}", k);
                // }
                entry.push(e.unescape().unwrap().to_string());
                // dictionary
                //     .entry(k)
                //     .or_insert(Vec::new())
                //     .push(e.unescape().unwrap().to_string());
            }
            Ok(_event) => {
                // eprintln!("Event: {:?}", event);
            }
        }
        buf.clear();
    }
    eprintln!("Finished reading the file");
    serde_jsonlines::write_json_lines(&options.output, &output_vec)?;
    // let writer = std::io::BufWriter::new(std::fs::File::create(&options.output)?);
    // // let value =
    // serde_json::to_writer(writer, &dictionary)?;
    // writer.flush();
    Ok(())
}

#[cfg(test)]
mod test {
    const INPUT_FILE: &str = "/projects/CLIENTS/q-2024-006/analysis/sampling/biosamples.out.xml";
}
