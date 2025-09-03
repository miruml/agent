# ConfigType

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** | ID of the config type | 
**name** | **String** | Name of the config type | 
**slug** | **String** | An immutable, code-friendly name for the config type | 
**created_at** | **String** | Timestamp of when the config type was created | 
**updated_at** | **String** | Timestamp of when the config type was last updated | 
**custom_validation_enabled** | **bool** | Whether this config type requires custom validation for its config instances | 
**created_by_id** | **String** |  | 
**updated_by_id** | **String** |  | 
**created_by** | Option<[**models::Principal**](Principal.md)> |  | 
**updated_by** | Option<[**models::Principal**](Principal.md)> |  | 
**config_schemas** | Option<[**models::ConfigSchemaList**](ConfigSchemaList.md)> | Expand the config schemas using 'expand[]=config_schemas' in the query string | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


