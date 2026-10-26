use serde::Serialize;

pub const PUBLIC_CONTRACT_SCHEMA: u32 = 1;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessReport<'a, T>
where
    T: Serialize,
{
    pub schema: u32,
    pub data: &'a T,
}

impl<'a, T> SuccessReport<'a, T>
where
    T: Serialize,
{
    pub fn new(data: &'a T) -> Self {
        Self {
            schema: PUBLIC_CONTRACT_SCHEMA,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SuccessReport, PUBLIC_CONTRACT_SCHEMA};

    #[test]
    fn success_report_has_stable_schema() {
        let value = 7_u32;
        let report = SuccessReport::new(&value);
        assert_eq!(report.schema, PUBLIC_CONTRACT_SCHEMA);

        let json = serde_json::to_value(report).unwrap();
        assert_eq!(json["schema"], PUBLIC_CONTRACT_SCHEMA);
        assert_eq!(json["data"], 7);
    }
}
