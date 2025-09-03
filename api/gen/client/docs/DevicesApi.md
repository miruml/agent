# \DevicesApi

All URIs are relative to *https://configs.dev.api.miruml.com/agent/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**activate_device**](DevicesApi.md#activate_device) | **POST** /devices/{device_id}/activate | Activate a device
[**issue_device_token**](DevicesApi.md#issue_device_token) | **POST** /devices/{device_id}/issue_token | Issue a device token
[**update_device_by_agent**](DevicesApi.md#update_device_by_agent) | **PATCH** /devices/{device_id} | Update a device by agent



## activate_device

> models::Device activate_device(device_id, activate_device_request)
Activate a device

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** | The unique identifier of the device | [required] |
**activate_device_request** | Option<[**ActivateDeviceRequest**](ActivateDeviceRequest.md)> |  |  |

### Return type

[**models::Device**](Device.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## issue_device_token

> models::TokenResponse issue_device_token(device_id, issue_device_token_request)
Issue a device token

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** | The unique identifier of the device | [required] |
**issue_device_token_request** | Option<[**IssueDeviceTokenRequest**](IssueDeviceTokenRequest.md)> |  |  |

### Return type

[**models::TokenResponse**](TokenResponse.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_device_by_agent

> models::Device update_device_by_agent(device_id, update_device_from_agent_request)
Update a device by agent

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** | The unique identifier of the device | [required] |
**update_device_from_agent_request** | [**UpdateDeviceFromAgentRequest**](UpdateDeviceFromAgentRequest.md) |  | [required] |

### Return type

[**models::Device**](Device.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

