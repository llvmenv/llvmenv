//! Get remote LLVM/Clang source

use failure::{bail, err_msg};
use log::info;
use reqwest;
use std::fs;
use std::path::*;
use std::process::Command;
use tempfile::TempDir;
use url::Url;

use crate::config::*;
use crate::error::*;

/// Remote LLVM/Clang resource
#[derive(Debug, PartialEq)]
pub enum Resource {
    /// Remote Subversion repository
    Svn { url: String },
    /// Remote Git repository
    Git { url: String, branch: Option<String> },
    /// Tar archive
    Tar { url: String },
}

impl Resource {
    /// Detect remote resorce from URL
    ///
    /// ```
    /// # use llvmenv::resource::Resource;
    /// // Official SVN repository
    /// let llvm_official_url = "http://llvm.org/svn/llvm-project/llvm/trunk";
    /// let svn = Resource::from_url(llvm_official_url).unwrap();
    /// assert_eq!(svn, Resource::Svn { url: llvm_official_url.into() });
    ///
    /// // GitHub mirror
    /// let github_mirror = "https://github.com/llvm-mirror/llvm";
    /// let git = Resource::from_url(github_mirror).unwrap();
    /// assert_eq!(git, Resource::Git { url: github_mirror.into(), branch: None });
    ///
    /// // Tar release
    /// let tar_url = "http://releases.llvm.org/6.0.1/llvm-6.0.1.src.tar.xz";
    /// let tar = Resource::from_url(tar_url).unwrap();
    /// assert_eq!(tar, Resource::Tar { url: tar_url.into() });
    /// ```
    pub fn from_url(url_str: &str) -> Result<Self> {
        // Check file extension
        if let Ok(filename) = get_filename_from_url(url_str) {
            for ext in &[".tar.gz", ".tar.xz", ".tar.bz2", ".tar.Z", ".tgz", ".taz"] {
                if filename.ends_with(ext) {
                    info!("Find archive extension '{}' at the end of URL", ext);
                    return Ok(Resource::Tar {
                        url: url_str.into(),
                    });
                }
            }

            if filename.ends_with("trunk") {
                info!("Find 'trunk' at the end of URL");
                return Ok(Resource::Svn {
                    url: url_str.into(),
                });
            }

            if filename.ends_with(".git") {
                info!("Find '.git' extension");
                return Ok(Resource::Git {
                    url: strip_branch_from_url(url_str)?,
                    branch: get_branch_from_url(url_str)?,
                });
            }
        }

        // Hostname
        let url = Url::parse(url_str)?;
        for service in &["github.com", "gitlab.com"] {
            if url.host_str() == Some(service) {
                info!("URL is a cloud git service: {}", service);
                return Ok(Resource::Git {
                    url: strip_branch_from_url(url_str)?,
                    branch: get_branch_from_url(url_str)?,
                });
            }
        }

        if url.host_str() == Some("llvm.org") {
            if url.path().starts_with("/svn") {
                info!("URL is LLVM SVN repository");
                return Ok(Resource::Svn {
                    url: url_str.into(),
                });
            }
            if url.path().starts_with("/git") {
                info!("URL is LLVM Git repository");
                return Ok(Resource::Git {
                    url: strip_branch_from_url(url_str)?,
                    branch: get_branch_from_url(url_str)?,
                });
            }
        }

        // Try access with git
        //
        // - SVN repository cannot handle git access
        // - Some Git service (e.g. GitHub) *can* handle svn access
        //
        // ```
        // git init
        // git remote add $url
        // git ls-remote       # This must fail for SVN repo
        // ```
        info!("Try access with git to {}", url_str);
        let tmp_dir = TempDir::new()?;
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .silent()
            .check_run()?;
        Command::new("git")
            .args(&["remote", "add", "origin"])
            .arg(url_str)
            .current_dir(tmp_dir.path())
            .silent()
            .check_run()?;
        match Command::new("git")
            .args(&["ls-remote"])
            .current_dir(tmp_dir.path())
            .silent()
            .check_run()
        {
            Ok(_) => {
                info!("Git access succeeds");
                Ok(Resource::Git {
                    url: strip_branch_from_url(url_str)?,
                    branch: get_branch_from_url(url_str)?,
                })
            }
            Err(_) => {
                info!("Git access failed. Regarded as a SVN repository.");
                Ok(Resource::Svn {
                    url: url_str.into(),
                })
            }
        }
    }

