# Artifact

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** |  | 
**status** | [**models::ArtifactStatus**](ArtifactStatus.md) |  | 
**digest** | **String** |  | 
**aarch64** | **bool** |  | 
**x86_64** | **bool** |  | 
**created_at** | **String** |  | 
**ready_at** | Option<**String**> |  | 
**failed_at** | Option<**String**> |  | 
**source_id** | **String** |  | 
**source_type** | [**models::ArtifactSourceType**](ArtifactSourceType.md) |  | 
**created_by** | Option<[**models::User**](User.md)> |  | 
**registry_source** | Option<[**models::RegistrySource**](RegistrySource.md)> |  | 
**github_source** | Option<[**models::GitHubSource**](GitHubSource.md)> |  | 
**github_source_data** | Option<[**models::GitHubSourceData**](GitHubSourceData.md)> |  | 
**images** | [**models::ImageList**](ImageList.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


