use game_tracker_lib::{api::RawgClient, models::error::AppError};
use mockito::Server;

#[tokio::test]
async fn search_return_results() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/games")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
        "results": [
            {
                "id": 1,
                "name": "Ninja Gaiden",
                "released": "2005-09-20",
                "rating": 4.5,
                "metacritic": 95,
                "platforms": [{"platform": {"id": 1, "name": "Xbox"}}],
                "genres": [{"id": 1, "name": "Action"}],
                "background_image": "https://example.com/img.jpg",
                "esrb_rating": {"name": "Everyone"} 
            }
        ]
    }"#,
        )
        .create_async()
        .await;

    let client = RawgClient::with_base_url("test_key".to_string(), server.url());
    let results = client.search_games("ninja_gaiden").await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Ninja Gaiden");
    assert_eq!(results[0].released, Some("2005-09-20".to_string()));
    mock.assert_async().await;
}

#[tokio::test]
async fn search_empty_query_rejected() {
    let client = RawgClient::with_base_url("test_key".to_string(), "http://unused".to_string());
    let result = client.search_games("").await;

    assert!(matches!(result, Err(AppError::ValidationError(_))));
}

#[tokio::test]
async fn search_api_error_returns_unavailable() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/games")
        .match_query(mockito::Matcher::Any)
        .with_status(500)
        .create_async()
        .await;

    let client = RawgClient::with_base_url("test_key".to_string(), server.url());
    let result = client.search_games("test").await;

    assert!(matches!(result, Err(AppError::ApiUnavailable(_))));
    mock.assert_async().await;
}

#[tokio::test]
async fn search_empty_results() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/games")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"results": []}"#)
        .create_async()
        .await;

    let client = RawgClient::with_base_url("test_key".to_string(), server.url());
    let results = client.search_games("nonexistent_game_xyz").await.unwrap();

    assert_eq!(results.len(), 0);
    mock.assert_async().await;
}
