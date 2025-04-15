# \ConfigsApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_config**](ConfigsApi.md#create_config) | **POST** /configs | Create a config
[**delete_config**](ConfigsApi.md#delete_config) | **DELETE** /configs/{config_id} | Delete a config
[**get_config**](ConfigsApi.md#get_config) | **GET** /configs/{config_id} | Get a config
[**get_configs**](ConfigsApi.md#get_configs) | **GET** /configs | List the configs in a workspace
[**update_config**](ConfigsApi.md#update_config) | **PUT** /configs/{config_id} | Update a config



## create_config

> models::Config create_config(workspace_id, create_config_request, expand_left_square_bracket_right_square_bracket)
Create a config

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |
**create_config_request** | [**CreateConfigRequest**](CreateConfigRequest.md) |  | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConfigExpand>**](models::ConfigExpand.md)> |  |  |

### Return type

[**models::Config**](Config.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_config

> models::Config delete_config(config_id, expand_left_square_bracket_right_square_bracket)
Delete a config

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_id** | **String** | The unique identifier of the config | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConfigExpand>**](models::ConfigExpand.md)> |  |  |

### Return type

[**models::Config**](Config.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_config

> models::Config get_config(config_id, expand_left_square_bracket_right_square_bracket)
Get a config

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_id** | **String** | The unique identifier of the config | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConfigExpand>**](models::ConfigExpand.md)> |  |  |

### Return type

[**models::Config**](Config.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_configs

> models::ConfigList get_configs(workspace_id, offset, limit, order_by, expand_left_square_bracket_right_square_bracket, search)
List the configs in a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |
**offset** | Option<**i32**> | The offset to begin returning results from |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return |  |[default to 10]
**order_by** | Option<[**Vec<models::ConfigOrderBy>**](models::ConfigOrderBy.md)> |  |  |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConfigExpand>**](models::ConfigExpand.md)> |  |  |
**search** | Option<[**Vec<models::ConfigSearch>**](models::ConfigSearch.md)> |  |  |

### Return type

[**models::ConfigList**](ConfigList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_config

> models::Config update_config(config_id, update_config_request, expand_left_square_bracket_right_square_bracket)
Update a config

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_id** | **String** | The unique identifier of the config | [required] |
**update_config_request** | [**UpdateConfigRequest**](UpdateConfigRequest.md) |  | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConfigExpand>**](models::ConfigExpand.md)> |  |  |

### Return type

[**models::Config**](Config.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

