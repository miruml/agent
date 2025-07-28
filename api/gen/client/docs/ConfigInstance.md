# ConfigInstance

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** | ID of the config instance | 
**target_status** | [**models::ConfigInstanceTargetStatus**](ConfigInstanceTargetStatus.md) |  | 
**activity_status** | [**models::ConfigInstanceActivityStatus**](ConfigInstanceActivityStatus.md) |  | 
**error_status** | [**models::ConfigInstanceErrorStatus**](ConfigInstanceErrorStatus.md) |  | 
**status** | [**models::ConfigInstanceStatus**](ConfigInstanceStatus.md) |  | 
**relative_filepath** | **String** | The file path to deploy the config instance relative to `/srv/miru/config_instances`. `v1/motion-control.json` would deploy to `/srv/miru/config_instances/v1/motion-control.json` | 
**created_at** | **String** | The timestamp of when the config instance was created | 
**updated_at** | **String** | The timestamp of when the config instance was last updated | 
**device_id** | **String** | ID of the device which the config instance is deployed to | 
**config_schema_id** | **String** | ID of the config schema which the config instance must adhere to | 
**config_type_id** | **String** | ID of the config type which the config instance (and its schema) is a part of | 
**created_by_id** | **String** |  | 
**updated_by_id** | **String** |  | 
**patch_id** | Option<**String**> |  | 
**created_by** | Option<[**models::Principal**](Principal.md)> |  | 
**updated_by** | Option<[**models::Principal**](Principal.md)> |  | 
**patch** | Option<[**models::Patch**](Patch.md)> |  | 
**device** | Option<[**models::Device**](Device.md)> |  | 
**config_schema** | Option<[**models::ConfigSchema**](ConfigSchema.md)> | Expand the config schema using 'expand[]=config_schema' in the query string | 
**config_type** | Option<[**models::ConfigType**](ConfigType.md)> | Expand the config type using 'expand[]=config_type' in the query string | 
**content** | Option<[**serde_json::Value**](.md)> | The configuration values associated with the config instance | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


