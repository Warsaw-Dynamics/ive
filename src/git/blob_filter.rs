//! A blob as a checkout writes it to the work tree: libgit2's `git_blob_filter`, which `git2` does not wrap.

use std::ffi::{CString, c_char, c_int, c_void};
use std::ptr;

use git2::{Binding, Blob, Buf};

use super::types::GitPath;

/// `git_blob_filter_options` of libgit2 1.9 with SHA-1 object ids, as `libgit2-sys` builds it.
#[repr(C)]
struct FilterOptions {
    version: c_int,
    flags: u32,
    /// unused, reserved by libgit2
    commit_id: *mut c_void,
    attr_commit_id: [u8; 20],
}

/// `GIT_BLOB_FILTER_OPTIONS_VERSION`
const OPTIONS_VERSION: c_int = 1;
/// `GIT_BLOB_FILTER_CHECK_FOR_BINARY`: a binary blob is left as it is, as a checkout leaves it
const CHECK_FOR_BINARY: u32 = 1;

unsafe extern "C" {
    /// `out` a `git_buf`, `blob` a `git_blob`
    fn git_blob_filter(out: *mut c_void, blob: *mut c_void, as_path: *const c_char, options: *mut FilterOptions) -> c_int;
}

/// `blob`'s content with the filters a checkout to `path` applies: line endings, `ident`, the attributes' filters
/// libgit2 knows.
pub(crate) fn checked_out(blob: &Blob<'_>, path: &GitPath) -> Result<Vec<u8>, git2::Error> {
    let as_path = CString::new(path.as_bytes()).map_err(|_| git2::Error::from_str(&format!("a path with a NUL byte: {}", path.display())))?;
    let mut options = FilterOptions { version: OPTIONS_VERSION, flags: CHECK_FOR_BINARY, commit_id: ptr::null_mut(), attr_commit_id: [0; 20] };
    let out = Buf::new();
    // SAFETY: `out` and `blob` are live libgit2 objects for the call, `as_path` a NUL-terminated string, `options` laid
    // out as libgit2 1.9 declares it. `out` may point into the blob's content, so it is copied while `blob` lives.
    let code = unsafe { git_blob_filter(out.raw().cast(), blob.raw().cast(), as_path.as_ptr(), &mut options) };
    if code < 0 {
        return Err(git2::Error::last_error(code));
    }
    Ok(out.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::TestRepo;

    #[test]
    fn should_give_a_blob_the_line_endings_a_checkout_writes() {
        // given
        let repo = TestRepo::init();
        repo.write(".gitattributes", "*.txt text eol=crlf\n");
        repo.write("a.txt", "one\ntwo\n");
        repo.commit("first");
        let head = repo.repo.head().unwrap().peel_to_tree().unwrap();
        let blob = repo.repo.find_blob(head.get_name("a.txt").unwrap().id()).unwrap();

        // when
        let found = checked_out(&blob, &GitPath::from_bytes(b"a.txt")).expect("filtered");

        // then
        assert_eq!((blob.content(), found.as_slice()), (b"one\ntwo\n".as_slice(), b"one\r\ntwo\r\n".as_slice()));
    }
}
