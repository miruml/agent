# BaseConfigSchema

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** | ID of the config schema | 
**version** | **i32** | Version of the config schema | 
**digest** | **String** | Digest of the config schema | 
**created_at** | **String** | Timestamp of when the config schema was created | 
**config_type_id** | **String** | ID of the config type | 
**content** | Option<[**serde_json::Value**](.md)> | JSON schema contents | 
**config_type** | Option<[**models::ConfigType**](ConfigType.md)> | Expand the config type using 'expand[]=config_type' in the query string | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


