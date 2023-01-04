use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Deserialize, Serialize)]
pub struct Secret {
    #[serde(rename = "ARN")]
    pub arn: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "SecretString")]
    pub value: Option<String>,
}

impl Tabled for Secret {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<String> {
        let desc = match &self.description {
            Some(d) => d,
            None => "",
        };
        vec![self.name.clone(), desc.to_string()]
    }

    fn headers() -> Vec<String> {
        vec!["Name".to_string(), "Description".to_string()]
    }
}
