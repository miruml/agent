# Device

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** | ID of the device | 
**name** | **String** | Name of the device | 
**status** | [**models::DeviceStatus**](DeviceStatus.md) |  | 
**last_connected_at** | Option<**String**> | Timestamp of when the device was last made an initial connection (this is not the same as the last time the device was seen). | 
**last_disconnected_at** | Option<**String**> | Timestamp of when the device was last disconnected (this is not the same as the last time the device was seen). | 
**created_at** | **String** | Timestamp of when the device was created | 
**updated_at** | **String** | Timestamp of when the device was last updated | 
**session_id** | **String** | Session ID of the device | 
**created_by_id** | **String** |  | 
**updated_by_id** | **String** |  | 
**created_by** | Option<[**models::Principal**](Principal.md)> |  | 
**updated_by** | Option<[**models::Principal**](Principal.md)> |  | 
**device_tags** | Option<[**models::DeviceTagPaginatedList**](DeviceTagPaginatedList.md)> |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


