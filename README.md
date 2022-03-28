# typed-dialogflow

An easy-to-use typed [Google Dialogflow](https://dialogflow.cloud.google.com/) client for Rust

_This library was extracted from [a personal project](https://github.com/WartaPoirier-corp/vakanssbot). As a result, it may still be very basic and quite rigid. However, I'm all for this library growing and becoming more capable._

## Example

The concept of this library is to provide a relatively type-safe interface to Dialogflow. Intents are modelled as a single Rust `enum` that you can pattern-match on.

Take the following Dialogflow intents:

![A screenshot of the Dialogflow dashboard with 3 intents named "hello", "weather" and "thank_you"](https://user-images.githubusercontent.com/46636609/160357511-0f5f8283-574b-402d-82a4-25a5ee23cdf1.png)

You can then query and deserialize intents with this code :

```rust
#[derive(Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Intent {
  Hello,
  Weather {
    location: String,
  },
  ThankYou,
}

let intent = dialogflow.detect_intent_serde::<Intent>("What's the weather like in Antarctica ?").await.unwrap();

match intent {
  Intent::Weather { location } => println!("The user asked for the weather in/at {location}"),
  ...
}
```

## Testing

Due to the private / proprietary / externally-hosted nature of Dialogflow, testing this library isn't as easy as any other library. In addition to that, because Dialogflow relies on AI, there's always a small probability that a text may be interpreted differently if the model is re-trained, making tests undeterministic. I could eventually try to make the Google Backend mockable to test the deserialization part of the library. Besides that, I'm running tests locally on a private Dialogflow model, but having tests run on only one computer makes it impossible to enforce rules about having the code pass tests before being commited.

## License

`MIT OR APACHE-2.0`
