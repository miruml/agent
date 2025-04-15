# \TagsApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_tag**](TagsApi.md#create_tag) | **POST** /tags | Create a tag
[**delete_tag**](TagsApi.md#delete_tag) | **DELETE** /tags/{tag_id} | Delete a tag
[**get_tag**](TagsApi.md#get_tag) | **GET** /tags/{tag_id} | Get a tag
[**get_tags**](TagsApi.md#get_tags) | **GET** /tags | List the tags in a workspace
[**update_tag**](TagsApi.md#update_tag) | **PUT** /tags/{tag_id} | Update a tag



## create_tag

> models::Tag create_tag(tag_type_id, create_tag_request, expand_left_square_bracket_right_square_bracket)
Create a tag

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tag_type_id** | **String** | The unique identifier of the tag type | [required] |
**create_tag_request** | [**CreateTagRequest**](CreateTagRequest.md) |  | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::TagExpand>**](models::TagExpand.md)> |  |  |

### Return type

[**models::Tag**](Tag.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_tag

> models::Tag delete_tag(tag_id, expand_left_square_bracket_right_square_bracket)
Delete a tag

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tag_id** | **String** | The unique identifier of the tag | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::TagExpand>**](models::TagExpand.md)> |  |  |

### Return type

[**models::Tag**](Tag.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tag

> models::Tag get_tag(tag_id, expand_left_square_bracket_right_square_bracket)
Get a tag

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tag_id** | **String** | The unique identifier of the tag | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::TagExpand>**](models::TagExpand.md)> |  |  |

### Return type

[**models::Tag**](Tag.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tags

> models::TagList get_tags(workspace_id, offset, limit, order_by, expand_left_square_bracket_right_square_bracket, search)
List the tags in a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |
**offset** | Option<**i32**> | The offset to begin returning results from |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return |  |[default to 10]
**order_by** | Option<[**Vec<models::TagOrderBy>**](models::TagOrderBy.md)> |  |  |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::TagExpand>**](models::TagExpand.md)> |  |  |
**search** | Option<[**Vec<models::TagSearch>**](models::TagSearch.md)> |  |  |

### Return type

[**models::TagList**](TagList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_tag

> models::Tag update_tag(tag_id, update_tag_request, expand_left_square_bracket_right_square_bracket)
Update a tag

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tag_id** | **String** | The unique identifier of the tag | [required] |
**update_tag_request** | [**UpdateTagRequest**](UpdateTagRequest.md) |  | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::TagExpand>**](models::TagExpand.md)> |  |  |

### Return type

[**models::Tag**](Tag.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

