# \ConfigInstancesApi

All URIs are relative to *https://configs.dev.api.miruml.com/agent/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_config_instances**](ConfigInstancesApi.md#list_config_instances) | **GET** /config_instances | List config instances in a workspace
[**update_config_instance**](ConfigInstancesApi.md#update_config_instance) | **PATCH** /config_instances/{config_instance_id} | Update a config instance



## list_config_instances

> models::ConfigInstanceList list_config_instances(offset, limit, order_by, expand_left_square_bracket_right_square_bracket, search)
List config instances in a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**i32**> | The offset of the items to return. An offset of 10 with a limit of 10 returns items 11-20. |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return. A limit of 15 with an offset of 0 returns items 1-15. |  |[default to 10]
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

> models::ConfigInstance update_config_instance(config_instance_id, update_config_instance_request, expand_left_square_bracket_right_square_bracket)
Update a config instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_instance_id** | **String** | The unique identifier of the config instance | [required] |
**update_config_instance_request** | [**UpdateConfigInstanceRequest**](UpdateConfigInstanceRequest.md) |  | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConfigInstanceExpand>**](models::ConfigInstanceExpand.md)> |  |  |

### Return type

[**models::ConfigInstance**](ConfigInstance.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

