# ConfigSchema

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** | ID of the config schema | 
**version** | **i32** | Version of the config schema | 
**digest** | **String** | Digest of the config schema | 
**relative_filepath** | **String** | The default file path to deploy the config instances of this config schema relative to /srv/miru/config_instances. v1/motion-control.json would deploy to /srv/miru/config_instances/v1/motion-control.json | 
**created_at** | **String** | Timestamp of when the config schema was created | 
**updated_at** | **String** | Timestamp of when the config schema was last updated | 
**config_type_id** | **String** | ID of the config type | 
**content** | Option<[**serde_json::Value**](.md)> | JSON schema contents | 
**config_type** | Option<[**models::ConfigType**](ConfigType.md)> | Expand the config type using 'expand[]=config_type' in the query string | 
**created_by_id** | Option<**String**> |  | [optional]
**updated_by_id** | Option<**String**> |  | [optional]
**created_by** | Option<[**models::User**](User.md)> |  | 
**updated_by** | Option<[**models::User**](User.md)> |  | [optional]
**config_schema_git_commits** | Option<[**models::ConfigSchemaGitCommitList**](ConfigSchemaGitCommitList.md)> |  | 
**overrides** | Option<[**models::OverrideList**](OverrideList.md)> |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


