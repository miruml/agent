# \ConfigInstancesApi

All URIs are relative to *https://configs.dev.api.miruml.com/agent/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_config_instance**](ConfigInstancesApi.md#create_config_instance) | **POST** /config_instances | Create a config instance
[**get_config_instance**](ConfigInstancesApi.md#get_config_instance) | **GET** /config_instances/{config_instance_id} | Get a config instance
[**list_config_instances**](ConfigInstancesApi.md#list_config_instances) | **GET** /config_instances | List config instances in a workspace
[**update_config_instance**](ConfigInstancesApi.md#update_config_instance) | **PATCH** /config_instances/{config_instance_id} | Update a config instance



## create_config_instance

> models::BackendConfigInstance create_config_instance(create_config_instance_request)
Create a config instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_config_instance_request** | [**CreateConfigInstanceRequest**](CreateConfigInstanceRequest.md) |  | [required] |

### Return type

[**models::BackendConfigInstance**](BackendConfigInstance.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_config_instance

> models::BackendConfigInstance get_config_instance(config_instance_id, expand_left_square_bracket_right_square_bracket)
Get a config instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_instance_id** | **String** | The unique identifier of the config instance | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConfigInstanceExpand>**](models::ConfigInstanceExpand.md)> |  |  |

### Return type

[**models::BackendConfigInstance**](BackendConfigInstance.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_config_instances

> models::ConfigInstanceList list_config_instances(offset, limit, order_by, expand_left_square_bracket_right_square_bracket, search)
List config instances in a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**i32**> | The offset to begin returning results from |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return |  |[default to 10]
**order_by** | Option<[**Vec<models::ConfigInstanceOrderBy>**](models::ConfigInstanceOrderBy.md)> |  |  |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConfigInstanceExpand>**](models::ConfigInstanceExpand.md)> |  |  |
**search** | Option<[**Vec<models::ConfigInstanceSearch>**](models::ConfigInstanceSearch.md)> |  |  |

### Return type

[**models::ConfigInstanceList**](ConfigInstanceList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_config_instance

> models::BackendConfigInstance update_config_instance(config_instance_id, update_config_instance_request, expand_left_square_bracket_right_square_bracket)
Update a config instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_instance_id** | **String** | The unique identifier of the config instance | [required] |
**update_config_instance_request** | [**UpdateConfigInstanceRequest**](UpdateConfigInstanceRequest.md) |  | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConfigInstanceExpand>**](models::ConfigInstanceExpand.md)> |  |  |

### Return type

[**models::BackendConfigInstance**](BackendConfigInstance.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

