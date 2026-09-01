//! Read a file's Windows security descriptor and emit it as JSON.
//!
//! # Why this lives in a compiled binary
//!
//! Node exposes no ACL API of any kind, so the installer cannot ask a Windows
//! host whether a private key is owner-restricted. The two alternatives were
//! both rejected:
//!
//!   * Parsing `icacls` output. Its principal names are LOCALIZED --
//!     "Administrators" is "Administratoren" on a German host -- so any parser
//!     comparing names silently passes or silently fails depending on the
//!     operator's display language. That is the single most common defect in
//!     hand-rolled versions of this check.
//!   * Giving up and weakening the gate on Windows. That removes the guarantee
//!     instead of expressing it.
//!
//! This subcommand is deliberately a DUMB READER. It resolves nothing, decides
//! nothing, and applies nothing; it reports owner SID, DACL presence,
//! DACL protection, inherited ACE count, and every ACE's trustee SID as a
//! string, and the caller makes every judgement. Emitting SIDs rather than names
//! is what makes the caller's comparison locale-independent.
//!
//! Remediation is never performed here. Silently repairing a key's ACL would
//! destroy the only evidence that it had been changed.

use std::path::Path;

use serde::Serialize;

/// Bumped only when the emitted shape changes incompatibly. The consumer
/// (`scripts/lib/key-protection.js`) rejects any other value and fails closed.
pub const SECURITY_REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AceReport {
    /// Trustee as a string SID, e.g. `S-1-5-32-544`. Never a display name.
    pub sid: String,
    /// `allow`, `deny`, `audit`, or `unsupported`.
    pub kind: &'static str,
    pub inherited: bool,
    pub access_mask: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsSecurityReport {
    pub schema_version: u32,
    pub model: &'static str,
    pub path: String,
    /// SID of the user in this process's access token.
    pub process_owner_sid: String,
    pub owner_sid: String,
    pub group_sid: Option<String>,
    /// False for a NULL DACL, which grants full control to everyone.
    pub dacl_present: bool,
    /// True when SE_DACL_PROTECTED is set, i.e. nothing is inherited.
    pub dacl_protected: bool,
    pub inherited_ace_count: usize,
    pub aces: Vec<AceReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsupportedSecurityReport {
    pub schema_version: u32,
    pub model: &'static str,
    pub path: String,
    pub supported: bool,
    pub reason: &'static str,
}

/// Emit the report for `path` as JSON on stdout.
pub fn inspect(path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(windows)]
    {
        let report = windows_impl::read(path)?;
        Ok(serde_json::to_string_pretty(&report)?)
    }
    #[cfg(not(windows))]
    {
        // A POSIX host asserts owner-only protection through mode and ownership,
        // which the caller reads directly. Reporting `supported: false` here
        // makes a misrouted call fail closed rather than look like a pass.
        let report = UnsupportedSecurityReport {
            schema_version: SECURITY_REPORT_SCHEMA_VERSION,
            model: "posix",
            path: path.display().to_string(),
            supported: false,
            reason: "security descriptors exist only on Windows; assert mode and ownership instead",
        };
        Ok(serde_json::to_string_pretty(&report)?)
    }
}

#[cfg(windows)]
mod windows_impl {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::ptr;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, LocalFree};
    use windows_sys::Win32::Security::Authorization::{
        ConvertSidToStringSidW, GetNamedSecurityInfoW, SE_FILE_OBJECT,
    };
    use windows_sys::Win32::Security::{
        GetSecurityDescriptorControl, GetTokenInformation, ACL, DACL_SECURITY_INFORMATION,
        GROUP_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
        TOKEN_QUERY, TOKEN_USER, TokenUser,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    use super::{AceReport, WindowsSecurityReport, SECURITY_REPORT_SCHEMA_VERSION};

    type BoxError = Box<dyn std::error::Error + Send + Sync>;

    // Declared locally rather than imported so that a rename in the binding
    // crate cannot silently change the meaning of the protection check.
    const SE_DACL_PROTECTED_FLAG: u16 = 0x1000;
    const INHERITED_ACE_FLAG: u8 = 0x10;
    const ACCESS_ALLOWED_ACE_TYPE: u8 = 0x00;
    const ACCESS_DENIED_ACE_TYPE: u8 = 0x01;
    const SYSTEM_AUDIT_ACE_TYPE: u8 = 0x02;
    /// Offset of the trustee SID inside a non-object ACE: ACE_HEADER (4 bytes)
    /// followed by the 4-byte access mask.
    const ACE_SID_OFFSET: usize = 8;

    #[repr(C)]
    struct AceHeader {
        ace_type: u8,
        ace_flags: u8,
        ace_size: u16,
    }

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    }

    /// Convert a SID to its canonical string form. The buffer returned by
    /// `ConvertSidToStringSidW` is LocalAlloc'd and must be LocalFree'd.
    unsafe fn sid_to_string(sid: PSID) -> Result<String, BoxError> {
        if sid.is_null() {
            return Err("security descriptor has a null security identifier".into());
        }
        let mut raw: *mut u16 = ptr::null_mut();
        if ConvertSidToStringSidW(sid, &mut raw) == 0 {
            return Err(format!(
                "ConvertSidToStringSidW failed: {}",
                std::io::Error::last_os_error()
            )
            .into());
        }
        let mut length = 0usize;
        while *raw.add(length) != 0 {
            length += 1;
        }
        let text = String::from_utf16(std::slice::from_raw_parts(raw, length))?;
        LocalFree(raw as *mut c_void);
        Ok(text)
    }

    unsafe fn process_owner_sid() -> Result<String, BoxError> {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(format!("OpenProcessToken failed: {}", std::io::Error::last_os_error()).into());
        }
        let mut needed: u32 = 0;
        GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut needed);
        let mut buffer = vec![0u8; needed as usize];
        let ok = GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr() as *mut c_void,
            needed,
            &mut needed,
        );
        if ok == 0 {
            CloseHandle(token);
            return Err(format!(
                "GetTokenInformation(TokenUser) failed: {}",
                std::io::Error::last_os_error()
            )
            .into());
        }
        let user = &*(buffer.as_ptr() as *const TOKEN_USER);
        let sid = sid_to_string(user.User.Sid);
        CloseHandle(token);
        sid
    }

    pub fn read(path: &Path) -> Result<WindowsSecurityReport, BoxError> {
        // Canonicalize so a relative or short-name path reaches the same object
        // the caller means. `fs::canonicalize` returns a \\?\-prefixed verbatim
        // path on Windows; GetNamedSecurityInfoW accepts it, and the prefix is
        // stripped for the REPORTED path so that callers comparing paths do not
        // have to know about it.
        let canonical = std::fs::canonicalize(path)?;
        let display = canonical
            .to_string_lossy()
            .strip_prefix(r"\\?\")
            .map(str::to_owned)
            .unwrap_or_else(|| canonical.to_string_lossy().into_owned());

        let wide_path = wide(&canonical);
        let mut owner: PSID = ptr::null_mut();
        let mut group: PSID = ptr::null_mut();
        let mut dacl: *mut ACL = ptr::null_mut();
        let mut descriptor: PSECURITY_DESCRIPTOR = ptr::null_mut();

        unsafe {
            let status = GetNamedSecurityInfoW(
                wide_path.as_ptr(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                &mut owner,
                &mut group,
                &mut dacl,
                ptr::null_mut(),
                &mut descriptor,
            );
            if status != 0 {
                return Err(format!("GetNamedSecurityInfoW failed with status {status}").into());
            }

            let mut control: u16 = 0;
            let mut revision: u32 = 0;
            if GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) == 0 {
                LocalFree(descriptor);
                return Err(format!(
                    "GetSecurityDescriptorControl failed: {}",
                    std::io::Error::last_os_error()
                )
                .into());
            }

            let owner_sid = sid_to_string(owner)?;
            let group_sid = if group.is_null() { None } else { Some(sid_to_string(group)?) };
            let process_owner = process_owner_sid()?;

            let mut aces = Vec::new();
            let mut inherited_ace_count = 0usize;
            // A NULL DACL is not an empty DACL. NULL grants everyone full
            // control; empty denies everyone. Reporting them identically would
            // turn the worst case into the best one.
            let dacl_present = !dacl.is_null();
            if dacl_present {
                let count = (*dacl).AceCount as usize;
                let acl_bytes = dacl as *const u8;
                // Walk by ACE size from the end of the ACL header rather than
                // calling GetAce, so the offsets used for the trustee SID and
                // the ones used for iteration come from the same arithmetic.
                let mut offset = std::mem::size_of::<ACL>();
                for _ in 0..count {
                    let header = &*(acl_bytes.add(offset) as *const AceHeader);
                    let inherited = header.ace_flags & INHERITED_ACE_FLAG != 0;
                    if inherited {
                        inherited_ace_count += 1;
                    }
                    let (kind, sid) = match header.ace_type {
                        ACCESS_ALLOWED_ACE_TYPE => ("allow", Some(ACE_SID_OFFSET)),
                        ACCESS_DENIED_ACE_TYPE => ("deny", Some(ACE_SID_OFFSET)),
                        SYSTEM_AUDIT_ACE_TYPE => ("audit", Some(ACE_SID_OFFSET)),
                        // Object and callback ACEs carry extra fields before the
                        // trustee. Rather than guess at the layout, report the
                        // entry with an unresolvable trustee so the caller's
                        // allowlist rejects it.
                        _ => ("unsupported", None),
                    };
                    let mask = *(acl_bytes.add(offset + 4) as *const u32);
                    aces.push(AceReport {
                        sid: match sid {
                            Some(sid_offset) => {
                                sid_to_string(acl_bytes.add(offset + sid_offset) as PSID)?
                            }
                            None => "UNRESOLVED".to_owned(),
                        },
                        kind,
                        inherited,
                        access_mask: mask,
                    });
                    offset += header.ace_size as usize;
                }
            }

            LocalFree(descriptor);

            Ok(WindowsSecurityReport {
                schema_version: SECURITY_REPORT_SCHEMA_VERSION,
                model: "windows-security-descriptor",
                path: display,
                process_owner_sid: process_owner,
                owner_sid,
                group_sid,
                dacl_present,
                dacl_protected: control & SE_DACL_PROTECTED_FLAG != 0,
                inherited_ace_count,
                aces,
            })
        }
    }
}
