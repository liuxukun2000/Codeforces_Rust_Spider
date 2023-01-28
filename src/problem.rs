use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::format;
use std::iter::zip;
use lazy_static::lazy_static;
use scraper::{ElementRef, Html, Selector};
use scraper::element_ref::Select;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use reqwest::header::HeaderMap;
use serde_json::Value;


lazy_static! {
    static ref DOC_SELECTOR: Selector = Selector::parse("div.problem-statement").unwrap();
    static ref TITLE_SELECTOR: Selector = Selector::parse("div.title").unwrap();
    static ref TIME_SELECTOR: Selector = Selector::parse("div.time-limit").unwrap();
    static ref MEMORY_SELECTOR: Selector = Selector::parse("div.memory-limit").unwrap();
    static ref CONTENT_SELECTOR: Selector = Selector::parse("p").unwrap();
    static ref INPUT_SELECTOR: Selector = Selector::parse("div.input-specification").unwrap();
    static ref OUTPUT_SELECTOR: Selector = Selector::parse("div.output-specification").unwrap();
    static ref NOTE_SELECTOR: Selector = Selector::parse("div.note").unwrap();
    static ref DATA_TITLE_SELECTOR: Selector = Selector::parse("div.section-title").unwrap();
    static ref DATA_INPUT_SELECTOR: Selector = Selector::parse("div.input").unwrap();
    static ref DATA_OUTPUT_SELECTOR: Selector = Selector::parse("div.output").unwrap();
    static ref TAG_SELECTOR: Selector = Selector::parse("span.tag-box").unwrap();
    static ref CONTEST_NAME_SELECTOR: Selector = Selector::parse("#sidebar > div:nth-child(1) > table > tbody > tr:nth-child(1) > th > a").unwrap();
    static ref CSRF_SELECTOR: Selector = Selector::parse(r#"meta[name="X-Csrf-Token"]"#).unwrap();
    static ref SUBMIT_ID_SELECTOR: Selector = Selector::parse("a.view-source").unwrap();
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Problem {
    pub title: String,
    pub content: String,
    pub input_specification: String,
    pub output_specification: String,
    pub note: String,
    pub time_limit: i32,
    pub memory_limit: i32,
    pub sample_data: Vec<Data>,
    pub data: Vec<Data>,
    pub tags: Vec<String>,
    pub metadata: Contest,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Contest {
    pub id: i32,
    pub name: String,
    pub index: String
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Data {
    pub title: String,
    pub input: String,
    pub output: String
}

impl Problem {
    pub async fn get_content(element: Select<'_, '_>) -> String {
        let mut content = String::new();
        for p in element {
            for i in p.children() {
                if i.value().is_text() {
                    content = content + &i.value().as_text().unwrap();
                }
                if i.value().is_element() {
                    content = format!("{}**{}** ", &content, &ElementRef::wrap(i).unwrap().inner_html());
                }
            }
            content = format!("{}\n\n", content);
        }
        content.replace("$$$", "$$")
    }

    pub fn get_limit(mut element: Select<'_, '_>) -> i32 {
        if let Some(element) = element.next() {
            return element.inner_html()
                    .split("</div>").last().unwrap_or(" 1")
                    .split_ascii_whitespace().next().unwrap_or("1")
                    .to_string().parse::<i32>().unwrap_or(1)
        }
        1
    }

    pub fn get_testcase(element: &ElementRef) -> Vec<Data> {
        let titles = element.select(&DATA_TITLE_SELECTOR);
        let inputs = element.select(&DATA_INPUT_SELECTOR);
        let outputs = element.select(&DATA_OUTPUT_SELECTOR);

        zip(titles, zip(inputs, outputs))
            .map(|(title, (input, output))|
                Data{
                    title: title.inner_html(),
                    input: ElementRef::wrap(input.last_child().unwrap()).unwrap()
                        .inner_html().replace("\r\n", "\n"),
                    output: ElementRef::wrap(output.last_child().unwrap()).unwrap()
                        .inner_html().replace("\r\n", "\n"),
                }).collect()
    }

    pub async fn get_problem(problem_set: i32, problem_id: &String) -> Result<Self, String> {
        let y = problem_id.clone();
        let data_handle = tokio::spawn(async move {
                Problem::get_data(problem_set, &y).await
        });
        let url = format!("https://codeforces.com/problemset/problem/{}/{}", problem_set, problem_id);
        let resp = reqwest::get(url).await.unwrap();
        let body = resp.text().await.unwrap();
        let mut prob = Problem::default();
        let doc = Html::parse_fragment(&body);
        prob.metadata = Contest{
                id: problem_set,
                name: doc.select(&CONTEST_NAME_SELECTOR).next().unwrap().inner_html(),
                index: problem_id.clone(),
            };
        prob.tags = doc.select(&TAG_SELECTOR).map(|x|x.inner_html().trim().to_string()).collect();

        let doc = doc.select(&DOC_SELECTOR).next();
        if let Some(doc) = doc {
            prob.title = doc.select(&TITLE_SELECTOR).next().unwrap().inner_html();
            prob.time_limit  = Problem::get_limit(doc.select(&TITLE_SELECTOR));
            prob.memory_limit = Problem::get_limit(doc.select(&MEMORY_SELECTOR));
            prob.content = Problem::get_content(doc.select(&CONTENT_SELECTOR)).await;
            prob.input_specification = Problem::get_content(doc.select(&INPUT_SELECTOR)).await;
            prob.output_specification = Problem::get_content(doc.select(&OUTPUT_SELECTOR)).await;
            prob.note = Problem::get_content(doc.select(&NOTE_SELECTOR)).await;
            prob.sample_data = Problem::get_testcase(&doc);
            prob.data = data_handle.await.unwrap();
        }
        else {
            return Err("Not found!".to_string());
        }
        return Ok(prob);
    }

    pub fn save(&self, path: &String, convert: bool) -> Result<(), String> {
        if convert == true {
            if fs::write(Path::new(&path), &self.convert()).is_ok() {
                return Ok(());
            }
        }
        else {
            if let Ok(content) = toml::to_string(&self) {
                if fs::write(Path::new(&path), &content).is_ok() {
                    return Ok(());
                }
            }
        }

        Err("Can not save problem!".to_string())
    }

    pub fn load(path: &String) -> Result<Self, ()> {
        let path = PathBuf::from(path);
        if path.is_file() {

            return Ok(toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap_or_default());
        }
        Err(())
    }

    pub fn convert(&self) -> String {
        let mut data = String::new();
        for i in &self.sample_data {
            data = format!("{}\n\n```iodata-in:Input-out:Output-no:{}\n{}<|==|>\n{}```\n",
                           &data, &i.title, &i.input, &i.output
            )
        }
        format!("# {}\n\n{}{}{}{}{}",
            &self.title,
            &self.content,
            &self.input_specification,
            &self.output_specification,
            &data,
            &self.note
        )
    }

    pub async fn get_token_and_submit_id(problem_set: i32, problem_id: &String) -> (String, String) {
        let url = format!("https://codeforces.com/problemset/status/{}/problem/{}", problem_set, problem_id);
        let resp = reqwest::get(url).await.unwrap();
        let body = resp.text().await.unwrap();
        let doc = Html::parse_document(&body);
        let token = doc.select(&CSRF_SELECTOR).next().unwrap()
            .value().attr("content").unwrap().to_string();
        let submit_id = doc.select(&SUBMIT_ID_SELECTOR).next().unwrap()
            .inner_html();
        (token, submit_id)
    }

    pub async fn get_data(problem_set: i32, problem_id: &String) -> Vec<Data> {
        let (token, submit_id) = Problem::get_token_and_submit_id(problem_set, &problem_id).await;
        let client = reqwest::Client::new();
        let data = client.post("https://codeforces.com/data/submitSource")
            .header("x-csrf-token", &token)
            .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36 Edg/109.0.1518.70")
            .form(&[("submissionId", &submit_id), ("csrf_token", &token)]).send().await.unwrap();
        let data = data.json::<HashMap<String, Value>>().await.unwrap();
        let mut index = 1;
        let mut ans = vec![];
        loop {
            let try_input = data.get(&format!("answer#{}", index));
            if try_input.is_none() { break; }
            ans.push(
              Data {
                  title: format!("Test_{}", index),
                  input: try_input.unwrap().to_string().replace("\\r\\n", "\n")
                      .strip_suffix("\"").unwrap()
                      .strip_prefix("\"").unwrap()
                      .to_string(),
                  output: data.get(&format!("input#{}", index)).unwrap()
                      .to_string().replace("\\r\\n", "\n")
                      .strip_suffix("\"").unwrap()
                      .strip_prefix("\"").unwrap()
                      .to_string(),
              }
            );
            index += 1;
        }
        ans
    }

    pub fn to_jcoder_formant(&self, path: &String) -> Result<(), ()> {
        let path = PathBuf::from(path);
        if path.exists() == false { fs::create_dir(&path).unwrap(); }
        if path.is_dir() {
            fs::write(&path.join(Path::new(&format!("{}.md", &self.title))), &self.convert()).unwrap();
            fs::create_dir(&path.join(Path::new("static/")));
            fs::create_dir(&path.join(Path::new("attach/")));
            let path = path.join(Path::new("answer/"));
            fs::create_dir(&path);

            for data in &self.data {
                fs::write(&path.join(Path::new(&format!("{}.in", &data.title))), &data.input).unwrap();
                fs::write(&path.join(Path::new(&format!("{}.out", &data.title))), &data.output).unwrap();
            }
            return Ok(());
        }
        Err(())
    }
}