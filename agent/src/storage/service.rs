// internal crates
use crate::filesys::dir::Dir;
use crate::filesys::path::PathExt;
use crate::models::device_cfg::DeviceConfig;
use crate::storage::{
    art_lib::ArtifactLibrary, auth_dir::AuthDir, cont_lib::ContainerLibrary,
    dep_lib::DeploymentLibrary, device_cfg::DeviceConfigFile, errors::StorageErr,
    job_lib::JobLibrary, job_run_lib::JobRunLibrary, prelude::*, script_lib::ScriptLibrary,
    script_run_lib::ScriptRunLibrary,
};
use crate::trace;
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug)]
pub struct StorageService {
    pub dir: Dir,
    // contents of the miru application directory
    pub cfg: DeviceConfigFile,
    pub auth_dir: AuthDir,
    pub art_lib: ArtifactLibrary,
    pub cont_lib: ContainerLibrary,
    pub dep_lib: DeploymentLibrary,
    pub job_lib: JobLibrary,
    pub job_run_lib: JobRunLibrary,
    pub script_run_lib: ScriptRunLibrary,
    pub script_lib: ScriptLibrary,
}

impl StorageService {
    // Creates a new StorageService instance. Since this struct holds information about the
    // filesystem of the device in an optimized and thread-safe manner, it should only
    // be initialized ONCE at the start of the program and passed through the "StorageService"
    // struct to all functions. The default location of the miru application directory
    // is /var/lib/miru. The only reason to not use this directory path is for testing
    // purposes.
    pub fn new(dir: Option<Dir>) -> Result<Self, StorageErr> {
        // initialize the directory
        let dir = match dir {
            Some(dir) => dir,
            None => Dir::new(Self::miru_app_dir_path()),
        };

        // initialize the device configuration file
        let file = dir.new_file(DeviceConfigFile::file_name());
        let cfg = DeviceConfigFile::new(file)?;

        // initialize the auth directory. This directory has to exist. It can not be
        // regenerated since it contains the RSA private key. A new installation must
        // be performed to generate a new RSA key pair.
        let auth_dir = AuthDir::new(dir.new_subdir(&[&AuthDir::name()]))?;

        // initialize the deployments directory. This directory should exist but if it
        // doesn't, it will be created. The agent can simply repull the correct
        // deployment state from the server if it got deleted for some reason.
        let dep_lib =
            DeploymentLibrary::create_if_absent(dir.new_subdir(&[&DeploymentLibrary::name()]))?;

        // initialize the artifacts directory. This directory should exist but if it
        // doesn't, it will be created. The agent can simply repull the correct
        // artifacts from the server if it got deleted for some reason.
        let art_lib =
            ArtifactLibrary::create_if_absent(dir.new_subdir(&[&ArtifactLibrary::name()]))?;

        // initialize the containers directory. This directory should exist but if it
        // doesn't, it will be created. The agent can simply repull the correct
        // containers from the server if it got deleted for some reason.
        let cont_lib =
            ContainerLibrary::create_if_absent(dir.new_subdir(&[&ContainerLibrary::name()]))?;

        // initialize the jobs directory. This directory should exist but if it
        // doesn't, it will be created. The agent can simply repull the correct
        // jobs from the server if it got deleted for some reason.
        let job_lib = JobLibrary::create_if_absent(dir.new_subdir(&[&JobLibrary::name()]))?;

        // initialize the job runs directory. This directory should exist but if it
        // doesn't, it will be created. The agent can simply repull the correct
        // job runs from the server if it got deleted for some reason.
        let job_run_lib =
            JobRunLibrary::create_if_absent(dir.new_subdir(&[&JobRunLibrary::name()]))?;

        // initialize the scripts directory. This directory should exist but if it
        // doesn't, it will be created. The agent can simply repull the correct
        // scripts from the server if it got deleted for some reason.
        let script_lib =
            ScriptLibrary::create_if_absent(dir.new_subdir(&[&ScriptLibrary::name()]))?;

        // initialize the script runs directory. This directory should exist but if it
        // doesn't, it will be created. The agent can simply repull the correct
        // script runs from the server if it got deleted for some reason.
        let script_run_lib =
            ScriptRunLibrary::create_if_absent(dir.new_subdir(&[&ScriptRunLibrary::name()]))?;

        let miru_app_dir = Self {
            dir,
            cfg,
            auth_dir,
            art_lib,
            cont_lib,
            dep_lib,
            job_lib,
            job_run_lib,
            script_lib,
            script_run_lib,
        };

        Ok(miru_app_dir)
    }

    /// Return the default path of the miru application directory
    pub fn miru_app_dir_path() -> String {
        "/var/lib/miru".to_string()
    }

    pub fn compose_project_dir_path(dir_name: &str) -> String {
        let miru_app_dir_path = Self::miru_app_dir_path();
        format!("{}/{}/{}", miru_app_dir_path, "compose", dir_name)
    }

    /// Create the Miru application directory and its contents. If miru_app_dir is None
    /// (common case), the app directory is created at the default path. You should only
    /// specify the directory for testing purposes. If the directory already exists, an
    /// error is returned.
    pub fn create(
        miru_app_dir: Option<Dir>,
        device_cfg: &DeviceConfig,
        auth_token: &str,
    ) -> Result<Self, StorageErr> {
        let miru_app_dir = match miru_app_dir {
            Some(dir) => dir,
            None => Dir::new(Self::miru_app_dir_path()),
        };
        debug!("Creating StorageService at {:?}", miru_app_dir);

        // create the directory
        miru_app_dir
            .assert_doesnt_exist()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        miru_app_dir
            .create_if_absent()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;

        // create the contents
        let file = miru_app_dir.new_file(DeviceConfigFile::file_name());
        DeviceConfigFile::create(file, device_cfg)?;
        AuthDir::create(miru_app_dir.new_subdir(&[&AuthDir::name()]), auth_token)?;
        DeploymentLibrary::create_if_absent(
            miru_app_dir.new_subdir(&[&DeploymentLibrary::name()]),
        )?;
        ArtifactLibrary::create_if_absent(miru_app_dir.new_subdir(&[&ArtifactLibrary::name()]))?;
        ContainerLibrary::create_if_absent(miru_app_dir.new_subdir(&[&ContainerLibrary::name()]))?;
        JobLibrary::create_if_absent(miru_app_dir.new_subdir(&[&JobLibrary::name()]))?;
        JobRunLibrary::create_if_absent(miru_app_dir.new_subdir(&[&JobRunLibrary::name()]))?;
        ScriptLibrary::create_if_absent(miru_app_dir.new_subdir(&[&ScriptLibrary::name()]))?;
        ScriptRunLibrary::create_if_absent(miru_app_dir.new_subdir(&[&ScriptRunLibrary::name()]))?;

        // return the new instance
        debug!("Created StorageService at {:?}", miru_app_dir);
        Self::new(Some(miru_app_dir))
    }

    // Validate the entire Miru application directory by ensuring all of its contents
    // are valid.
    pub fn validate(&mut self) -> Result<(), StorageErr> {
        self.cfg.validate()?;
        self.auth_dir.validate()?;
        self.art_lib.validate()?;
        self.cont_lib.validate()?;
        self.dep_lib.validate()?;
        self.job_lib.validate()?;
        self.job_run_lib.validate()?;
        self.script_lib.validate()?;
        self.script_run_lib.validate()?;
        Ok(())
    }
}
