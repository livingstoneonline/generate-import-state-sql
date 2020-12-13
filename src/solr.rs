use crate::Result;
use quick_xml::events::attributes::Attribute;
use quick_xml::{events::BytesStart, events::Event, Reader};
use std::collections::HashMap;

use std::io::BufRead;

trait Rows {
    type Row: ToString;

    fn rows(items: Vec<Self::Row>) -> String {
        let items = items
            .into_iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>();
        items.join(",\n")
    }

    fn sql(items: Vec<Self::Row>) -> String;
}

struct Object {
    pid: String,
    content_model: String,
    private: bool,
    md5: String,
}

impl Object {
    fn new(pid: String, content_model: String, private: bool, md5: String) -> Self {
        Self {
            pid,
            content_model,
            private,
            md5,
        }
    }

    fn r#type(&self) -> &'static str {
        match self.content_model.as_str() {
            "islandora:manuscriptCModel" => "manuscript",
            "islandora:manuscriptPageCModel" => "manuscript_page",
            "islandora:sp_pdf" => "manuscript_additional_pdf",
            "livingstone:spectralManuscriptCModel" => "spectral_manuscript",
            "livingstone:spectralManuscriptPageCModel" => "spectral_manuscript_page",
            "islandora:sp_large_image_cmodel" => {
                if self.pid.ends_with("_noCrop") {
                    "no_crop"
                } else {
                    "illustrative"
                }
            }
            _ => panic!("Unknown content model encountered"),
        }
    }
}

impl ToString for Object {
    fn to_string(&self) -> String {
        format!(
            "('{pid}', '{model}', {private}, '{type}', '{md5}')",
            pid = self.pid,
            model = self.content_model,
            private = if self.private { 1 } else { 0 },
            r#type = self.r#type(),
            md5 = self.md5
        )
    }
}

impl Rows for Object {
    type Row = Self;

    fn sql(items: Vec<Self>) -> String {
        let rows = Self::rows(items);
        format!(
            r#"
TRUNCATE TABLE livingstone_fedora_local_objects;
INSERT INTO livingstone_fedora_local_objects (pid, content_model, private, type, md5)
VALUES
{};

"#,
            rows
        )
    }
}

struct Datastream {
    pid: String,
    dsid: String,
    md5: String,
}

impl Datastream {
    fn new(pid: String, dsid: String, md5: String) -> Self {
        Self { pid, dsid, md5 }
    }
}

impl ToString for Datastream {
    fn to_string(&self) -> String {
        format!(
            "('{pid}', '{dsid}', '{md5}')",
            pid = self.pid,
            dsid = self.dsid,
            md5 = self.md5
        )
    }
}

impl Rows for Datastream {
    type Row = Self;

    fn sql(items: Vec<Self>) -> String {
        let rows = Self::rows(items);
        format!(
            r#"
TRUNCATE TABLE livingstone_fedora_local_datastreams;
INSERT INTO livingstone_fedora_local_datastreams (pid, dsid, md5)
VALUES
{};

"#,
            rows
        )
    }
}

fn get_attribute<'a>(element: &'a BytesStart, name: &[u8]) -> Option<Attribute<'a>> {
    let mut attributes = element.attributes().filter_map(|x| x.ok());
    attributes.find(|attribute| attribute.key == name)
}

fn get_text<B>(reader: &mut Reader<B>) -> String
where
    B: BufRead,
{
    let mut buffer = Vec::new();
    loop {
        let event = reader.read_event(&mut buffer).unwrap();
        if let Event::Text(e) = event {
            let bytes = &e.unescaped().unwrap();
            let s = std::str::from_utf8(bytes).unwrap().to_string();
            if !s.trim().is_empty() {
                return s;
            }
        } else if let Event::Eof = event {
            panic!("Prevent infinite loop. This should never be reached.");
        }
    }
}

