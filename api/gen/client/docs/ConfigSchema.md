# ConfigSchema

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** | ID of the config schema | 
**version** | **i32** | Config schema version for the config type | 
**digest** | **String** | Hash of the config schema contents | 
**relative_filepath** | **String** | The file path to deploy the config instance relative to `/srv/miru/config_instances`. `v1/motion-control.json` would deploy to `/srv/miru/config_instances/v1/motion-control.json` | 
**created_at** | **String** | Timestamp of when the config schema was created | 
**updated_at** | **String** | Timestamp of when the config schema was last updated | 
**config_type_id** | **String** | ID of the config type | 
**content** | Option<[**serde_json::Value**](.md)> | The config schema's JSON Schema definition | 
**created_by_id** | **String** |  | 
**updated_by_id** | **String** |  | 
**created_by** | Option<[**models::Principal**](Principal.md)> |  | 
**updated_by** | Option<[**models::Principal**](Principal.md)> |  | [optional]
**config_schema_git_commits** | Option<[**models::ConfigSchemaGitCommitList**](ConfigSchemaGitCommitList.md)> |  | 
**overrides** | Option<[**models::OverrideList**](OverrideList.md)> |  | 
**config_type** | Option<[**models::ConfigType**](ConfigType.md)> | Expand the config type using 'expand[]=config_type' in the query string | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


