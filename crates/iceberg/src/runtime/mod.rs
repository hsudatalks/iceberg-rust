// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

// This module contains the async runtime abstraction for iceberg.

pub(crate) use tokio::task::spawn;

use crate::{Error, ErrorKind};

impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Self {
        if error.is_panic() {
            std::panic::resume_unwind(error.into_panic());
        }
        Error::new(ErrorKind::Unexpected, "Iceberg task was cancelled")
            .with_retryable(true)
            .with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancelled_join_returns_retryable_error() {
        let task = spawn(std::future::pending::<()>());
        task.abort();
        let error = Error::from(task.await.unwrap_err());
        assert!(error.retryable());
    }

    #[cfg(panic = "unwind")]
    #[tokio::test]
    #[should_panic(expected = "loader invariant")]
    async fn joined_task_preserves_its_panic() {
        let task = spawn(async { panic!("loader invariant") });
        let join_error = task.await.unwrap_err();
        let _ = Error::from(join_error);
    }
}
