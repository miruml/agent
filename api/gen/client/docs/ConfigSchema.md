# ConfigSchema

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** |  | 
**version** | **i32** |  | 
**digest** | **String** |  | 
**created_at** | **String** |  | 
**created_by_id** | Option<**String**> |  | 
**config_type_id** | **String** |  | 
**created_by** | Option<[**models::User**](User.md)> |  | 
**schema** | Option<[**serde_json::Value**](.md)> |  | 
**config_type** | Option<[**models::ConfigType**](ConfigType.md)> |  | 
**config_schema_git_commits** | Option<[**models::ConfigSchemaGitCommitList**](ConfigSchemaGitCommitList.md)> |  | 
**overrides** | Option<[**models::OverrideList**](OverrideList.md)> |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


