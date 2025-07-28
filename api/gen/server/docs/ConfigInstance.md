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
**content** | [**serde_json::Value**](.md) | The configuration values associated with the config instance | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


