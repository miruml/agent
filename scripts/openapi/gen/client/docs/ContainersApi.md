# \ContainersApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_containers**](ContainersApi.md#get_containers) | **GET** /devices/{device_id}/containers | Get the containers for a device
[**remove_container**](ContainersApi.md#remove_container) | **PATCH** /containers/{container_id}/remove | Remove a container
[**restart_container**](ContainersApi.md#restart_container) | **PATCH** /containers/{container_id}/restart | Restart a container
[**start_container**](ContainersApi.md#start_container) | **PATCH** /containers/{container_id}/start | Start a container
[**stop_container**](ContainersApi.md#stop_container) | **PATCH** /containers/{container_id}/stop | Stop a container



## get_containers

> models::ContainerList get_containers(device_id)
Get the containers for a device

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** | The unique identifier of the device | [required] |

### Return type

[**models::ContainerList**](ContainerList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## remove_container

> models::Container remove_container(container_id)
Remove a container

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**container_id** | **String** | The unique identifier of the container | [required] |

### Return type

[**models::Container**](Container.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## restart_container

> models::Container restart_container(container_id)
Restart a container

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**container_id** | **String** | The unique identifier of the container | [required] |

### Return type

[**models::Container**](Container.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## start_container

> models::Container start_container(container_id)
Start a container

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**container_id** | **String** | The unique identifier of the container | [required] |

### Return type

[**models::Container**](Container.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## stop_container

> models::Container stop_container(container_id)
Stop a container

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**container_id** | **String** | The unique identifier of the container | [required] |

### Return type

[**models::Container**](Container.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

