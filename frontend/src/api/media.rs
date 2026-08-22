use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaItem {
    pub url: String,
    pub name: String,
}

#[server(ListMedia, "/api")]
pub async fn list_media(token: String) -> Result<Vec<MediaItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::api::auth::ssr_utils::verify_token;
        verify_token(&token)?;
        use google_cloud_storage::client::{Client, ClientConfig};
        use google_cloud_storage::http::objects::list::ListObjectsRequest;

        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to load GCS auth config: {}", e)))?;
        let client = Client::new(config);

        let request = ListObjectsRequest {
            bucket: "jakewray-portfolio".to_string(),
            prefix: Some("media/journalism/".to_string()),
            ..Default::default()
        };

        let response = client
            .list_objects(&request)
            .await
            .map_err(|e| ServerFnError::new(format!("GCS list objects failed: {}", e)))?;

        let mut items = Vec::new();
        let base_url = "https://storage.googleapis.com/jakewray-portfolio";

        if let Some(objects) = response.items {
            for object in objects {
                let name = object
                    .name
                    .split('/')
                    .next_back()
                    .unwrap_or(&object.name)
                    .to_string();
                if name.is_empty() {
                    continue; // Skip directory placeholders
                }
                items.push(MediaItem {
                    url: format!("{}/{}", base_url, object.name),
                    name,
                });
            }
        }

        Ok(items)
    }

    #[cfg(not(feature = "ssr"))]
    Ok(Vec::new())
}

#[server(UploadMedia, "/api")]
pub async fn upload_media(
    token: String,
    filename: String,
    data: Vec<u8>,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::api::auth::ssr_utils::verify_token;
        verify_token(&token)?;

        // We'll upload to a 'uploads' folder for manual picking or sorting later
        let filtered_name: String = filename
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .collect();

        if filtered_name.is_empty() {
            return Err(ServerFnError::new("Invalid filename"));
        }

        let timestamp = chrono::Utc::now().timestamp();
        let safe_name = format!("{}_{}", timestamp, filtered_name);
        use google_cloud_storage::client::{Client, ClientConfig};
        use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};

        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to load GCS auth config: {}", e)))?;
        let client = Client::new(config);

        let ext = filtered_name
            .split('.')
            .next_back()
            .unwrap_or("")
            .to_lowercase();
        let content_type = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "mp4" => "video/mp4",
            _ => "application/octet-stream",
        };

        let upload_type = UploadType::Simple(Media {
            name: format!("media/journalism/uploads/{}", safe_name).into(),
            content_length: Some(data.len() as u64),
            content_type: content_type.to_string().into(),
        });

        let request = UploadObjectRequest {
            bucket: "jakewray-portfolio".to_string(),
            ..Default::default()
        };

        // GCS upload_object takes &UploadObjectRequest, Body (Vec<u8>), and &UploadType
        client
            .upload_object(&request, data, &upload_type)
            .await
            .map_err(|e| ServerFnError::new(format!("GCS upload failed: {}", e)))?;

        Ok(format!(
            "https://storage.googleapis.com/jakewray-portfolio/media/journalism/uploads/{}",
            safe_name
        ))
    }

    #[cfg(not(feature = "ssr"))]
    Ok(String::new())
}

#[server(DeleteMedia, "/api")]
pub async fn delete_media(token: String, object_name: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::api::auth::ssr_utils::verify_token;
        verify_token(&token)?;
        use google_cloud_storage::client::{Client, ClientConfig};
        use google_cloud_storage::http::objects::delete::DeleteObjectRequest;

        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to load GCS auth config: {}", e)))?;
        let client = Client::new(config);

        // Safety check to prevent deleting things outside of the public uploads directory
        if !object_name.starts_with("media/journalism/") {
            return Err(ServerFnError::new("Unauthorized directory access"));
        }

        let request = DeleteObjectRequest {
            bucket: "jakewray-portfolio".to_string(),
            object: object_name,
            ..Default::default()
        };

        client
            .delete_object(&request)
            .await
            .map_err(|e| ServerFnError::new(format!("GCS delete failed: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    Ok(())
}
