# \ConcreteConfigsApi

All URIs are relative to *https://configs.dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**render_latest_concrete_config**](ConcreteConfigsApi.md#render_latest_concrete_config) | **POST** /concrete_configs/render_latest | Render the latest concrete config for a client



## render_latest_concrete_config

> models::BaseConcreteConfig render_latest_concrete_config(render_latest_concrete_config_request)
Render the latest concrete config for a client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**render_latest_concrete_config_request** | Option<[**RenderLatestConcreteConfigRequest**](RenderLatestConcreteConfigRequest.md)> |  |  |

### Return type

[**models::BaseConcreteConfig**](BaseConcreteConfig.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

