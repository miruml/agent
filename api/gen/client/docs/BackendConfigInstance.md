# BackendConfigInstance

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** |  | 
**target_status** | [**models::ConfigInstanceTargetStatus**](ConfigInstanceTargetStatus.md) |  | 
**status** | [**models::ConfigInstanceStatus**](ConfigInstanceStatus.md) |  | 
**activity_status** | [**models::ConfigInstanceActivityStatus**](ConfigInstanceActivityStatus.md) |  | 
**error_status** | [**models::ConfigInstanceErrorStatus**](ConfigInstanceErrorStatus.md) |  | 
**relative_filepath** | Option<**String**> |  | 
**patch_id** | Option<**String**> |  | 
**created_by_id** | Option<**String**> |  | 
**created_at** | **String** |  | 
**updated_by_id** | Option<**String**> |  | 
**updated_at** | **String** |  | 
**device_id** | **String** |  | 
**config_schema_id** | **String** |  | 
**instance** | Option<[**serde_json::Value**](.md)> |  | 
**created_by** | Option<[**models::User**](User.md)> |  | 
**updated_by** | Option<[**models::User**](User.md)> |  | 
**patch** | Option<[**models::Patch**](Patch.md)> |  | 
**config_schema** | Option<[**models::ConfigSchema**](ConfigSchema.md)> |  | 
**device** | Option<[**models::Device**](Device.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


