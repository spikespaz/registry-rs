use registry::{Hive, Security};
use std::convert::TryInto;
use utfx::U16CString;
use windows::Win32::{System::Threading::GetCurrentProcess,
    Security::{
        SE_PRIVILEGE_ENABLED, TOKEN_PRIVILEGES, TOKEN_ADJUST_PRIVILEGES, LUID_AND_ATTRIBUTES,
        AdjustTokenPrivileges, LookupPrivilegeValueW
    },
    Foundation::{LUID, HANDLE, PWSTR},
    System::Threading::OpenProcessToken
};

const SE_BACKUP_NAME: &'static str = "SeBackupPrivilege";
const SE_RESTORE_NAME: &'static str = "SeRestorePrivilege";

fn main() -> Result<(), std::io::Error> {
    let mut token = HANDLE::default();
    let r = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &mut token) };
    if r == false {
        return Err(std::io::Error::last_os_error());
    }

    set_privilege(token, SE_RESTORE_NAME)?;
    set_privilege(token, SE_BACKUP_NAME)?;
    let hive_key = Hive::load_file(
        r"C:\Users\Default\NTUSER.DAT",
        Security::Read | Security::Write,
    )
    .unwrap();

    let keys: Vec<_> = hive_key.keys().map(|k| k.unwrap().to_string()).collect();

    println!("{:?}", keys);
    Ok(())
}

fn set_privilege(handle: HANDLE, name: &str) -> Result<(), std::io::Error> {
    let mut luid: LUID = LUID {
        LowPart: 0,
        HighPart: 0,
    };
    let name: U16CString = name.try_into().unwrap();
    let r = unsafe { LookupPrivilegeValueW(PWSTR::default(), PWSTR(name.as_ptr() as *mut u16), &mut luid) };
    if r == false {
        return Err(std::io::Error::last_os_error());
    }

    let mut privilege = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };

    let r = unsafe {
        AdjustTokenPrivileges(
            handle,
            false,
            &mut privilege,
            std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    if r == false {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}
