use std::fmt::Display;
use std::mem::size_of;
use widestring::U16CString;

use windows::Win32::Foundation::BOOL;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    GetProcessAffinityMask, OpenProcess, SetProcessAffinityMask, PROCESS_ACCESS_RIGHTS,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
};

pub mod error;
use error::*;

pub enum Success {
    Updated {
        process_name: String,
        process_id: u32,
        old_affinity_mask: usize,
        new_affinity_mask: usize,
        system_affinity_mask: usize,
    },
    Unchanged {
        process_name: String,
        process_id: u32,
        affinity_mask: usize,
        system_affinity_mask: usize,
    },
}
impl Success {
    fn updated(
        process_name: &str,
        process_id: u32,
        old_affinity_mask: usize,
        new_affinity_mask: usize,
        system_affinity_mask: usize,
    ) -> Self {
        Success::Updated {
            process_name: process_name.to_string(),
            process_id,
            old_affinity_mask,
            new_affinity_mask,
            system_affinity_mask,
        }
    }

    fn unchanged(
        process_name: &str,
        process_id: u32,
        affinity_mask: usize,
        system_affinity_mask: usize,
    ) -> Self {
        Success::Unchanged {
            process_name: process_name.to_string(),
            process_id,
            affinity_mask,
            system_affinity_mask,
        }
    }
}
impl Display for Success {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
			Success::Updated{process_name, process_id, old_affinity_mask, new_affinity_mask, system_affinity_mask} => write!(f, 
				"Updated affinity mask for process {process_name} with PID {process_id}: {old_affinity_mask:x} -> {new_affinity_mask:x} (system affinity mask: {system_affinity_mask:x})"),
			Success::Unchanged { process_name, process_id, affinity_mask, system_affinity_mask } => write!(f, "Affinity mask for process {process_name} with PID {process_id} already set to {affinity_mask:x} (system affinity mask: {system_affinity_mask:x})"),
		}
    }
}

pub fn set_process_affinity(process_name: &str, affinity_mask: usize) -> Result<Success, Error> {
    let process_id = get_process_pid_by_name(process_name)?;

    let access_rights: PROCESS_ACCESS_RIGHTS =
        PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION; // bitor is not impl for const
    const FALSE: BOOL = BOOL(0); // value does not matter, only used when creating processes

    unsafe {
        let process_handle = match OpenProcess(access_rights, FALSE, process_id) {
            Ok(process_handle) => process_handle,
            Err(api_error) => {
                return Err(Error::APIError(APIError::with_message_and_api_error(
                    &format!("Error opening process {process_name}"),
                    api_error,
                )))
            }
        };

        let mut process_affinity_mask = 0_usize;
        let mut system_affinity_mask = 0_usize;

        if !GetProcessAffinityMask(
            process_handle,
            &mut process_affinity_mask,
            &mut system_affinity_mask,
        )
        .as_bool()
        {
            return Err(Error::APIError(APIError::with_message(
                "GetProcessAffinityMask",
            )));
        }

        if process_affinity_mask == affinity_mask {
            return Ok(Success::unchanged(
                process_name,
                process_id,
                affinity_mask,
                system_affinity_mask,
            ));
        }

        if !is_possible_affinity(affinity_mask, system_affinity_mask) {
            return Err(Error::InvalidAffinityMaskError(
                InvalidAffinityMaskError::new(affinity_mask, system_affinity_mask),
            ));
        }

        if !SetProcessAffinityMask(process_handle, affinity_mask).as_bool() {
            return Err(Error::APIError(APIError::with_message(
                "SetProcessAffinityMask",
            )));
        }

        Ok(Success::updated(
            process_name,
            process_id,
            process_affinity_mask,
            affinity_mask,
            system_affinity_mask,
        ))
    }
}

fn is_possible_affinity(process_affinity: usize, system_affinity: usize) -> bool {
    let res = process_affinity & !system_affinity;
    res == 0
}

fn get_process_pid_by_name(process_name: &str) -> Result<u32, Error> {
    unsafe {
        let process_list = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(process_list) => process_list,
            Err(error) => {
                return Err(Error::APIError(APIError::with_message_and_api_error(
                    "Unable to read process list",
					error
                )))
            }
        };

        let mut current_process = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut has_next = Process32FirstW(process_list, &mut current_process);

        while has_next.as_bool() {
            let current_process_name =
                U16CString::from_ptr_str(current_process.szExeFile.as_ptr()).to_string_lossy();

            if current_process_name.eq(process_name) {
                return Ok(current_process.th32ProcessID);
            }

            has_next = Process32NextW(process_list, &mut current_process);
        }
    }

    Err(Error::ProcessNotFoundError(ProcessNotFoundError::new(
        process_name,
    )))
}
