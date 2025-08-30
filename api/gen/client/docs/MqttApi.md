# \MqttApi

All URIs are relative to *https://configs.dev.api.miruml.com/agent/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**device_ping**](MqttApi.md#device_ping) | **GET** /cmd/devices/{device_id}/ping | Ping a device
[**device_pong**](MqttApi.md#device_pong) | **GET** /resp/devices/{device_id}/pong | Ping device response
[**sync_device**](MqttApi.md#sync_device) | **GET** /cmd/devices/{device_id}/sync | Trigger a device sync



## device_ping

> models::Ping device_ping(device_id)
Ping a device

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** |  | [required] |

### Return type

[**models::Ping**](Ping.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## device_pong

> models::Pong device_pong(device_id)
Ping device response

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** |  | [required] |

### Return type

[**models::Pong**](Pong.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


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

