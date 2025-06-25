# \ConfigInstancesApi

All URIs are relative to *https://configs.dev.api.miruml.com/agent/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_latest_config_instance**](ConfigInstancesApi.md#get_latest_config_instance) | **GET** /config_instances/deployed | Get the latest config instance



## get_latest_config_instance

> models::BaseConfigInstance get_latest_config_instance(config_schema_digest, config_type_slug)
Get the latest config instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
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

