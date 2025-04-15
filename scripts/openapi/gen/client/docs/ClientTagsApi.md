# \ClientTagsApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_client_tags**](ClientTagsApi.md#create_client_tags) | **POST** /client_tags | Create client tags
[**delete_client_tags**](ClientTagsApi.md#delete_client_tags) | **POST** /client_tags/delete | Delete client tags



## create_client_tags

> models::ClientTagList create_client_tags(client_tags_request)
Create client tags

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_tags_request** | [**ClientTagsRequest**](ClientTagsRequest.md) |  | [required] |

### Return type

[**models::ClientTagList**](ClientTagList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_client_tags

> models::ClientTagList delete_client_tags(client_id, client_tags_request)
Delete client tags

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The unique identifier of the client | [required] |
**client_tags_request** | [**ClientTagsRequest**](ClientTagsRequest.md) |  | [required] |

### Return type

[**models::ClientTagList**](ClientTagList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