    pub fn download(&self, dest: &Path) -> Result<()> {
        if !dest.exists() {
            fs::create_dir_all(dest)?;
        }
        if !dest.is_dir() {
            bail!(
                "Download destination must be a directory: {}",
                dest.display()
            );
        }
        match self {
            Resource::Svn { url, .. } => Command::new("svn")
                .args(&["co", url.as_str(), "-r", "HEAD"])
                .arg(dest)
                .check_run()?,
            Resource::Git { url, branch } => {
                info!("Git clone {}", url);
                let mut git = Command::new("git");
                git.args(&["clone", url.as_str(), "--depth", "1"]).arg(dest);
                if let Some(branch) = branch {
                    git.args(&["-b", branch]);
                }
                git.check_run()?;
            }
            Resource::Tar { url } => {
                info!("Download Tar file: {}", url);
                let working = cache_dir()?.join(".tar_download");
                fs::create_dir_all(&working)?;
                let filename = get_filename_from_url(url)?;
                let path = working.join(&filename);
                let mut req = reqwest::get(url)?;
                let mut f = fs::File::create(&path)?;
                req.copy_to(&mut f)?;
                Command::new("tar")
                    .arg("xf")
                    .arg(filename)
                    .current_dir(&working)
                    .check_run()?;
                let d = fs::read_dir(&working)?
                    .map(|d| d.unwrap())
                    .filter(|d| d.file_type().unwrap().is_dir())
                    .nth(0)
                    .expect("Archive does not contains file");
                for contents in fs::read_dir(d.path())? {
                    let path = contents?.path();
                    if path.is_dir() {
                        let opt = fs_extra::dir::CopyOptions::new();
                        fs_extra::dir::copy(path, dest, &opt)?;
                    } else {
                        fs::copy(&path, dest.join(path.file_name().unwrap()))?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn update(&self, dest: &Path) -> Result<()> {
        match self {
            Resource::Svn { .. } => Command::new("svn")
                .arg("update")
                .current_dir(dest)
                .check_run()?,
            Resource::Git { .. } => Command::new("git")
                .arg("pull")
                .current_dir(dest)
                .check_run()?,
            Resource::Tar { .. } => {}
        }
        Ok(())
    }
}

fn get_filename_from_url(url_str: &str) -> Result<String> {
    let url = ::url::Url::parse(url_str)?;
    let seg = url.path_segments().ok_or(err_msg("URL parse failed"))?;
    let filename = seg.last().ok_or(err_msg("URL is invalid"))?;
    Ok(filename.to_string())
}

fn get_branch_from_url(url_str: &str) -> Result<Option<String>> {
    let url = ::url::Url::parse(url_str)?;
    Ok(url.fragment().map(ToOwned::to_owned))
}

fn strip_branch_from_url(url_str: &str) -> Result<String> {
    let mut url = ::url::Url::parse(url_str)?;
    url.set_fragment(None);
    Ok(url.into_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test donwloading this repo
    #[test]
    fn test_git_donwload() -> Result<()> {
        let git = Resource::Git {
            url: "http://github.com/termoshtt/llvmenv".into(),
            branch: None,
        };
        let tmp_dir = TempDir::new()?;
        git.download(tmp_dir.path())?;
        let cargo_toml = tmp_dir.path().join("Cargo.toml");
        assert!(cargo_toml.exists());
        Ok(())
    }

    #[test]
    fn test_tar_download() -> Result<()> {
        let tar = Resource::Tar {
            url: "https://github.com/termoshtt/llvmenv/archive/0.1.10.tar.gz".into(),
        };
        let tmp_dir = cache_dir()?.join("_llvmenv_test");
        if tmp_dir.exists() {
            fs::remove_dir_all(&tmp_dir)?;
        }
        tar.download(&tmp_dir)?;
        let cargo_toml = tmp_dir.join("Cargo.toml");
        assert!(cargo_toml.exists());
        Ok(())
    }

    #[test]
    fn test_get_filename_from_url() {
        let url = "http://releases.llvm.org/6.0.1/llvm-6.0.1.src.tar.xz";
        assert_eq!(get_filename_from_url(url).unwrap(), "llvm-6.0.1.src.tar.xz");
    }

    #[test]
    fn test_with_git_branches() {
        let github_mirror = "https://github.com/llvm-mirror/llvm";
        let git = Resource::from_url(github_mirror).unwrap();
        assert_eq!(git, Resource::Git { url: github_mirror.into(), branch: None });
        assert_eq!(Resource::from_url("https://github.com/llvm-mirror/llvm#release_80").unwrap(), Resource::Git {
            url: "https://github.com/llvm-mirror/llvm".into(),
            branch: Some("release_80".into())
        });
    }

}
