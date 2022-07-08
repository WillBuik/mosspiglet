use std::{path::PathBuf, ffi::OsString};

use thiserror::Error;
use windows_service::{service_manager::{ServiceManager, ServiceManagerAccess}, service::{ServiceAccess, ServiceInfo, ServiceType, ServiceStartType, ServiceErrorControl}};

/// System service managment errors.
#[derive(Error, Debug)]
pub enum ServiceError {
    /// Process does not have valid permissions to interact with the
    /// service manager.
    #[error("access to service manager was denied")]
    AccessDenied,

    /// The service name is not valid.
    #[error("invalid service name")]
    InvalidServiceName,

    /// Installation of the service failed.
    #[error("failed to install service: {0}")]
    InstallationFailed (String),

    /// The service is not insalled into the OS service manager.
    #[error("service is not installed")]
    ServiceNotInstalled,

    /// The service is running and cannot be uninstalled.
    #[error("service is running")]
    ServiceRunning,

    /// An unknown error occurred.
    #[error("unknown error: {0}")]
    UnknownError (String)
}

impl From<windows_service::Error> for ServiceError {
    /// Convert Windows service errors into a ServiceError.
    fn from(err: windows_service::Error) -> Self {
        match err {
            windows_service::Error::InvalidAccountName(err) => Self::InstallationFailed(format!("{}", err)),
            windows_service::Error::InvalidAccountPassword(err) => Self::InstallationFailed(format!("{}", err)),
            windows_service::Error::InvalidDisplayName(err) => Self::InstallationFailed(format!("{}", err)),
            windows_service::Error::InvalidDatabaseName(err) => Self::InstallationFailed(format!("{}", err)),
            windows_service::Error::InvalidExecutablePath(err) => Self::InstallationFailed(format!("{}", err)),
            windows_service::Error::InvalidLaunchArgument(_, err) => Self::InstallationFailed(format!("{}", err)),
            windows_service::Error::LaunchArgumentsNotSupported => Self::InstallationFailed(format!("launch arguments not supported")),
            windows_service::Error::InvalidDependency(err) => Self::InstallationFailed(format!("{}", err)),
            windows_service::Error::InvalidMachineName(err) => Self::UnknownError(format!("{}", err)),
            windows_service::Error::InvalidServiceName(_) => Self::InvalidServiceName,
            windows_service::Error::InvalidStartArgument(err) => Self::UnknownError(format!("{}", err)),
            windows_service::Error::InvalidServiceState(err) => Self::UnknownError(format!("{}", err)),
            windows_service::Error::InvalidServiceStartType(err) => Self::UnknownError(format!("{}", err)),
            windows_service::Error::InvalidServiceErrorControl(err) => Self::UnknownError(format!("{}", err)),
            windows_service::Error::InvalidServiceActionType(err) => Self::UnknownError(format!("{}", err)),
            windows_service::Error::InvalidServiceActionFailuresRebootMessage(err) => Self::UnknownError(format!("{}", err)),
            windows_service::Error::InvalidServiceActionFailuresCommand(err) => Self::UnknownError(format!("{}", err)),
            windows_service::Error::InvalidServiceDescription(err) => Self::InstallationFailed(format!("{}", err)),
            windows_service::Error::Winapi(err) => {
                match (err.kind(), err.raw_os_error()) {
                    (std::io::ErrorKind::PermissionDenied, _) => Self::AccessDenied,
                    (_, Some(1060)) => Self::ServiceNotInstalled,
                    _ => Self::UnknownError(format!("Kind={:?}, {}", err.kind(), err)),
                }
            },
        }
    }
}

/// System service status.
#[derive(PartialEq, Eq, Debug)]
pub enum ServiceStatus {
    /// Service is not installed into the OS service manager.
    Uninstalled,
    /// Service process is not stopped, but is installed.
    Stopped,
    /// Service process is running, but may be in the process of shutting down.
    Running,
}

/// Service installation details.
#[derive(Debug)]
pub struct ServiceDescription {
    /// Friendly/display name for the service.
    pub friendly_name: OsString,
    /// Path to the service binary.
    pub binary_path: PathBuf,
    /// Arguments to the service binary.
    pub args: Vec<OsString>,
}

/// System service manager.
/// 
/// Used to [un]install, query, and manage a system service.
pub struct SystemService (String);

impl SystemService {
    /// Create a new SystemService to interact with the service `name`.
    pub fn new(name: String) -> Self {
        SystemService (name)
    }

    /// Query the status of the service.
    pub fn status(&self) -> Result<ServiceStatus, ServiceError> {
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
        let service_handle = manager.open_service(self.0.clone(), ServiceAccess::QUERY_STATUS).map_err(|err| ServiceError::from(err));

        match service_handle {
            Ok(service_handle) => {
                let status = service_handle.query_status()?;
                match status.current_state {
                    windows_service::service::ServiceState::Stopped => Ok(ServiceStatus::Stopped),
                    _ => Ok(ServiceStatus::Running),
                }
            },
            Err(ServiceError::ServiceNotInstalled) => {
                Ok(ServiceStatus::Uninstalled)
            },
            Err(err) => Err(ServiceError::InstallationFailed(format!("{}", err))),
        }
    }

    /// Get the service description for this service.
    /// 
    /// Returns an error if the service is not installed.
    pub fn description(&self) -> Result<ServiceDescription, ServiceError> {
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
        let service_handle = manager.open_service(self.0.clone(), ServiceAccess::QUERY_CONFIG)?;
        let service_config = service_handle.query_config()?;

        Ok(ServiceDescription {
            friendly_name: service_config.display_name,
            binary_path: service_config.executable_path.into(),
            args: vec![], // TODO: there doesn't seem to be a way to get the arguments.
        })
    }

    /// Install the service.
    /// 
    /// If the service is already installed, this will update its service
    /// description but will not to restart the service if it is already
    /// running.
    pub fn install(&self, description: ServiceDescription) -> Result<ServiceDescription, ServiceError> {
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;
        let service_info = ServiceInfo {
            name: (&self.0).into(),
            display_name: description.friendly_name.into(),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: description.binary_path.into(),
            launch_arguments: description.args,
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };
        manager.create_service(&service_info, ServiceAccess::all())?;

        self.description()
    }

    /// Uninstall the service.
    /// 
    /// Returns an error if the service is running.
    pub fn uninstall(&self) -> Result<(), ServiceError> {
        let status = self.status()?;
        if status == ServiceStatus::Running {
            return Err(ServiceError::ServiceRunning);
        }
        
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
        let service_handle = manager.open_service(self.0.clone(), ServiceAccess::all())?;
        service_handle.delete()?;

        Ok(())
    }

    /// Start the service.
    /// 
    /// This queues a start for the service and returns immediately. If
    /// the service is already running or in the process of stopping
    /// this may have no effect. Confirm with `status()`.
    pub fn start(&self) -> Result<(), ServiceError> {
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
        let service_handle = manager.open_service(self.0.clone(), ServiceAccess::START)?;
        service_handle.start(&Vec::<OsString>::new())?;

        Ok(())
    }

    /// Stop the service.
    /// 
    /// This queues a stop for the service and returns immediately. If
    /// the service is already stopped or in the process of starting
    /// this may have no effect. Confirm with `status()`.
    pub fn stop(&self) -> Result<(), ServiceError> {
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
        let service_handle = manager.open_service(self.0.clone(), ServiceAccess::STOP)?;
        service_handle.stop()?;

        Ok(())
    }
}
