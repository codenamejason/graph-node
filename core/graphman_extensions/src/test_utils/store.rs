use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use serde_json::Value;
use uuid::Uuid;

use crate::store::Execution;
use crate::GraphmanExtensionStore;

#[derive(Clone, Default)]
pub struct TestStore {
    pub expect_new_execution: Arc<Mutex<Vec<Result<()>>>>,
    pub expect_get_execution: Arc<Mutex<Vec<Result<Option<Execution>>>>>,
    pub expect_any_executions_in_progress: Arc<Mutex<Vec<Result<bool>>>>,
    pub expect_execution_in_progress: Arc<Mutex<Vec<Result<()>>>>,
    pub expect_execution_failed: Arc<Mutex<Vec<Result<()>>>>,
    pub expect_execution_succeeded: Arc<Mutex<Vec<Result<()>>>>,
    pub expect_handle_broken_executions: Arc<Mutex<Vec<Result<()>>>>,
}

impl TestStore {
    pub fn assert_no_expected_calls_left(&self) {
        let Self {
            expect_new_execution,
            expect_get_execution,
            expect_any_executions_in_progress,
            expect_execution_in_progress,
            expect_execution_failed,
            expect_execution_succeeded,
            expect_handle_broken_executions,
        } = self;

        assert!(expect_new_execution.lock().unwrap().is_empty());
        assert!(expect_get_execution.lock().unwrap().is_empty());
        assert!(expect_any_executions_in_progress.lock().unwrap().is_empty());
        assert!(expect_execution_in_progress.lock().unwrap().is_empty());
        assert!(expect_execution_failed.lock().unwrap().is_empty());
        assert!(expect_execution_succeeded.lock().unwrap().is_empty());
        assert!(expect_handle_broken_executions.lock().unwrap().is_empty());
    }
}

impl GraphmanExtensionStore for TestStore {
    fn new_execution(&self, _id: Uuid, _kind: String) -> Result<()> {
        self.expect_new_execution
            .lock()
            .unwrap()
            .pop()
            .expect("unexpected call to `new_execution`")
    }

    fn get_execution(&self, _id: Uuid) -> Result<Option<Execution>> {
        self.expect_get_execution
            .lock()
            .unwrap()
            .pop()
            .expect("unexpected call to `get_execution`")
    }

    fn any_executions_in_progress(&self, _kind: String) -> Result<bool> {
        self.expect_any_executions_in_progress
            .lock()
            .unwrap()
            .pop()
            .expect("unexpected call to `any_executions_in_progress`")
    }

    fn execution_in_progress(&self, _id: Uuid) -> Result<()> {
        self.expect_execution_in_progress
            .lock()
            .unwrap()
            .pop()
            .expect("unexpected call to `execution_in_progress`")
    }

    fn execution_failed(&self, _id: Uuid, _error_message: String) -> Result<()> {
        self.expect_execution_failed
            .lock()
            .unwrap()
            .pop()
            .expect("unexpected call to `execution_failed`")
    }

    fn execution_succeeded(&self, _id: Uuid, _command_output: Option<Value>) -> Result<()> {
        self.expect_execution_succeeded
            .lock()
            .unwrap()
            .pop()
            .expect("unexpected call to `execution_succeeded`")
    }

    fn handle_broken_executions(&self, _kind: String, _max_inactive_time: Duration) -> Result<()> {
        self.expect_handle_broken_executions
            .lock()
            .unwrap()
            .pop()
            .expect("unexpected call to `handle_broken_executions`")
    }
}
