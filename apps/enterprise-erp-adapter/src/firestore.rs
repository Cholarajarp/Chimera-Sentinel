use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};

pub struct FirestoreClient {
    client: Client,
    project_id: String,
    database_id: String,
}

impl FirestoreClient {
    pub fn new(project_id: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap(),
            project_id,
            database_id: "(default)".to_string(),
        }
    }

    pub async fn fetch_token(&self) -> Result<String, String> {
        let resp = self
            .client
            .get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
            .header("Metadata-Flavor", "Google")
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;

        if let Ok(r) = resp {
            if r.status().is_success() {
                let body: Value = r.json().await.map_err(|e| e.to_string())?;
                if let Some(token) = body["access_token"].as_str() {
                    return Ok(token.to_string());
                }
            }
        }

        let output = std::process::Command::new("gcloud")
            .args(["auth", "application-default", "print-access-token"])
            .output()
            .map_err(|e| format!("Failed to run gcloud: {}", e))?;

        if output.status.success() {
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(token);
        }

        Err("Failed to fetch Google Cloud token".to_string())
    }

    fn base_url(&self) -> String {
        format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/{}/documents",
            self.project_id, self.database_id
        )
    }

    pub async fn upsert_doc(&self, path: &str, fields: Value, token: &str) -> Result<(), String> {
        let url = format!("{}/{}", self.base_url(), path);
        let body = json!({ "fields": fields });

        let resp = self
            .client
            .patch(&url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(format!("Firestore error: {}", err_body));
        }
        Ok(())
    }

    pub async fn get_doc(&self, path: &str, token: &str) -> Result<Option<Value>, String> {
        let url = format!("{}/{}", self.base_url(), path);

        let resp = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        match resp.status().as_u16() {
            200 => Ok(Some(resp.json().await.map_err(|e| e.to_string())?)),
            404 => Ok(None),
            _ => {
                let err_body = resp.text().await.unwrap_or_default();
                Err(format!("Firestore error: {}", err_body))
            }
        }
    }

    pub async fn list_docs(
        &self,
        collection_path: &str,
        token: &str,
    ) -> Result<Vec<Value>, String> {
        let url = format!("{}/{}", self.base_url(), collection_path);

        let resp = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(format!("Firestore error: {}", err_body));
        }

        let body: Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(body["documents"].as_array().cloned().unwrap_or_default())
    }

    #[allow(dead_code)]
    pub async fn delete_doc(&self, path: &str, token: &str) -> Result<(), String> {
        let url = format!("{}/{}", self.base_url(), path);
        let resp = self
            .client
            .delete(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() && resp.status().as_u16() != 404 {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(format!("Firestore error: {}", err_body));
        }
        Ok(())
    }
}

pub fn to_firestore_fields<T: Serialize>(value: &T) -> Result<Value, String> {
    let json_val = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let obj = json_val
        .as_object()
        .ok_or("Value must serialize to an object")?;
    let mut fields = serde_json::Map::new();
    for (k, v) in obj {
        fields.insert(k.clone(), encode_value(v));
    }
    Ok(Value::Object(fields))
}

pub fn from_firestore_doc<T: for<'de> Deserialize<'de>>(doc: &Value) -> Result<T, String> {
    let fields = doc.get("fields").ok_or("Document has no 'fields'")?;
    let fields_obj = fields.as_object().ok_or("Fields is not an object")?;
    let mut decoded = serde_json::Map::new();
    for (k, v) in fields_obj {
        decoded.insert(k.clone(), decode_value(v));
    }
    serde_json::from_value(Value::Object(decoded)).map_err(|e| e.to_string())
}

fn encode_value(v: &Value) -> Value {
    match v {
        Value::Null => json!({"nullValue": null}),
        Value::Bool(b) => json!({"booleanValue": b}),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                json!({"integerValue": i.to_string()})
            } else {
                json!({"doubleValue": n.as_f64().unwrap_or(0.0)})
            }
        }
        Value::String(s) => json!({"stringValue": s}),
        Value::Array(arr) => {
            let values: Vec<Value> = arr.iter().map(encode_value).collect();
            json!({"arrayValue": {"values": values}})
        }
        Value::Object(map) => {
            let mut fields = serde_json::Map::new();
            for (k, v) in map {
                fields.insert(k.clone(), encode_value(v));
            }
            json!({"mapValue": {"fields": fields}})
        }
    }
}

fn decode_value(fv: &Value) -> Value {
    if let Some(s) = fv.get("stringValue") {
        return s.clone();
    }
    if let Some(b) = fv.get("booleanValue") {
        return b.clone();
    }
    if let Some(i) = fv.get("integerValue") {
        if let Some(s) = i.as_str() {
            if let Ok(n) = s.parse::<i64>() {
                return Value::Number(n.into());
            }
        }
        return i.clone();
    }
    if let Some(d) = fv.get("doubleValue") {
        return d.clone();
    }
    if fv.get("nullValue").is_some() {
        return Value::Null;
    }
    if let Some(av) = fv.get("arrayValue") {
        let values = av["values"]
            .as_array()
            .map(|arr| arr.iter().map(decode_value).collect())
            .unwrap_or_default();
        return Value::Array(values);
    }
    if let Some(mv) = fv.get("mapValue") {
        if let Some(fields) = mv["fields"].as_object() {
            let mut decoded = serde_json::Map::new();
            for (k, v) in fields {
                decoded.insert(k.clone(), decode_value(v));
            }
            return Value::Object(decoded);
        }
    }
    Value::Null
}
