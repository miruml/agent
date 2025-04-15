# \OverridesApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_override**](OverridesApi.md#create_override) | **POST** /overrides | Create an override
[**delete_override**](OverridesApi.md#delete_override) | **DELETE** /overrides/{override_id} | Delete an override
[**get_override**](OverridesApi.md#get_override) | **GET** /overrides/{override_id} | Get an override
[**list_overrides**](OverridesApi.md#list_overrides) | **GET** /overrides | List the overrides in a workspace
[**render_override**](OverridesApi.md#render_override) | **POST** /overrides/{override_id}/render | Render an override
[**update_override**](OverridesApi.md#update_override) | **PUT** /overrides/{override_id} | Update an override



## create_override

> models::Override create_override(create_override_request, expand)
Create an override

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_override_request** | [**CreateOverrideRequest**](CreateOverrideRequest.md) |  | [required] |
**expand** | Option<[**Vec<models::OverrideExpand>**](models::OverrideExpand.md)> |  |  |

### Return type

[**models::Override**](Override.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_override

> models::Override delete_override(override_id, expand)
Delete an override

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**override_id** | **String** | The unique identifier of the override | [required] |
**expand** | Option<[**Vec<models::OverrideExpand>**](models::OverrideExpand.md)> |  |  |

### Return type

[**models::Override**](Override.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_override

> models::Override get_override(override_id, expand)
Get an override

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**override_id** | **String** | The unique identifier of the override | [required] |
**expand** | Option<[**Vec<models::OverrideExpand>**](models::OverrideExpand.md)> |  |  |

### Return type

[**models::Override**](Override.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_overrides

> models::OverrideList list_overrides(offset, limit, order_by, expand, search)
List the overrides in a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**i32**> | The offset to begin returning results from |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return |  |[default to 10]
**order_by** | Option<[**Vec<models::OverrideOrderBy>**](models::OverrideOrderBy.md)> |  |  |
**expand** | Option<[**Vec<models::OverrideExpand>**](models::OverrideExpand.md)> |  |  |
**search** | Option<[**Vec<models::OverrideSearch>**](models::OverrideSearch.md)> |  |  |

### Return type

[**models::OverrideList**](OverrideList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## render_override

> models::RenderedOverride render_override(override_id, render_override_request, rendered_expand)
Render an override

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**override_id** | **String** | The unique identifier of the override | [required] |
**render_override_request** | [**RenderOverrideRequest**](RenderOverrideRequest.md) |  | [required] |
**rendered_expand** | Option<[**Vec<models::RenderedOverrideExpand>**](models::RenderedOverrideExpand.md)> |  |  |

### Return type

[**models::RenderedOverride**](RenderedOverride.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_override

> models::Override update_override(override_id, update_override_request, expand)
Update an override

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**override_id** | **String** | The unique identifier of the override | [required] |
**update_override_request** | [**UpdateOverrideRequest**](UpdateOverrideRequest.md) |  | [required] |
**expand** | Option<[**Vec<models::OverrideExpand>**](models::OverrideExpand.md)> |  |  |

### Return type

[**models::Override**](Override.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

