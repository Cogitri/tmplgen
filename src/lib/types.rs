//This file is part of tmplgen
//
//tmplgen is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//tmplgen is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
//along with tmplgen.  If not, see <http://www.gnu.org/licenses/>.

use crossbeam_deque::{Injector, Steal, Stealer, Worker};
use serde_derive::Deserialize;
use std::io::Read;
use std::iter;

/// The TemplateBuilder struct, which is used to build a [Template](crate::types::Template)
pub struct TmplBuilder {
    pub pkg_name: String,
    pub pkg_type: Option<PkgType>,
    pub pkg_info: Option<PkgInfo>,
}

pub struct Template {
    pub inner: String,
    pub name: String,
}

impl Read for Template {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.as_bytes().read(buf)
    }
}

/// The PkgType enum, containing all types of packages tmplgen can handle
#[derive(Copy, Clone, Eq, Ord, PartialOrd, Hash, Debug, PartialEq)]
pub enum PkgType {
    Crate,
    Gem,
    PerlDist,
}

/// The Dependencies struct that contains all dependencies a package might have
#[derive(Clone, Eq, Ord, PartialOrd, Hash, Default, Debug, PartialEq)]
pub struct Dependencies {
    pub host: Option<Vec<String>>,
    pub make: Option<Vec<String>>,
    pub run: Option<Vec<String>>,
}

/// The PkgInfo struct, that contains all info relevant to the package
#[derive(Clone, Eq, Ord, PartialOrd, Hash, Default, Debug, PartialEq)]
pub struct PkgInfo {
    pub pkg_name: String,
    pub version: String,
    pub description: Option<String>,
    pub homepage: String,
    pub license: Option<Vec<String>>,
    pub dependencies: Option<Dependencies>,
    pub sha: String,
    pub download_url: Option<String>,
}

pub(super) struct DownloadProgress<R> {
    pub inner: R,
    pub progress_bar: indicatif::ProgressBar,
}

impl<R: std::io::Read> std::io::Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct BuiltInDeps {
    pub perl: Vec<String>,
    pub ruby: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CorrectedVals {
    pub licenses: Vec<CorrectedLicenses>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CorrectedLicenses {
    pub is: String,
    pub should: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct NativeDepType {
    pub rust: Vec<NativeDeps>,
}

#[derive(Debug, Deserialize)]
pub(super) struct NativeDeps {
    pub name: String,
    pub dep: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct TomlData {
    pub builtin: BuiltInDeps,
    pub licenses: Vec<CorrectedLicenses>,
    pub native_deps: NativeDepType,
}

pub struct SharedQueue<T> {
    queue: Injector<T>,
    stealers: Option<Vec<Stealer<T>>>,
}

impl<T> SharedQueue<T> {
    pub fn new() -> SharedQueue<T> {
        SharedQueue {
            queue: Injector::<T>::new(),
            stealers: None,
        }
    }

    pub fn add_worker(&mut self) -> Worker<T> {
        let new_worker = Worker::new_fifo();

        self.stealers
            .unwrap_or_else(|| Vec::new())
            .push(new_worker.stealer());

        new_worker
    }

    pub fn push_job(&mut self, job: T) {
        self.queue.push(job);
    }

    pub fn get_work(&self, worker: &Worker<T>) -> Option<T> {
        // Pop a task from the local queue, if not empty.
        worker.pop().or_else(|| {
            iter::repeat_with(|| {
                // Try stealing a batch of tasks from the global queue.
                self.queue
                    .steal_batch_and_pop(&worker)
                    // Or try stealing a task from one of the other threads.
                    .or_else(|| self.stealers.unwrap().iter().map(|s| s.steal()).collect())
            })
            // Loop while no task was stolen and any steal operation needs to be retried.
            .find(|s| !s.is_retry())
            // Extract the stolen task, if there is one.
            .and_then(|s| s.success())
        })
    }
}
