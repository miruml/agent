# \ClientsApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_client**](ClientsApi.md#create_client) | **POST** /clients | Create a client
[**delete_client**](ClientsApi.md#delete_client) | **DELETE** /clients/{client_id} | Delete a client
[**get_client**](ClientsApi.md#get_client) | **GET** /clients/{client_id} | Get a client
[**get_clients**](ClientsApi.md#get_clients) | **GET** /clients | List clients in a workspace
[**update_client**](ClientsApi.md#update_client) | **PUT** /clients/{client_id} | Update a client



## create_client

> models::Client create_client(workspace_id, create_client_request, expand_left_square_bracket_right_square_bracket)
Create a client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |
**create_client_request** | [**CreateClientRequest**](CreateClientRequest.md) |  | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ClientExpand>**](models::ClientExpand.md)> |  |  |

### Return type

[**models::Client**](Client.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_client

> models::Client delete_client(client_id, expand_left_square_bracket_right_square_bracket)
Delete a client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The unique identifier of the client | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ClientExpand>**](models::ClientExpand.md)> |  |  |

### Return type

[**models::Client**](Client.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_client

> models::Client get_client(client_id, expand_left_square_bracket_right_square_bracket)
Get a client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The unique identifier of the client | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ClientExpand>**](models::ClientExpand.md)> |  |  |

### Return type

[**models::Client**](Client.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_clients

> models::ClientList get_clients(workspace_id, offset, limit, order_by, expand_left_square_bracket_right_square_bracket, search)
List clients in a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |
**offset** | Option<**i32**> | The offset to begin returning results from |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return |  |[default to 10]
**order_by** | Option<[**Vec<models::ClientOrderBy>**](models::ClientOrderBy.md)> |  |  |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ClientExpand>**](models::ClientExpand.md)> |  |  |
**search** | Option<[**Vec<models::ClientSearch>**](models::ClientSearch.md)> |  |  |

### Return type

[**models::ClientList**](ClientList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_client

> models::Client update_client(client_id, update_client_request, expand_left_square_bracket_right_square_bracket)
Update a client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The unique identifier of the client | [required] |
**update_client_request** | [**UpdateClientRequest**](UpdateClientRequest.md) |  | [required] |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ClientExpand>**](models::ClientExpand.md)> |  |  |

### Return type

[**models::Client**](Client.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

