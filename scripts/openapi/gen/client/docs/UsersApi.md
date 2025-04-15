# \UsersApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_user**](UsersApi.md#get_user) | **GET** /user | Get a user
[**update_user**](UsersApi.md#update_user) | **PUT** /user | Update a user



## get_user

> models::User get_user(expand)
Get a user

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**expand** | Option<[**Vec<models::UserExpand>**](models::UserExpand.md)> |  |  |

### Return type

[**models::User**](User.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user

> models::User update_user(update_user_request, expand)
Update a user

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**update_user_request** | [**UpdateUserRequest**](UpdateUserRequest.md) |  | [required] |
**expand** | Option<[**Vec<models::UserExpand>**](models::UserExpand.md)> |  |  |

### Return type

[**models::User**](User.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

