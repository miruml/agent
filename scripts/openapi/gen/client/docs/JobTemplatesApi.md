# \JobTemplatesApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_job_template**](JobTemplatesApi.md#create_job_template) | **POST** /workspaces/{workspace_id}/jobs/templates | Create a job template
[**delete_job_template**](JobTemplatesApi.md#delete_job_template) | **DELETE** /jobs/templates/{job_template_id} | Delete a job template
[**get_job_template**](JobTemplatesApi.md#get_job_template) | **GET** /jobs/templates/{job_template_id} | Get a job template
[**get_job_templates**](JobTemplatesApi.md#get_job_templates) | **GET** /workspaces/{workspace_id}/jobs/templates | Get all job templates for a workspace
[**update_job_template**](JobTemplatesApi.md#update_job_template) | **PATCH** /jobs/templates/{job_template_id} | Update a job template



## create_job_template

> models::JobTemplate create_job_template(workspace_id, create_job_template_request)
Create a job template

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |
**create_job_template_request** | [**CreateJobTemplateRequest**](CreateJobTemplateRequest.md) |  | [required] |

### Return type

[**models::JobTemplate**](JobTemplate.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_job_template

> models::JobTemplate delete_job_template(job_template_id)
Delete a job template

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_template_id** | **String** | The unique identifier of the job template | [required] |

### Return type

[**models::JobTemplate**](JobTemplate.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_template

> models::JobTemplate get_job_template(job_template_id)
Get a job template

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_template_id** | **String** | The unique identifier of the job template | [required] |

### Return type

[**models::JobTemplate**](JobTemplate.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_templates

> models::BaseJobTemplateList get_job_templates(workspace_id)
Get all job templates for a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |

### Return type

[**models::BaseJobTemplateList**](BaseJobTemplateList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_job_template

> models::JobTemplate update_job_template(job_template_id, update_job_template_request)
Update a job template

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_template_id** | **String** | The unique identifier of the job template | [required] |
**update_job_template_request** | [**UpdateJobTemplateRequest**](UpdateJobTemplateRequest.md) |  | [required] |

### Return type

[**models::JobTemplate**](JobTemplate.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

