# \ImagesApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_external_images**](ImagesApi.md#get_external_images) | **GET** /repositories/containers/{container_repository_id}/images/external | Get the external images for a container repository



## get_external_images

> models::ExternalContainerImageList get_external_images(container_repository_id)
Get the external images for a container repository

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**container_repository_id** | **String** | The unique identifier of the container repository | [required] |

### Return type

[**models::ExternalContainerImageList**](ExternalContainerImageList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

