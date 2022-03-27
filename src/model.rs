use serde::de::*;
use serde_json::Value as JsonValue;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectIntentResponse {
    query_result: QueryResult,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryResult {
    parameters: JsonValue,

    #[serde(default)]
    intent: Option<Intent>,

    #[serde(default)]
    pub intent_detection_confidence: Option<f64>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Intent {
    display_name: String,
}

impl<'de, 'a> Deserializer<'de> for DetectIntentResponse {
    type Error = serde::de::value::Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::custom("this deserializer only supports enums"))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct map struct ignored_any identifier
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }
}

impl<'de> EnumAccess<'de> for DetectIntentResponse {
    type Error = serde::de::value::Error;
    type Variant = Parameters;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(<V as DeserializeSeed<'de>>::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let intent_name = match self.query_result.intent {
            Some(i) => seed.deserialize(i.display_name.into_deserializer()),
            None => seed.deserialize("unknown".into_deserializer()),
        }?;

        Ok((intent_name, Parameters(self.query_result.parameters)))
    }
}

pub struct Parameters(JsonValue);

impl<'de, 'a> VariantAccess<'de> for Parameters {
    type Error = serde::de::value::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(
        self,
        _seed: T,
    ) -> Result<<T as DeserializeSeed<'de>>::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        unimplemented!()
    }

    fn tuple_variant<V>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0
            .deserialize_struct("", fields, visitor)
            .map_err(|err| serde::de::value::Error::custom(err))
    }
}
