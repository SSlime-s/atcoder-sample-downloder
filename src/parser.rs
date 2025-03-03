use itertools::Itertools;
use selectors::Element;

pub struct AtCoderParser {
  document: scraper::Html,
}

impl AtCoderParser {
  pub fn new(html: &str) -> AtCoderParser {
      AtCoderParser {
          document: scraper::Html::parse_document(html),
      }
  }
  pub fn sample_cases(&self) -> Option<Vec<(String, String)>> {
      let task_statement_selector =
          scraper::Selector::parse(r#"div[id="task-statement"]"#).unwrap();
      let pre_selector = scraper::Selector::parse("pre").unwrap();
      let h3_selector = scraper::Selector::parse("h3").unwrap();
      let input_h3_text = vec!["入力例", "Sample Input"];
      let output_h3_text = vec!["出力例", "Sample Output"];

      let mut input_cases = vec![];
      let mut output_cases = vec![];
      if let Some(task_statement) = self.document.select(&task_statement_selector).next() {
          for pre in task_statement.select(&pre_selector) {
              if let Some(pre_parent) = pre.parent_element() {
                  if let Some(h3) = pre_parent.select(&h3_selector).next() {
                      let h3_text = h3.text().collect::<String>();
                      let input = input_h3_text.iter().any(|&x| h3_text.contains(x));
                      let output = output_h3_text.iter().any(|&x| h3_text.contains(x));
                      let text = pre.text().collect::<String>();
                      if input {
                          input_cases.push(text);
                      } else if output {
                          output_cases.push(text);
                      }
                  }
              }
          }
      } else {
          return None;
      }
      // make cases unique to remove extra duplicated language cases
      let input_cases: Vec<String> = input_cases.into_iter().unique().collect();
      let output_cases: Vec<String> = output_cases.into_iter().unique().collect();
      let sample_test_cases: Vec<(String, String)> = input_cases
          .into_iter()
          .zip(output_cases)
          .map(|(input, output)| (input, output))
          .collect();
      Some(sample_test_cases)
  }

  pub fn csrf_token(&self) -> Option<String> {
      let selector = scraper::Selector::parse(r#"input[name="csrf_token"]"#).unwrap();
      if let Some(element) = self.document.select(&selector).next() {
          if let Some(token) = element.value().attr("value") {
              return Some(token.to_string());
          }
      }
      None
  }
}
