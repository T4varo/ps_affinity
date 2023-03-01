use std::fmt::Display;
pub enum Error {
    InvalidAffinityMaskError(InvalidAffinityMaskError),
    APIError(APIError),
    ProcessNotFoundError(ProcessNotFoundError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::InvalidAffinityMaskError(error) => error.fmt(f),
            Error::APIError(error) => error.fmt(f),
            Error::ProcessNotFoundError(error) => error.fmt(f),
        }
    }
}

pub struct InvalidAffinityMaskError {
    desired_affinity: usize,
    system_affinity: usize,
}
impl InvalidAffinityMaskError {
    pub fn new(desired_affinity: usize, system_affinity: usize) -> Self {
        InvalidAffinityMaskError {
            desired_affinity,
            system_affinity,
        }
    }
}
impl Display for InvalidAffinityMaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Affinity mask not applicable on this system:\nProcess Mask: {:064b}\nSystem Mask:  {:064b}", self.desired_affinity, self.system_affinity)
    }
}

pub struct APIError {
    api_error: Option<windows::core::Error>,
    message: Option<String>,
}
impl APIError {
    pub fn new() -> Self {
        APIError {
            api_error: None,
            message: None,
        }
    }

    pub fn with_message(message: &str) -> Self {
        APIError {
            api_error: None,
            message: Some(message.to_string()),
        }
    }

    pub fn with_api_error(api_error: windows::core::Error) -> Self {
        APIError {
            api_error: Some(api_error),
            message: None,
        }
    }

    pub fn with_message_and_api_error(message: &str, api_error: windows::core::Error) -> Self {
        APIError {
            api_error: Some(api_error),
            message: Some(message.to_string()),
        }
    }
}
impl Default for APIError {
    fn default() -> Self {
        APIError::new()
    }
}
impl Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.api_error, &self.message) {
            (Some(api_error), Some(message)) => write!(f, "{message}: {api_error}"),
            (Some(api_error), None) => write!(f, "Error using the API: {api_error}"),
            (None, Some(message)) => write!(f, "Error using the API: {message}"),
            (None, None) => write!(f, "Error using the API."),
        }
    }
}

pub struct ProcessNotFoundError {
    process_name: String,
}
impl ProcessNotFoundError {
    pub fn new(process_name: &str) -> Self {
        ProcessNotFoundError {
            process_name: process_name.to_string(),
        }
    }
}
impl Display for ProcessNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot find a process with the name {}.",
            self.process_name
        )
    }
}
