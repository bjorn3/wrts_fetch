extern crate url;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::io::prelude::*;
use std::fs::File;
use url::Url;

#[derive(Debug, Deserialize)]
struct WrtsData {
    title: String,
    list_collection: ListCollection,
}

#[derive(Debug, Deserialize)]
struct ListCollection {
    lists: Vec<List>,
}

#[derive(Debug, Deserialize)]
struct List {
    words: Vec<Word>,
    subject: String,
}

#[derive(Debug, Deserialize)]
struct Word {
    word: String,
}

#[derive(Debug)]
struct OutData {
    title: String,
    subjects: [String; 2],
    words: Vec<[String; 2]>,
}

fn parse_list_url(url: &str) -> String {
    let url = Url::parse(&url).expect("failed to parse url");
    let fragment = url.fragment().expect("invalid url");
    let fragment = Url::options().base_url(Some(&Url::parse("http://example.com/").unwrap())).parse(&fragment).expect("failed to parse fragment");
    let fragment_segments = fragment.path_segments().unwrap();
    let id = fragment_segments.skip(1).next().unwrap();
    println!("id: {}", id);
    id.to_string()
}

fn fetch_and_parse_list(id: &str) -> WrtsData {
    let list_url = format!("https://wrts.nl/lists/{}.json", id);
    println!("List url: {}", list_url);

    let mut res = reqwest::get(&list_url).unwrap();
    assert!(res.status().is_success());

    let mut content = String::new();
    res.read_to_string(&mut content).unwrap();

    serde_json::from_str(&content).unwrap()
}

fn wrts_data_to_out_data(data: WrtsData) -> OutData {
    let lists = data.list_collection.lists;
    let mut subjects = [String::new(), String::new()];
    let mut words = lists[0].words.iter().map(|_|[String::new(), String::new()]).collect::<Vec<[String; 2]>>();
    for (list_id, list) in lists.into_iter().enumerate() {
        subjects[list_id] = list.subject;
        for (word_id, word) in list.words.into_iter().enumerate() {
            words[word_id][list_id] = word.word;
        }
    }

    OutData {
        title: data.title,
        subjects: subjects,
        words: words,
    }
}

fn format_openteacher_2_file(data: OutData) -> String {
    let mut out_xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n\
    <root>\n".to_string();
    out_xml.push_str(&format!(
    "    <title>{}</title>\n\
        <question_language>{}</question_language>\n\
        <answer_language>{}</answer_language>\n\
    ",
        data.title, data.subjects[1], data.subjects[0]));
    
    for word_pair in data.words {
        out_xml.push_str(&format!(
    "    <word>\n\
            <known>{}</known>\n\
            <foreign>{}</foreign>\n\
            <results>0/0</results>\n\
        </word>\n\
    ", word_pair[1], word_pair[0]));
    }

    out_xml.push_str("</root>");
    out_xml
}

fn main() {
    let url = std::env::args().skip(1).next().expect("No url given");
    println!("Url: {}", url);

    let id = parse_list_url(&url);
    let data = fetch_and_parse_list(&id);
    let title = data.title.clone();
    let out_data = wrts_data_to_out_data(data);

    println!("num words: {}", out_data.words.len());

    let out_xml = format_openteacher_2_file(out_data);
    
    //println!("==========\n{}", out_xml);
    
    let mut file = File::create(&format!("{}.ot", title)).unwrap();
    file.write_all(out_xml.as_bytes()).unwrap();
}

