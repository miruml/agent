// standard library
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
// internal crates
use crate::models::{
    artifact::Artifact,
    container::ContainerTargetStatus,
    deployment::Deployment,
    image::Image,
    job::Job,
    job_run::{JobRun, JobRunAction, JobRunErrStatus, JobRunTargetStatus},
    script::Script,
    script_run::{ScriptRun, ScriptRunAction, ScriptRunErrStatus},
};
use crate::storage::service::StorageService;
use crate::storage::{
    errors::StorageErr, job_run_lib::JobRunAssets, prelude::*, script_run_lib::ScriptRunAssets,
};
use crate::trace;
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// -------------------------------- DEPLOYMENTS ------------------------------------ //
/// Finds all deployments whose artifact is from the given source
pub fn find_deps_by_artifact_source(stg: &StorageService, source_id: &str) -> Vec<Arc<Deployment>> {
    let matching_artifacts = stg.art_lib.find_artifacts_by_source(source_id);

    let mut matching_deps: Vec<Arc<Deployment>> = Vec::new();
    for artifact in matching_artifacts.into_iter() {
        matching_deps.extend(stg.dep_lib.find_deps_by_artifact_id(&artifact.id));
    }

    matching_deps
}

pub fn get_dep_image_ids(
    stg: &StorageService,
    dep: &Deployment,
) -> Result<HashSet<String>, StorageErr> {
    let artifact = stg.art_lib.read_mnf(&dep.artifact_id)?;
    Ok(HashSet::from_iter(artifact.get_image_ids_set_owned()))
}

// -------------------------------- ARTIFACTS ------------------------------------ //

/// Returns a list of all the artifacts from ongoing deployments
pub async fn get_ongoing_artifacts(stg: &StorageService) -> Result<Vec<Arc<Artifact>>, StorageErr> {
    let ongoing_artifact_ids = stg.dep_lib.get_ongoing_artifact_ids();
    stg.art_lib.read_mnfs(ongoing_artifact_ids.iter())
}

/// Retrieves all the artifact ids from removable deployments which are not part of
/// any ongoing deployments (since two different deployments can have the same
/// artifact)
pub fn get_removable_artifacts(stg: &StorageService) -> Vec<Arc<Artifact>> {
    let ongoing_artifact_ids = stg.dep_lib.get_ongoing_artifact_ids();
    let mut removable_artifacts: Vec<Arc<Artifact>> = Vec::new();
    for artifact in stg.art_lib.read_all_mnfs() {
        if !ongoing_artifact_ids.contains(&artifact.id) {
            removable_artifacts.push(artifact);
        }
    }
    removable_artifacts
}

// -------------------------------- CONTAINERS ------------------------------------ //
/// Sets the target status of containers with the given image ids to the given
/// target status.
pub fn set_target_for_conts_w_image_id(
    stg: &mut StorageService,
    target_status: ContainerTargetStatus,
    image_ids: &HashSet<String>,
) -> Result<(), StorageErr> {
    let conts = stg.cont_lib.read_all_mnfs();

    for cont in conts.into_iter() {
        if cont.target_status == target_status {
            continue;
        }
        let cont_miru_image_id = match &cont.miru_image_id {
            Some(image_id) => image_id.clone(),
            None => {
                continue;
            }
        };
        if image_ids.contains(&cont_miru_image_id) {
            let mut cont = (*cont).clone();
            cont.target_status = target_status.clone();
            stg.cont_lib.update_mnf(cont)?;
        }
    }

    Ok(())
}

// -------------------------------- IMAGES ------------------------------------ //

/// Returns a list of all the images from ongoing artifacts
pub fn get_ongoing_images(stg: &StorageService) -> Result<Vec<Image>, StorageErr> {
    let ongoing_artifact_ids = stg.dep_lib.get_ongoing_artifact_ids();
    stg.art_lib.get_images(ongoing_artifact_ids.iter())
}

/// Returns a list of all the images from removable artifacts which are not part of any
/// ongoing artifacts (since two different artifacts can have the same image)
pub fn get_removable_images(stg: &StorageService) -> Result<Vec<Image>, StorageErr> {
    let ongoing_image_digests = get_ongoing_images(stg)?
        .into_iter()
        .map(|image| image.digest.clone())
        .collect::<HashSet<String>>();
    let mut removable_images = Vec::new();
    for artifact in get_removable_artifacts(stg).into_iter() {
        for image in artifact.images.iter() {
            if !ongoing_image_digests.contains(&image.digest) {
                removable_images.push(image.clone());
            }
        }
    }
    Ok(removable_images)
}

