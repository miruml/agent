# BaseConfigInstance

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** | ID of the config instance | 
**target_status** | [**models::ConfigInstanceTargetStatus**](ConfigInstanceTargetStatus.md) |  | 
**status** | [**models::ConfigInstanceStatus**](ConfigInstanceStatus.md) |  | 
**activity_status** | [**models::ConfigInstanceActivityStatus**](ConfigInstanceActivityStatus.md) |  | 
**error_status** | [**models::ConfigInstanceErrorStatus**](ConfigInstanceErrorStatus.md) |  | 
**relative_filepath** | Option<**String**> | The file path to deploy the config instance relative to /srv/miru/config_instances. v1/motion-control.json would deploy to /srv/miru/config_instances/v1/motion-control.json | 
**created_at** | **String** | The timestamp when the config instance was created | 
**updated_at** | **String** | The timestamp when the config instance was last updated | 
**config_schema_id** | **String** | The ID of the config schema which the config instance must adhere to | 
**config_type_id** | **String** | The ID of the config type of the config instance  | 
**config_type** | Option<[**models::ConfigType**](ConfigType.md)> | Expand the config type using 'expand[]=config_type' in the query string | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


