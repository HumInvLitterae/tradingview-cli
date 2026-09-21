//! Native Windows state-directory creation and handle-based security checks.

use crate::{Failure, Result, error::StateSecurityReason as Reason};
use std::{
    ffi::c_void,
    fs::{File, OpenOptions},
    mem::{offset_of, size_of},
    os::windows::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::Path,
    ptr::null_mut,
};
use windows_sys::Win32::{
    Foundation::{ERROR_ALREADY_EXISTS, ERROR_INSUFFICIENT_BUFFER, GetLastError, LocalFree},
    Security::{
        ACCESS_ALLOWED_ACE, ACE_HEADER, ACL,
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
            GetSecurityInfo, SDDL_REVISION_1, SE_FILE_OBJECT,
        },
        DACL_SECURITY_INFORMATION, EqualSid, GetAce, GetLengthSid, GetTokenInformation, IsValidAcl,
        IsValidSid, IsWellKnownSid, OWNER_SECURITY_INFORMATION, PSID, SECURITY_ATTRIBUTES,
        TOKEN_QUERY, TOKEN_USER, TokenUser, WinBuiltinAdministratorsSid, WinCreatorOwnerRightsSid,
        WinCreatorOwnerSid, WinLocalSystemSid,
    },
    Storage::FileSystem::{
        CreateDirectoryW, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_READ,
        FILE_SHARE_WRITE, READ_CONTROL,
    },
    System::{
        SystemServices::{ACCESS_ALLOWED_ACE_TYPE, ACCESS_DENIED_ACE_TYPE},
        Threading::{GetCurrentProcess, OpenProcessToken},
    },
};

fn rejected(reason: Reason) -> Failure {
    Failure::StateSecurity {
        reason,
        win32_error: None,
    }
}

fn native_error(reason: Reason, code: u32) -> Failure {
    Failure::StateSecurity {
        reason,
        win32_error: Some(code),
    }
}

fn last_error(reason: Reason) -> Failure {
    // SAFETY: GetLastError has no preconditions; call immediately after failure.
    native_error(reason, unsafe { GetLastError() })
}

fn io_error(reason: Reason, error: std::io::Error) -> Failure {
    Failure::StateSecurity {
        reason,
        win32_error: error.raw_os_error().map(|code| code as u32),
    }
}

/// Owns a buffer allocated by the Windows security APIs.
struct LocalAllocation(*mut c_void);

impl Drop for LocalAllocation {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Only LocalAlloc-compatible API results enter this wrapper.
            unsafe { LocalFree(self.0) };
        }
    }
}

/// Word storage provides the alignment required by TOKEN_USER and its SID.
struct User(Vec<usize>);

impl User {
    fn current() -> Result<Self> {
        let mut token = null_mut();
        // SAFETY: The pseudo process handle is valid; token points to writable storage.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(last_error(Reason::TokenQueryFailed));
        }
        // SAFETY: A successful OpenProcessToken returns an owned, non-pseudo handle.
        let token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut size = 0;
        // SAFETY: A null buffer and zero length request the required buffer size.
        let result = unsafe {
            GetTokenInformation(token.as_raw_handle(), TokenUser, null_mut(), 0, &mut size)
        };
        if result != 0 {
            return Err(rejected(Reason::TokenQueryFailed));
        }
        // SAFETY: Read the error immediately after the failed size-query call.
        let error = unsafe { GetLastError() };
        if error != ERROR_INSUFFICIENT_BUFFER {
            return Err(native_error(Reason::TokenQueryFailed, error));
        }
        if (size as usize) < size_of::<TOKEN_USER>() {
            return Err(rejected(Reason::TokenQueryFailed));
        }
        let mut storage = vec![0usize; (size as usize).div_ceil(size_of::<usize>())];
        // SAFETY: Aligned storage holds at least size bytes and remains live below.
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                storage.as_mut_ptr().cast(),
                size,
                &mut size,
            )
        } == 0
        {
            return Err(last_error(Reason::TokenQueryFailed));
        }
        Ok(Self(storage))
    }

    fn sid(&self) -> PSID {
        // SAFETY: current() populated this aligned buffer with a TOKEN_USER.
        unsafe { (*self.0.as_ptr().cast::<TOKEN_USER>()).User.Sid }
    }

    fn private_descriptor(&self) -> Result<LocalAllocation> {
        let mut string = null_mut();
        // SAFETY: The token buffer owns a valid SID for the duration of this call.
        if unsafe { ConvertSidToStringSidW(self.sid(), &mut string) } == 0 {
            return Err(last_error(Reason::DescriptorCreateFailed));
        }
        let allocation = LocalAllocation(string.cast());
        let mut length = 0;
        // SAFETY: ConvertSidToStringSidW guarantees a NUL-terminated UTF-16 string.
        unsafe {
            while *string.add(length) != 0 {
                length += 1;
            }
        }
        // SAFETY: The preceding scan found the allocated string's terminator.
        let sid = String::from_utf16(unsafe { std::slice::from_raw_parts(string, length) })
            .map_err(|_| rejected(Reason::DescriptorCreateFailed))?;
        drop(allocation);
        // Protected DACL: no broad parent permissions are inherited. New files and
        // directories inherit access only for this user, SYSTEM and administrators.
        descriptor(&format!(
            "O:{sid}D:P(A;OICI;FA;;;{sid})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)"
        ))
    }

    fn trusted_owner(&self, sid: PSID) -> bool {
        if sid.is_null() {
            return false;
        }
        // SAFETY: GetSecurityInfo supplies the SID inside its live descriptor.
        unsafe {
            IsValidSid(sid) != 0
                && (EqualSid(sid, self.sid()) != 0
                    || IsWellKnownSid(sid, WinLocalSystemSid) != 0
                    || IsWellKnownSid(sid, WinBuiltinAdministratorsSid) != 0)
        }
    }

    fn trusted_trustee(&self, sid: PSID) -> bool {
        // SAFETY: validate_acl checked this ACE's SID bounds and validity first.
        self.trusted_owner(sid)
            || unsafe {
                IsWellKnownSid(sid, WinCreatorOwnerSid) != 0
                    || IsWellKnownSid(sid, WinCreatorOwnerRightsSid) != 0
            }
    }
}

