//! # `typed-dialogflow`
//!
//! An easy-to-use typed [Google Dialogflow](https://dialogflow.cloud.google.com/) client.

pub mod model;

use gcp_auth::AuthenticationManager;
use language_tags::LanguageTag;
use model::DetectIntentResponse;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;

const SCOPES: &[&str] = &["https://www.googleapis.com/auth/dialogflow"];

#[derive(Debug, thiserror::Error)]
pub enum DialogflowError {
    #[error("no GCP auth methods available: {0}")]
    MissingAuth(#[from] gcp_auth::Error),

    #[error("GCP token not available")]
    TokenNotAvailable,

    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("cannot deserialize JSON response")]
    ResponseNotDeserializable,
}

pub struct DetectIntentOptions {
    language_code: LanguageTag,
    geolocation: Option<(f32, f32)>,
}

impl Default for DetectIntentOptions {
    fn default() -> Self {
        Self {
            language_code: LanguageTag::parse("en").unwrap(),
            geolocation: None,
        }
    }
}

/// An authenticated Dialogflow client
pub struct Dialogflow {
    auth: AuthenticationManager,
    client: reqwest::Client,
    detect_intent_url: Url,
    options: DetectIntentOptions,
}

impl Dialogflow {
    /// Initializes Dialogflow
    ///
    /// Multiple strategies are used to authenticate the client, please refer to
    /// [`gcp_auth`][gcp_auth::AuthenticationManager::new] for more information.
    pub async fn new() -> Result<Self, DialogflowError> {
        let auth = gcp_auth::AuthenticationManager::new().await?;
        let project_id = auth.project_id().await?;

        Ok(Self {
            auth,
            client: reqwest::Client::new(),
            detect_intent_url: format!("https://dialogflow.googleapis.com/v2/projects/{project_id}/agent/sessions/dev:detectIntent").parse().unwrap(),
            options: Default::default(),
        })
    }

    pub fn with_detect_intent_options(
        mut self,
        detect_intent_options: DetectIntentOptions,
    ) -> Self {
        self.options = detect_intent_options;
        self
    }

    /// Detects an intent and returns the result as [`serde`]-deserialized enum, `I`
    ///
    /// The enum must be shaped as follows:
    ///   * The name of each enum variant should be the same as a Dialogflow intent name
    ///   * The variants may only be, either:
    ///       * A unit variant, for intents with no parameters
    ///       * A struct variant, whose fields correspond to paramter names
    ///
    /// ## Enum definition example
    ///
    /// ```
    /// #[derive(serde::Deserialize)]
    /// #[serde(rename_all = "snake_case")]
    /// enum Intent {
    ///     Hello,
    ///     Weather {
    ///         location: String,
    ///     },
    ///     ThankYou,
    /// }
    /// ```
    ///
    /// In this case, the intents are named `hello`, `weather` and `thank_you`.
    ///
    /// ## Call example
    ///
    /// ```
    /// # async fn f(dialogflow: typed_dialogflow::Dialogflow) {
    /// #[derive(Debug, Eq, PartialEq, serde::Deserialize)]
    /// #[serde(rename_all = "snake_case")]
    /// enum Intent {
    ///     Hello,
    ///     Goodbye,
    /// }
    ///
    /// let intent = dialogflow.detect_intent_serde::<Intent>("Hello !").await.unwrap();
    ///
    /// assert_eq!(intent, Intent::Hello);
    /// # }
    /// ```
    ///
    /// ## Unknown intent
    ///
    /// If the Dialogflow API cannot recognize an intent, this function will attempt to deserialize
    /// a variant called `unknown` on your enum. This allows you to know that the text wasn't
    /// recognized without having to deal with an [`Err`].
    pub async fn detect_intent_serde<I: DeserializeOwned>(
        &self,
        text: &str,
    ) -> Result<I, DialogflowError> {
        #[derive(serde::Serialize)]
        struct Request<'a> {
            query_input: QueryInput<'a>,
            query_params: QueryParams,
        }

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct QueryInput<'a> {
            text: QueryInputText<'a>,
        }

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct QueryInputText<'a> {
            language_code: &'a LanguageTag,
            text: &'a str,
        }

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct QueryParams {
            #[serde(skip_serializing_if = "Option::is_none")]
            geo_location: Option<GeoLocation>,
        }

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct GeoLocation<F: Serialize = f32> {
            latitude: F,
            longitude: F,
        }

        let req = Request {
            query_input: QueryInput {
                text: QueryInputText {
                    language_code: &self.options.language_code,
                    text,
                },
            },
            query_params: QueryParams {
                geo_location: self.options.geolocation.map(|g| GeoLocation {
                    latitude: g.0,
                    longitude: g.1,
                }),
            },
        };

        let token = self
            .auth
            .get_token(SCOPES)
            .await
            .map_err(|_| DialogflowError::TokenNotAvailable)?;

        let res = self
            .client
            .post(self.detect_intent_url.clone())
            .header("Authorization", format!("Bearer {}", token.as_str()))
            .json(&req)
            .send()
            .await?;

        let res: DetectIntentResponse = res
            .json()
            .await
            .map_err(|_| DialogflowError::ResponseNotDeserializable)?;

        I::deserialize(res).map_err(|_| DialogflowError::ResponseNotDeserializable)
    }
}
