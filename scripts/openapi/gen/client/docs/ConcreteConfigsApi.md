# \ConcreteConfigsApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_concrete_config**](ConcreteConfigsApi.md#get_concrete_config) | **GET** /concrete_configs/{concrete_config_id} | Get a concrete config
[**list_concrete_configs**](ConcreteConfigsApi.md#list_concrete_configs) | **GET** /concrete_configs | List concrete configs in a workspace
[**render_latest_concrete_config**](ConcreteConfigsApi.md#render_latest_concrete_config) | **POST** /concrete_configs/render_latest | Render the latest concrete config for a client



## get_concrete_config

> models::BackendConcreteConfig get_concrete_config(concrete_config_id, expand_left_square_bracket_right_square_bracket)
Get a concrete config

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**concrete_config_id** | **String** | The unique identifier of the concrete config | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConcreteConfigExpand>**](models::ConcreteConfigExpand.md)> |  |  |

### Return type

[**models::BackendConcreteConfig**](BackendConcreteConfig.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_concrete_configs

> models::ConcreteConfigList list_concrete_configs(offset, limit, order_by, expand_left_square_bracket_right_square_bracket, search)
List concrete configs in a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**i32**> | The offset to begin returning results from |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return |  |[default to 10]
**order_by** | Option<[**Vec<models::ConcreteConfigOrderBy>**](models::ConcreteConfigOrderBy.md)> |  |  |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConcreteConfigExpand>**](models::ConcreteConfigExpand.md)> |  |  |
**search** | Option<[**Vec<models::ConcreteConfigSearch>**](models::ConcreteConfigSearch.md)> |  |  |

### Return type

[**models::ConcreteConfigList**](ConcreteConfigList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## render_latest_concrete_config

> models::BackendConcreteConfig render_latest_concrete_config(render_latest_concrete_config_request)
Render the latest concrete config for a client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**render_latest_concrete_config_request** | Option<[**RenderLatestConcreteConfigRequest**](RenderLatestConcreteConfigRequest.md)> |  |  |

### Return type

[**models::BackendConcreteConfig**](BackendConcreteConfig.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