fn get_array_text<B>(reader: &mut Reader<B>) -> Result<String>
where
    B: BufRead,
{
    let mut buffer = Vec::new();
    loop {
        match reader.read_event(&mut buffer)? {
            Event::Start(element) | Event::Empty(element) => {
                if element.local_name() == b"str" {
                    return Ok(get_text(reader));
                }
            }
            Event::Eof => {
                panic!("Prevent infinite loop. This should never be reached.");
            }
            _ => (),
        }
    }
}

fn doc<B>(reader: &mut Reader<B>) -> Result<(Object, Vec<Datastream>)>
where
    B: BufRead,
{
    let mut buffer = Vec::new();
    let mut pid = None;
    let mut content_model = None;
    let mut private = None;
    let mut md5 = None;
    let mut datastreams = HashMap::new(); // Map of DSID to MD5.
    loop {
        match reader.read_event(&mut buffer)? {
            Event::Start(element) | Event::Empty(element) => {
                let name = get_attribute(&element, b"name").ok_or("Failed to get name of field")?;
                match name.value.as_ref() {
                    b"PID" => {
                        pid = Some(get_text(reader));
                    }
                    b"checksum_s" => {
                        md5 = Some(get_text(reader));
                    }
                    b"hidden_b" => {
                        let hidden: bool = get_text(reader).parse()?;
                        private = Some(hidden)
                    }
                    b"RELS_EXT_hasModel_uri_s" => {
                        content_model = Some(
                            get_text(reader)
                                .strip_prefix("info:fedora/")
                                .unwrap()
                                .to_string(),
                        );
                    }
                    // Otherwise must be a datastream checksum.
                    _ => {
                        //fedora_datastream_latest_DC_MD5_ms
                        if name.value.ends_with(b"MD5_ms") {
                            let dsid = std::str::from_utf8(&name.value)?
                                .strip_prefix("fedora_datastream_latest_")
                                .unwrap()
                                .strip_suffix("_MD5_ms")
                                .unwrap()
                                .to_string();
                            datastreams.insert(dsid, get_array_text(reader)?);
                        }
                    }
                }
            }
            // Exit function if we've finished with the "doc", element they do not nest.
            Event::End(element) => {
                if element.name() == b"doc" {
                    break;
                }
            }
            Event::Eof => break,
            // We ignore Comments, CData, XML Declaration,
            // Processing Instructions, and DocType elements, etc.
            _ => (),
        };
    }
    let pid = pid.unwrap();
    let content_model = content_model.unwrap();
    let private = private.unwrap();
    let md5 = md5.unwrap();
    let datastreams = datastreams
        .into_iter()
        .map(|(dsid, md5)| Datastream::new(pid.clone(), dsid, md5))
        .collect::<Vec<_>>();
    Ok((Object::new(pid, content_model, private, md5), datastreams))
}

fn parse_response<B>(mut reader: Reader<B>) -> Result<(Vec<Object>, Vec<Datastream>)>
where
    B: BufRead,
{
    let mut objects: Vec<Object> = vec![];
    let mut datastreams: Vec<Datastream> = vec![];
    let mut buffer = Vec::new();
    loop {
        match reader.read_event(&mut buffer)? {
            Event::Start(element) | Event::Empty(element) => {
                if let b"doc" = element.name() {
                    let (object, object_datastreams) = doc(&mut reader)?;
                    objects.push(object);
                    datastreams.extend(object_datastreams);
                }
            }
            Event::Eof => break,
            // We ignore Comments, CData, XML Declaration,
            // Processing Instructions, and DocType elements, etc.
            _ => (),
        };
        // We have to clone to pass the data to the script so no point in maintaining reference to the string content.
        buffer.clear();
    }
    Ok((objects, datastreams))
}

pub async fn generate_sql(server: &str) -> Result<String> {
    let response = reqwest::get(server).await?;
    let response = response.text().await?;
    let reader = Reader::from_str(response.as_str());
    let (objects, datastreams) = parse_response(reader)?;
    Ok(Object::sql(objects) + Datastream::sql(datastreams).as_str())
}
