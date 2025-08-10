use serde::Serialize;
use serde_repr::Serialize_repr;

#[derive(Clone)]
pub struct PiShockClient {
    username: String,
    api_key: String,
    share_code: String,
    client: reqwest::Client,
}

#[derive(Serialize_repr)]
#[repr(u8)]
enum OperationType {
    Shock = 0,
    Vibrate = 1,
    Beep = 2,
}

#[derive(Serialize)]
struct Command {
    #[serde(rename = "Op")]
    op: OperationType,
    #[serde(rename = "Duration")]
    duration: u8,
    #[serde(rename = "Intensity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    intensity: Option<u8>,
}

#[derive(Serialize)]
struct CommandRequest<'a> {
    #[serde(rename = "Username")]
    username: &'a str,
    #[serde(rename = "Apikey")]
    api_key: &'a str,
    #[serde(rename = "Code")]
    code: &'a str,
    #[serde(rename = "Name")]
    name: &'a str,

    #[serde(flatten)]
    command: Command,
}

const API_URL: &str = "https://do.pishock.com/api/apioperate/";

impl PiShockClient {
    pub fn new(username: String, api_key: String, share_code: String) -> Self {
        Self {
            username,
            api_key,
            share_code,
            client: reqwest::Client::new(),
        }
    }

    pub async fn shock(&self, intensity: u8, duration: u8) -> reqwest::Result<()> {
        let result = self
            .make_request(Command {
                op: OperationType::Shock,
                duration,
                intensity: Some(intensity),
            })
            .await?;
        println!("Shock Result: {result}");
        Ok(())
    }

    pub async fn vibrate(&self, intensity: u8, duration: u8) -> reqwest::Result<()> {
        let result = self
            .make_request(Command {
                op: OperationType::Vibrate,
                duration,
                intensity: Some(intensity),
            })
            .await?;
        println!("Vibrate Result: {result}");
        Ok(())
    }

    pub async fn beep(&self, duration: u8) -> reqwest::Result<()> {
        let result = self
            .make_request(Command {
                op: OperationType::Beep,
                duration,
                intensity: None,
            })
            .await?;
        println!("Beep Result: {result}");
        Ok(())
    }

    async fn make_request(&self, command: Command) -> reqwest::Result<String> {
        let request = CommandRequest {
            username: &self.username,
            api_key: &self.api_key,
            code: &self.share_code,
            name: "Prompt or Shock",
            command,
        };
        self.client
            .post(API_URL)
            .json(&request)
            .send()
            .await?
            .text()
            .await
    }
}