// -------------------------------- JOB RUNS ------------------------------------ //

pub fn archive_job_run(stg: &mut StorageService, mut job_run: JobRun) -> Result<(), StorageErr> {
    job_run.target_status = JobRunTargetStatus::Archived;
    if job_run.err_status != JobRunErrStatus::Retrying {
        stg.job_run_lib.update_mnf(job_run)?;
        return Ok(());
    }

    // we need to update any of the job run's script run's error statuses to be failed
    // if they were retrying. If some part of this process fails, the worst that can
    // happen is that the script run's status shows as 'retrying' instead of 'failed',
    // which isn't bad enough for me to want to hold up the rest of the application.
    // Just log these errors and move on if they occur
    let script_runs = match stg.script_run_lib.read_mnfs_for_job_run(&job_run) {
        Err(e) => {
            error!(
                "Error updating script runs for job run '{}': {:?}",
                job_run.id, e
            );
            return Ok(());
        }
        Ok(script_runs) => script_runs,
    };

    let job_run_id = job_run.id.clone();
    for script_run in script_runs.into_iter() {
        if script_run.err_status != ScriptRunErrStatus::Retrying {
            continue;
        }
        let mut script_run = (*script_run).clone();
        script_run.err_status = ScriptRunErrStatus::Failed;
        let script_run_id = script_run.id.clone();
        if let Err(e) = stg.script_run_lib.update_mnf(script_run) {
            error!(
                "Error updating script run '{}' for job run '{}': {:?}",
                script_run_id, job_run_id, e
            );
        }
    }

    // update the job run itself
    stg.job_run_lib.update_mnf(job_run)?;

    Ok(())
}

pub fn create_job_run(
    stg: &mut StorageService,
    job_run: JobRun,
    script_runs: Vec<ScriptRun>,
) -> Result<(), StorageErr> {
    // we need to make sure that all the script runs specified in the job run actually
    // exist before creating anything in the job run library
    let script_run_ids = job_run.script_run_ids_owned();
    let mut script_runs_map = HashMap::new();
    for script_run in script_runs.iter() {
        script_runs_map.insert(script_run.id.clone(), script_run);
    }
    for script_run_id in script_run_ids.iter() {
        if !script_runs_map.contains_key(script_run_id) {
            return Err(StorageErr::MissingScriptRunErr {
                job_run_id: job_run.id.clone(),
                script_run_id: script_run_id.clone(),
                trace: trace!(),
            });
        }
    }

    // create the script runs
    for script_run_id in script_run_ids.iter() {
        match script_runs_map.get(script_run_id) {
            Some(script_run) => {
                stg.script_run_lib.create_dir(
                    script_run,
                    ScriptRunAssets {
                        stdout: None,
                        stderr: None,
                    },
                )?;
            }
            // this should be impossible but we'll catch it just in case
            None => {
                return Err(StorageErr::MissingScriptRunErr {
                    job_run_id: job_run.id.clone(),
                    script_run_id: script_run_id.clone(),
                    trace: trace!(),
                })
            }
        }
    }

    // create the job run
    let new_job_run = JobRun::new(
        job_run.id,
        job_run.job_id,
        job_run.device_id,
        job_run.target_status,
        job_run.job_run_steps,
    );
    stg.job_run_lib.create_dir(&new_job_run, JobRunAssets {})?;

    Ok(())
}

fn get_job_run_next_action(
    stg: &StorageService,
    job_run: &JobRun,
) -> Result<JobRunAction, StorageErr> {
    let script_runs = stg
        .script_run_lib
        .read_mnfs_for_job_run(job_run)
        .map_err(|_| StorageErr::MissingScriptRunsErr {
            job_run_id: job_run.id.clone(),
            trace: trace!(),
        })?;
    Ok(job_run.next_action(script_runs.iter(), true))
}

pub fn get_ongoing_job_runs(stg: &StorageService) -> Result<Vec<Arc<JobRun>>, StorageErr> {
    let mut job_runs: Vec<Arc<JobRun>> = Vec::new();
    for job_run in stg.job_run_lib.read_all_mnfs() {
        if get_job_run_next_action(stg, &job_run)? != JobRunAction::Remove {
            job_runs.push(job_run);
        }
    }
    Ok(job_runs)
}

