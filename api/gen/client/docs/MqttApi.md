# \MqttApi

All URIs are relative to *https://configs.dev.api.miruml.com/agent/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**sync_device**](MqttApi.md#sync_device) | **GET** /cmd/devices/{device_id}/sync | Trigger a device sync



## sync_device

> models::SyncDevice sync_device(device_id)
Trigger a device sync

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** |  | [required] |

### Return type

[**models::SyncDevice**](SyncDevice.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

