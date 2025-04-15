# Deployment

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**id** | **String** |  | 
**device_id** | **String** |  | 
**status** | [**models::DeploymentStatus**](DeploymentStatus.md) |  | 
**activity_status** | [**models::DeploymentActivityStatus**](DeploymentActivityStatus.md) |  | 
**error_status** | [**models::DeploymentErrorStatus**](DeploymentErrorStatus.md) |  | 
**target_status** | [**models::DeploymentTargetStatus**](DeploymentTargetStatus.md) |  | 
**created_at** | **String** |  | 
**downloading_at** | Option<**String**> |  | 
**downloaded_at** | Option<**String**> |  | 
**booting_at** | Option<**String**> |  | 
**active_at** | Option<**String**> |  | 
**stopping_at** | Option<**String**> |  | 
**stopped_at** | Option<**String**> |  | 
**removing_at** | Option<**String**> |  | 
**archived_at** | Option<**String**> |  | 
**cooldown_ends_at** | **String** |  | 
**created_by** | Option<[**models::User**](User.md)> |  | 
**artifact** | [**models::ArtifactWithSourceData**](ArtifactWithSourceData.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


