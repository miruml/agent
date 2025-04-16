# \TagTypesApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_tag_type**](TagTypesApi.md#create_tag_type) | **POST** /tag_types | Create a tag type
[**delete_tag_type**](TagTypesApi.md#delete_tag_type) | **DELETE** /tag_types/{tag_type_id} | Delete a tag type
[**get_tag_type**](TagTypesApi.md#get_tag_type) | **GET** /tag_types/{tag_type_id} | Get a tag type
[**get_tag_types**](TagTypesApi.md#get_tag_types) | **GET** /tag_types | List the tag types in a workspace
[**update_tag_type**](TagTypesApi.md#update_tag_type) | **PUT** /tag_types/{tag_type_id} | Update a tag type



## create_tag_type

> models::TagType create_tag_type(workspace_id, create_tag_type_request, expand_left_square_bracket_right_square_bracket)
Create a tag type

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |
**create_tag_type_request** | [**CreateTagTypeRequest**](CreateTagTypeRequest.md) |  | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::TagTypeExpand>**](models::TagTypeExpand.md)> |  |  |

### Return type

[**models::TagType**](TagType.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_tag_type

> models::TagType delete_tag_type(tag_type_id, expand_left_square_bracket_right_square_bracket, can_delete_tags, can_set_override_key_value_pairs_null)
Delete a tag type

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tag_type_id** | **String** | The unique identifier of the tag type | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::TagTypeExpand>**](models::TagTypeExpand.md)> |  |  |
**can_delete_tags** | Option<**bool**> |  |  |[default to false]
**can_set_override_key_value_pairs_null** | Option<**bool**> |  |  |[default to false]

### Return type

[**models::TagType**](TagType.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tag_type

> models::TagType get_tag_type(tag_type_id, expand_left_square_bracket_right_square_bracket)
Get a tag type

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tag_type_id** | **String** | The unique identifier of the tag type | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::TagTypeExpand>**](models::TagTypeExpand.md)> |  |  |

### Return type

[**models::TagType**](TagType.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tag_types

> models::TagTypeList get_tag_types(offset, limit, order_by, expand_left_square_bracket_right_square_bracket, search)
List the tag types in a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**i32**> | The offset to begin returning results from |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return |  |[default to 10]
**order_by** | Option<[**Vec<models::TagTypeOrderBy>**](models::TagTypeOrderBy.md)> |  |  |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::TagTypeExpand>**](models::TagTypeExpand.md)> |  |  |
**search** | Option<[**Vec<models::TagTypeSearch>**](models::TagTypeSearch.md)> |  |  |

### Return type

[**models::TagTypeList**](TagTypeList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_tag_type

> models::TagType update_tag_type(tag_type_id, update_tag_type_request, expand_left_square_bracket_right_square_bracket)
Update a tag type

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tag_type_id** | **String** | The unique identifier of the tag type | [required] |
**update_tag_type_request** | [**UpdateTagTypeRequest**](UpdateTagTypeRequest.md) |  | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::TagTypeExpand>**](models::TagTypeExpand.md)> |  |  |

### Return type

[**models::TagType**](TagType.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

