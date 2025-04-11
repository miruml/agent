# \ConcreteConfigsApi

All URIs are relative to *https://configs.dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_latest_concrete_config**](ConcreteConfigsApi.md#get_latest_concrete_config) | **GET** /concrete_configs/latest | Get the latest concrete config for a client



## get_latest_concrete_config

> models::BaseConcreteConfig get_latest_concrete_config(config_schema_digest, config_schema_slug)
Get the latest concrete config for a client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_schema_digest** | **String** | The digest of the config schema | [required] |
**config_schema_slug** | **String** | The slug of the config schema | [required] |

### Return type

[**models::BaseConcreteConfig**](BaseConcreteConfig.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

