# \ConfigInstancesApi

All URIs are relative to *https://configs.dev.api.miruml.com/agent/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_latest_config_instance**](ConfigInstancesApi.md#get_latest_config_instance) | **GET** /config_instances/latest | Get the latest config instance
[**refresh_latest_config_instance**](ConfigInstancesApi.md#refresh_latest_config_instance) | **POST** /config_instances/refresh_latest | Render the latest config instance for a device



## get_latest_config_instance

> models::BaseConfigInstance get_latest_config_instance(device_id, config_schema_digest, config_type_slug)
Get the latest config instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** | The unique identifier of the device | [required] |
**config_schema_digest** | **String** | The digest of the config schema | [required] |
**config_type_slug** | **String** | The slug of the config type | [required] |

### Return type

[**models::BaseConfigInstance**](BaseConfigInstance.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## refresh_latest_config_instance

> models::BaseConfigInstance refresh_latest_config_instance(refresh_latest_config_instance_request)
Render the latest config instance for a device

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**refresh_latest_config_instance_request** | Option<[**RefreshLatestConfigInstanceRequest**](RefreshLatestConfigInstanceRequest.md)> |  |  |

### Return type

[**models::BaseConfigInstance**](BaseConfigInstance.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