fn descriptor(sddl: &str) -> Result<LocalAllocation> {
    let text: Vec<u16> = sddl.encode_utf16().chain(Some(0)).collect();
    let mut result = null_mut();
    // SAFETY: text is NUL-terminated and the output pointer is writable.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text.as_ptr(),
            SDDL_REVISION_1,
            &mut result,
            null_mut(),
        )
    } == 0
    {
        return Err(last_error(Reason::DescriptorCreateFailed));
    }
    Ok(LocalAllocation(result))
}

fn create_missing(directory: &Path, descriptor: &LocalAllocation) -> Result<()> {
    match directory.symlink_metadata() {
        Ok(_) => return Ok(()), // Existing objects are inspected, never repaired here.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(io_error(Reason::DirectoryOpenFailed, error)),
    }
    let parent = directory
        .parent()
        .ok_or_else(|| rejected(Reason::PathInvalid))?;
    create_missing(parent, descriptor)?;
    // Canonicalizing the existing parent supplies the extended-length Windows path.
    // The leaf is not followed; it may race into existence and will be checked below.
    let path = parent
        .canonicalize()
        .map_err(|error| io_error(Reason::DirectoryOpenFailed, error))?
        .join(
            directory
                .file_name()
                .ok_or_else(|| rejected(Reason::PathInvalid))?,
        );
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    if wide[..wide.len() - 1].contains(&0) {
        return Err(rejected(Reason::PathInvalid));
    }
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    // SAFETY: Both input buffers remain live; the descriptor came from the OS parser.
    if unsafe { CreateDirectoryW(wide.as_ptr(), &attributes) } == 0 {
        let error = unsafe { GetLastError() };
        if error != ERROR_ALREADY_EXISTS {
            return Err(native_error(Reason::DirectoryCreateFailed, error));
        }
    }
    Ok(())
}

/// Keep this handle for the operation: disallow deletion/rename of the checked leaf.
pub(crate) fn prepare(directory: &Path) -> Result<File> {
    if !directory.is_absolute() {
        return Err(rejected(Reason::PathInvalid));
    }
    let user = User::current()?;
    if !directory
        .try_exists()
        .map_err(|e| io_error(Reason::DirectoryOpenFailed, e))?
    {
        create_missing(directory, &user.private_descriptor()?)?;
    }
    // Metadata-only handles do not participate in the sharing checks needed to
    // block deletion. Directory data access makes omission of FILE_SHARE_DELETE
    // effective; it does not enumerate entries or change directory permissions.
    let file = OpenOptions::new()
        .access_mode(READ_CONTROL | FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(directory)
        .map_err(|error| io_error(Reason::DirectoryOpenFailed, error))?;
    let metadata = file
        .metadata()
        .map_err(|e| io_error(Reason::DirectoryOpenFailed, e))?;
    if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(rejected(Reason::ReparsePoint));
    }
    if !metadata.is_dir() {
        return Err(rejected(Reason::PathInvalid));
    }
    validate_handle(&file, &user)?;
    Ok(file)
}

