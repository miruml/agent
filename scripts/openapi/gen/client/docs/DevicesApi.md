# \DevicesApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_device**](DevicesApi.md#create_device) | **POST** /groups/{group_id}/devices | Create a device in a group
[**get_device**](DevicesApi.md#get_device) | **GET** /devices/{device_id} | Get a device by ID
[**get_device_jwt**](DevicesApi.md#get_device_jwt) | **GET** /devices/{device_id}/jwt | Get a JWT for a device
[**get_group_devices**](DevicesApi.md#get_group_devices) | **GET** /groups/{group_id}/devices | Get all devices for a group
[**get_workspace_devices**](DevicesApi.md#get_workspace_devices) | **GET** /workspaces/{workspace_id}/devices | Get all devices for a workspace
[**update_device**](DevicesApi.md#update_device) | **PATCH** /devices/{device_id} | Update a device by ID



## create_device

> models::Device create_device(group_id, create_device_request)
Create a device in a group

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The unique identifier of the group | [required] |
**create_device_request** | [**CreateDeviceRequest**](CreateDeviceRequest.md) |  | [required] |

### Return type

[**models::Device**](Device.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_device

> models::Device get_device(device_id)
Get a device by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** | The unique identifier of the device | [required] |

### Return type

[**models::Device**](Device.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_device_jwt

> models::Jwt get_device_jwt(device_id)
Get a JWT for a device

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** | The unique identifier of the device | [required] |

### Return type

[**models::Jwt**](JWT.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_group_devices

> models::GroupDeviceList get_group_devices(group_id)
Get all devices for a group

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**group_id** | **String** | The unique identifier of the group | [required] |

### Return type

[**models::GroupDeviceList**](GroupDeviceList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_workspace_devices

> models::Device get_workspace_devices(workspace_id)
Get all devices for a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |

### Return type

[**models::Device**](Device.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_device

> models::Device update_device(device_id, update_device_request)
Update a device by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** | The unique identifier of the device | [required] |
**update_device_request** | [**UpdateDeviceRequest**](UpdateDeviceRequest.md) |  | [required] |

### Return type

[**models::Device**](Device.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