pub fn get_removable_job_runs(stg: &StorageService) -> Result<Vec<Arc<JobRun>>, StorageErr> {
    let mut job_runs: Vec<Arc<JobRun>> = Vec::new();
    for job_run in stg.job_run_lib.read_synced_mnfs() {
        if get_job_run_next_action(stg, &job_run)? == JobRunAction::Remove {
            job_runs.push(job_run);
        }
    }
    Ok(job_runs)
}

// -------------------------------- SCRIPT RUNS ------------------------------------ //

pub fn get_ongoing_script_runs(stg: &StorageService) -> Result<Vec<Arc<ScriptRun>>, StorageErr> {
    let mut script_runs: Vec<Arc<ScriptRun>> = Vec::new();
    for script_run in stg.script_run_lib.read_all_mnfs() {
        let job_run = match stg.job_run_lib.read_mnf(&script_run.job_run_id) {
            Ok(job_run) => job_run,
            Err(e) => {
                error!(
                    "Error reading job run for script run '{}': {:?}",
                    script_run.id, e
                );
                continue;
            }
        };
        if script_run.next_action(&job_run, true) != ScriptRunAction::Remove {
            script_runs.push(script_run);
        }
    }
    Ok(script_runs)
}

pub fn get_removable_script_runs(stg: &StorageService) -> Result<Vec<Arc<ScriptRun>>, StorageErr> {
    let mut script_runs: Vec<Arc<ScriptRun>> = Vec::new();
    for script_run in stg.script_run_lib.read_synced_mnfs() {
        if stg.job_run_lib.read_mnf(&script_run.job_run_id).is_err() {
            script_runs.push(script_run);
        }
    }
    Ok(script_runs)
}

// ----------------------------------- JOBS ---------------------------------------- //

pub fn get_ongoing_job_ids(stg: &StorageService) -> Result<HashSet<String>, StorageErr> {
    let mut job_ids: HashSet<String> = HashSet::new();
    for job_run in get_ongoing_job_runs(stg)? {
        job_ids.insert(job_run.job_id.clone());
    }
    Ok(job_ids)
}

pub fn get_ongoing_jobs(stg: &StorageService) -> Result<Vec<Arc<Job>>, StorageErr> {
    let mut jobs: Vec<Arc<Job>> = Vec::new();
    for job_id in get_ongoing_job_ids(stg)? {
        jobs.push(stg.job_lib.read_mnf(&job_id)?);
    }
    Ok(jobs)
}

pub fn get_removable_jobs(stg: &StorageService) -> Result<Vec<Arc<Job>>, StorageErr> {
    let mut jobs: Vec<Arc<Job>> = Vec::new();
    let ongoing_job_ids = get_ongoing_job_ids(stg)?;
    for job in stg.job_lib.read_all_mnfs() {
        if !ongoing_job_ids.contains(&job.id) {
            jobs.push(job);
        }
    }
    Ok(jobs)
}

// ---------------------------------- SCRIPTS -------------------------------------- //

pub fn get_ongoing_script_ids(stg: &StorageService) -> Result<HashSet<String>, StorageErr> {
    let mut script_ids: HashSet<String> = HashSet::new();
    // these shoulddd be the same but we'll use both for safety
    for script_run in get_ongoing_script_runs(stg)? {
        script_ids.insert(script_run.script_id.clone());
    }
    for job in get_ongoing_jobs(stg)? {
        for script_id in job.script_ids_owned() {
            script_ids.insert(script_id);
        }
    }
    Ok(script_ids)
}

pub fn get_ongoing_scripts(stg: &StorageService) -> Result<Vec<Arc<Script>>, StorageErr> {
    let script_ids = get_ongoing_script_ids(stg)?;
    stg.script_lib.read_mnfs(script_ids.iter())
}

pub fn get_removable_scripts(stg: &StorageService) -> Result<Vec<Arc<Script>>, StorageErr> {
    let mut scripts: Vec<Arc<Script>> = Vec::new();
    let ongoing_script_ids = get_ongoing_script_ids(stg)?;
    for script in stg.script_lib.read_all_mnfs() {
        if !ongoing_script_ids.contains(&script.id) {
            scripts.push(script);
        }
    }
    Ok(scripts)
}