pub(crate) fn validate_file(file: &File) -> Result<()> {
    let metadata = file
        .metadata()
        .map_err(|e| io_error(Reason::SecurityQueryFailed, e))?;
    if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(rejected(Reason::ReparsePoint));
    }
    if !metadata.is_file() {
        return Err(rejected(Reason::PathInvalid));
    }
    validate_handle(file, &User::current()?)
}

fn validate_handle(file: &File, user: &User) -> Result<()> {
    let mut owner = null_mut();
    let mut acl = null_mut();
    let mut security = null_mut();
    // SAFETY: The file handle remains open. All outputs refer to the returned
    // descriptor, which is owned until validation finishes.
    let status = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            null_mut(),
            &mut acl,
            null_mut(),
            &mut security,
        )
    };
    if status != 0 {
        return Err(native_error(Reason::SecurityQueryFailed, status));
    }
    let _security = LocalAllocation(security);
    if !user.trusted_owner(owner) {
        return Err(rejected(Reason::OwnerRejected));
    }
    // SAFETY: acl is an OS-produced pointer owned by _security for this call.
    unsafe { validate_acl(acl, user) }
}

/// The caller must keep the descriptor containing acl live for this call.
unsafe fn validate_acl(acl: *const ACL, user: &User) -> Result<()> {
    // A missing/null DACL grants unrestricted access. An empty DACL grants none.
    if acl.is_null() || unsafe { IsValidAcl(acl) } == 0 {
        return Err(rejected(Reason::AclInvalid));
    }
    let count = unsafe { (*acl).AceCount };
    for index in 0..count {
        let mut ace = null_mut();
        if unsafe { GetAce(acl, u32::from(index), &mut ace) } == 0 {
            return Err(last_error(Reason::SecurityQueryFailed));
        }
        // GetAce returns an aligned ACE within the validated ACL allocation.
        let header = unsafe { &*ace.cast::<ACE_HEADER>() };
        match u32::from(header.AceType) {
            ACCESS_DENIED_ACE_TYPE => continue,
            ACCESS_ALLOWED_ACE_TYPE => {}
            _ => return Err(rejected(Reason::AceUnsupported)),
        }
        let offset = offset_of!(ACCESS_ALLOWED_ACE, SidStart);
        // A SID consists of an 8-byte header plus its sub-authorities.
        if usize::from(header.AceSize) < offset + 8 {
            return Err(rejected(Reason::AclInvalid));
        }
        let sid = unsafe { ace.cast::<u8>().add(offset) };
        let sid_size = 8 + usize::from(unsafe { *sid.add(1) }) * 4;
        if offset + sid_size > usize::from(header.AceSize)
            || unsafe { IsValidSid(sid.cast()) } == 0
            || unsafe { GetLengthSid(sid.cast()) } as usize != sid_size
        {
            return Err(rejected(Reason::AclInvalid));
        }
        if !user.trusted_trustee(sid.cast()) {
            return Err(rejected(Reason::AclRejected));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::Security::GetSecurityDescriptorDacl;

    fn create_with_acl(path: &Path, sddl: &str) {
        create_missing(path, &descriptor(sddl).unwrap()).unwrap();
    }

    fn acl_snapshot(file: &File) -> Vec<u8> {
        let mut security = null_mut();
        let mut acl = null_mut();
        // SAFETY: The handle and output pointers remain valid during this call.
        let status = unsafe {
            GetSecurityInfo(
                file.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                &mut acl,
                null_mut(),
                &mut security,
            )
        };
        assert_eq!(status, 0);
        let _allocation = LocalAllocation(security);
        assert!(!acl.is_null());
        // SAFETY: GetSecurityInfo owns the complete ACL, whose header gives its size.
        let length = unsafe { (*acl).AclSize } as usize;
        unsafe { std::slice::from_raw_parts(acl.cast::<u8>(), length) }.to_vec()
    }

    #[test]
    fn private_creation_and_files_do_not_inherit_broad_parent_access() {
        let root = tempfile::tempdir().unwrap();
        let parent = root.path().join("broad-parent");
        create_with_acl(&parent, "D:P(A;OICI;FA;;;WD)");
        let directory = parent.join("private 日本語").join("state");
        let guard = prepare(&directory).unwrap();
        let path = directory.join("state.json");
        std::fs::write(&path, b"synthetic").unwrap();
        let file = File::open(&path).unwrap();
        validate_file(&file).unwrap();
        drop(file);
        drop(guard);
        // Existing safe state is reusable, without an ACL rewrite or a shell.
        let guard = prepare(&directory).unwrap();
        drop(guard);
    }

    #[test]
    fn unsafe_existing_acl_is_rejected_without_modification() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("unsafe-existing");
        create_with_acl(&directory, "D:P(A;OICI;FA;;;WD)");
        let handle = OpenOptions::new()
            .access_mode(READ_CONTROL)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(&directory)
            .unwrap();
        let before = acl_snapshot(&handle);
        assert_eq!(
            prepare(&directory).unwrap_err(),
            rejected(Reason::AclRejected)
        );
        assert!(before == acl_snapshot(&handle));
    }

    #[test]
    fn existing_file_with_broad_acl_is_rejected() {
        use windows_sys::Win32::{
            Security::{Authorization::SetSecurityInfo, PROTECTED_DACL_SECURITY_INFORMATION},
            Storage::FileSystem::WRITE_DAC,
        };

        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("state");
        let _guard = prepare(&directory).unwrap();
        let path = directory.join("existing.json");
        std::fs::write(&path, b"synthetic").unwrap();
        let file = OpenOptions::new()
            .access_mode(READ_CONTROL | WRITE_DAC)
            .open(&path)
            .unwrap();
        let security = descriptor("D:P(A;;FA;;;WD)").unwrap();
        let mut present = 0;
        let mut defaulted = 0;
        let mut acl = null_mut();
        // SAFETY: The synthetic descriptor owns its ACL and the outputs are writable.
        assert_ne!(
            unsafe {
                GetSecurityDescriptorDacl(security.0, &mut present, &mut acl, &mut defaulted)
            },
            0
        );
        // SAFETY: This changes only a disposable test file opened with WRITE_DAC.
        assert_eq!(
            unsafe {
                SetSecurityInfo(
                    file.as_raw_handle(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    null_mut(),
                    null_mut(),
                    acl,
                    null_mut(),
                )
            },
            0
        );
        assert_eq!(validate_file(&file), Err(rejected(Reason::AclRejected)));
    }

    #[test]
    fn directory_handles_prevent_delete_access_and_replacement_until_all_close() {
        use windows_sys::Win32::{
            Foundation::ERROR_SHARING_VIOLATION,
            Storage::FileSystem::{DELETE, FILE_SHARE_DELETE},
        };

        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("state");
        let first = prepare(&directory).unwrap();
        let second = prepare(&directory).unwrap();
        let other = root.path().join("moved");
        let delete_access = || {
            OpenOptions::new()
                .access_mode(DELETE)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&directory)
        };
        let error = delete_access().unwrap_err();
        assert_eq!(error.raw_os_error(), Some(ERROR_SHARING_VIOLATION as i32));
        assert!(std::fs::rename(&directory, &other).is_err());
        assert!(directory.is_dir());
        assert!(!other.exists());

        drop(first);
        let error = delete_access().unwrap_err();
        assert_eq!(error.raw_os_error(), Some(ERROR_SHARING_VIOLATION as i32));
        assert!(std::fs::rename(&directory, &other).is_err());

        drop(second);
        drop(delete_access().unwrap());
        std::fs::rename(&directory, &other).unwrap();
    }

    #[test]
    fn acl_policy_rejects_null_and_untrusted_inheritance_but_accepts_denies() {
        let user = User::current().unwrap();
        for (sddl, expected) in [
            ("D:NO_ACCESS_CONTROL", Err(rejected(Reason::AclInvalid))),
            ("D:P(A;OICIIO;FR;;;WD)", Err(rejected(Reason::AclRejected))),
            ("D:P(D;;FW;;;WD)(A;;FA;;;SY)(A;;FA;;;BA)", Ok(())),
        ] {
            let security = descriptor(sddl).unwrap();
            let mut present = 0;
            let mut defaulted = 0;
            let mut acl = null_mut();
            // SAFETY: security owns a valid descriptor and all output pointers are writable.
            assert_ne!(
                unsafe {
                    GetSecurityDescriptorDacl(security.0, &mut present, &mut acl, &mut defaulted)
                },
                0
            );
            // SAFETY: The descriptor remains live during validation.
            assert_eq!(unsafe { validate_acl(acl, &user) }, expected);
        }
    }
}
