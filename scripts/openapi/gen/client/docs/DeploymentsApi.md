# \DeploymentsApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_deployments**](DeploymentsApi.md#create_deployments) | **POST** /deployments | Create deployments
[**download_deployments**](DeploymentsApi.md#download_deployments) | **PATCH** /deployments/download | Set the target status of the deployments to be downloaded
[**get_artifact_deployments**](DeploymentsApi.md#get_artifact_deployments) | **GET** /artifacts/{artifact_id}/deployments | Get the deployments from an artifact
[**get_deployment**](DeploymentsApi.md#get_deployment) | **GET** /deployments/{deployment_id} | Get a deployment by ID
[**get_deployment_logs**](DeploymentsApi.md#get_deployment_logs) | **GET** /deployments/{deployment_id}/logs | Get the logs for a deployment
[**get_deployments**](DeploymentsApi.md#get_deployments) | **GET** /devices/{device_id}/deployments | Get the deployments for a device
[**remove_deployments**](DeploymentsApi.md#remove_deployments) | **PATCH** /deployments/remove | Set the target status of the deployments to be removed
[**start_deployments**](DeploymentsApi.md#start_deployments) | **PATCH** /deployments/start | Set the target status of the deployments to be running
[**stop_deployments**](DeploymentsApi.md#stop_deployments) | **PATCH** /deployments/stop | Set the target status of the deployments to be stopped



## create_deployments

> models::BaseDeploymentList create_deployments(create_deployments_request)
Create deployments

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_deployments_request** | [**CreateDeploymentsRequest**](CreateDeploymentsRequest.md) |  | [required] |

### Return type

[**models::BaseDeploymentList**](BaseDeploymentList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## download_deployments

> models::BaseDeploymentList download_deployments(deployment_id, download_deployments_request)
Set the target status of the deployments to be downloaded

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**deployment_id** | **String** | The unique identifier of the deployment | [required] |
**download_deployments_request** | [**DownloadDeploymentsRequest**](DownloadDeploymentsRequest.md) |  | [required] |

### Return type

[**models::BaseDeploymentList**](BaseDeploymentList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_artifact_deployments

> models::BaseDeploymentList get_artifact_deployments(artifact_id)
Get the deployments from an artifact

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**artifact_id** | **String** | The unique identifier of the artifact | [required] |

### Return type

[**models::BaseDeploymentList**](BaseDeploymentList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_deployment

> models::Deployment get_deployment(deployment_id)
Get a deployment by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**deployment_id** | **String** | The unique identifier of the deployment | [required] |

### Return type

[**models::Deployment**](Deployment.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_deployment_logs

> models::DeploymentLogList get_deployment_logs(deployment_id)
Get the logs for a deployment

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**deployment_id** | **String** | The unique identifier of the deployment | [required] |

### Return type

[**models::DeploymentLogList**](DeploymentLogList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_deployments

> models::DeploymentList get_deployments(device_id, on_device)
Get the deployments for a device

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**device_id** | **String** | The unique identifier of the device | [required] |
**on_device** | Option<**bool**> | Whether to include only on device deployments |  |

### Return type

[**models::DeploymentList**](DeploymentList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## remove_deployments

> models::BaseDeploymentList remove_deployments(deployment_id, remove_deployments_request)
Set the target status of the deployments to be removed

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**deployment_id** | **String** | The unique identifier of the deployment | [required] |
**remove_deployments_request** | [**RemoveDeploymentsRequest**](RemoveDeploymentsRequest.md) |  | [required] |

### Return type

[**models::BaseDeploymentList**](BaseDeploymentList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## start_deployments

> models::BaseDeploymentList start_deployments(deployment_id, start_deployments_request)
Set the target status of the deployments to be running

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**deployment_id** | **String** | The unique identifier of the deployment | [required] |
**start_deployments_request** | [**StartDeploymentsRequest**](StartDeploymentsRequest.md) |  | [required] |

### Return type

[**models::BaseDeploymentList**](BaseDeploymentList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## stop_deployments

> models::BaseDeploymentList stop_deployments(deployment_id, stop_deployments_request)
Set the target status of the deployments to be stopped

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**deployment_id** | **String** | The unique identifier of the deployment | [required] |
**stop_deployments_request** | [**StopDeploymentsRequest**](StopDeploymentsRequest.md) |  | [required] |

### Return type

[**models::BaseDeploymentList**](BaseDeploymentList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

