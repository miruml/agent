# \ConcreteConfigsApi

All URIs are relative to *https://configs.dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_latest_concrete_config**](ConcreteConfigsApi.md#get_latest_concrete_config) | **GET** /concrete_configs/latest | Get the latest concrete config
[**refresh_latest_concrete_config**](ConcreteConfigsApi.md#refresh_latest_concrete_config) | **POST** /concrete_configs/refresh_latest | Render the latest concrete config for a client



## get_latest_concrete_config

> models::BaseConcreteConfig get_latest_concrete_config(client_id, config_schema_digest, config_slug)
Get the latest concrete config

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The unique identifier of the client | [required] |
**config_schema_digest** | **String** | The digest of the config schema | [required] |
**config_slug** | **String** | The slug of the config | [required] |

### Return type

[**models::BaseConcreteConfig**](BaseConcreteConfig.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## refresh_latest_concrete_config

> models::BaseConcreteConfig refresh_latest_concrete_config(refresh_latest_concrete_config_request)
Render the latest concrete config for a client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**refresh_latest_concrete_config_request** | Option<[**RefreshLatestConcreteConfigRequest**](RefreshLatestConcreteConfigRequest.md)> |  |  |

### Return type

[**models::BaseConcreteConfig**](BaseConcreteConfig.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

